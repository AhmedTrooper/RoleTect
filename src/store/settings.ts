import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Stronghold } from '@tauri-apps/plugin-stronghold';
import { appDataDir } from '@tauri-apps/api/path';
import { readTextFile, writeTextFile, remove } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const hasSecureKey = ref(false);
  const selectedAiProvider = ref('openai'); // Ready for Gemini, Groq, etc.
  const selectedAiModel = ref('gpt-4o');

  const generateVaultPassword = async () => {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');
  };

  const getVaultPassword = async () => {
    const dir = await appDataDir();
    const passwordPath = `${dir}/stronghold.pass`;
    try {
      return (await readTextFile(passwordPath)).trim();
    } catch {
      const password = await generateVaultPassword();
      await writeTextFile(passwordPath, password);
      return password;
    }
  };

  // Helper to load the Stronghold instance and the Store
  const getVault = async () => {
    const dir = await appDataDir();
    const vaultPath = `${dir}/secrets.stronghold`;
    let password = await getVaultPassword();
    let stronghold;
    try {
      stronghold = await Stronghold.load(vaultPath, password);
    } catch {
      // If the vault cannot be decrypted (e.g., password mismatch), reset it.
      password = await generateVaultPassword();
      await writeTextFile(`${dir}/stronghold.pass`, password);
      try {
        await remove(vaultPath);
      } catch {
        // Ignore removal errors; load will recreate if needed.
      }
      stronghold = await Stronghold.load(vaultPath, password);
    }
    // We must load a client first, then get the store
    let client;
    try {
      client = await stronghold.loadClient('api_client');
    } catch {
      client = await stronghold.createClient('api_client');
      await stronghold.save();
    }
    const store = client.getStore();

    return { stronghold, store };
  };

  const saveApiKey = async (provider: string, key: string) => {
    try {
      const { stronghold, store } = await getVault();
      const storageKey = `ai_api_key_${provider}`;
      await store.insert(storageKey, Array.from(new TextEncoder().encode(key)));
      // Save is called on the stronghold instance!
      await stronghold.save(); 
      hasSecureKey.value = true;
    } catch (error) {
      console.error("Stronghold save error:", error);
      throw error;
    }
  };

  const getDecryptedKey = async (provider?: string): Promise<string | null> => {
    try {
      const targetProvider = provider || selectedAiProvider.value;
      const { store } = await getVault();
      const storageKey = `ai_api_key_${targetProvider}`;
      const keyBytes = await store.get(storageKey);
      if (keyBytes) {
        return new TextDecoder().decode(new Uint8Array(keyBytes as Iterable<number>));
      }
      return null;
    } catch {
      return null;
    }
  };

  const loadProviderKeyStatus = async (provider: string) => {
    try {
      const { store } = await getVault();
      const storageKey = `ai_api_key_${provider}`;
      const keyBytes = await store.get(storageKey);
      hasSecureKey.value = keyBytes !== null;
    } catch {
      hasSecureKey.value = false;
    }
  };




  const saveModelConfig = async (provider: string, model: string) => {
    try {
      // Pass both to Rust
      await invoke('save_model_pref', { provider, model });
      const config: { provider: string, model: string } = await invoke('get_model_pref');
      selectedAiProvider.value = config.provider;
      selectedAiModel.value = config.model;
      if (config.provider !== provider || config.model !== model) {
        throw new Error('Saved settings did not persist as expected.');
      }
    } catch (error) {
      console.error("SQLite save error:", error);
      const message =
        typeof error === 'string'
          ? error
          : (error as Error).message || JSON.stringify(error);
      throw new Error(message || 'Failed to save model settings.');
    }
  };

  const loadSettings = async () => {
    try {
      // Rust now returns the AiConfig object
      const config: { provider: string, model: string } = await invoke('get_model_pref');
      selectedAiProvider.value = config.provider;
      selectedAiModel.value = config.model;

      await loadProviderKeyStatus(selectedAiProvider.value);
    } catch (e) {
      console.error("Error loading settings:", e);
      hasSecureKey.value = false;
    }
  };

  return { 
    hasSecureKey, 
    selectedAiProvider,
    selectedAiModel, 
    saveApiKey, 
    getDecryptedKey,
    loadProviderKeyStatus,
    saveModelConfig,
    loadSettings 
  };
});