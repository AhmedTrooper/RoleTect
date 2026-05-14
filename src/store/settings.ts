import { defineStore } from 'pinia';
import { ref, shallowRef } from 'vue';
import { Stronghold, Store } from '@tauri-apps/plugin-stronghold';
import { appDataDir, join } from '@tauri-apps/api/path';
import { readTextFile, writeTextFile, remove, exists } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

export const useSettingsStore = defineStore('settings', () => {
  const hasSecureKey = ref(false);
  const selectedAiProvider = ref('openai'); 
  const selectedAiModel = ref('gpt-4o');
  
  // Cache Stronghold and Store instances
  const strongholdInstance = shallowRef<Stronghold | null>(null);
  const storeInstance = shallowRef<Store | null>(null);

  const generateVaultPassword = () => {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');
  };

  const getVaultPassword = async () => {
    const dir = await appDataDir();
    const passwordPath = await join(dir, 'stronghold.pass');
    try {
      return (await readTextFile(passwordPath)).trim();
    } catch {
      const password = generateVaultPassword();
      await writeTextFile(passwordPath, password);
      return password;
    }
  };

  const getVault = async () => {
    if (strongholdInstance.value && storeInstance.value) {
      return { stronghold: strongholdInstance.value, store: storeInstance.value };
    }

    const dir = await appDataDir();
    const vaultPath = await join(dir, 'secrets.stronghold');
    let password = await getVaultPassword();
    let stronghold: Stronghold;

    try {
      stronghold = await Stronghold.load(vaultPath, password);
    } catch (error) {
      console.warn("Failed to load Stronghold, attempting reset:", error);
      // Only reset if it's likely a password issue or corruption
      password = generateVaultPassword();
      const passwordPath = await join(dir, 'stronghold.pass');
      await writeTextFile(passwordPath, password);
      
      if (await exists(vaultPath)) {
        await remove(vaultPath);
      }
      stronghold = await Stronghold.load(vaultPath, password);
    }

    let client;
    try {
      client = await stronghold.loadClient('api_client');
    } catch {
      client = await stronghold.createClient('api_client');
      await stronghold.save();
    }
    
    const store = client.getStore();
    
    strongholdInstance.value = stronghold;
    storeInstance.value = store;

    return { stronghold, store };
  };

  const saveApiKey = async (provider: string, key: string) => {
    try {
      const { stronghold, store } = await getVault();
      const storageKey = `ai_api_key_${provider}`;
      await store.insert(storageKey, Array.from(new TextEncoder().encode(key)));
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
      if (keyBytes && (keyBytes as number[]).length > 0) {
        return new TextDecoder().decode(new Uint8Array(keyBytes as Iterable<number>));
      }
      return null;
    } catch (error) {
      console.error("Stronghold get error:", error);
      return null;
    }
  };

  const loadProviderKeyStatus = async (provider: string) => {
    try {
      const { store } = await getVault();
      const storageKey = `ai_api_key_${provider}`;
      const keyBytes = await store.get(storageKey);
      hasSecureKey.value = keyBytes !== null && (keyBytes as number[]).length > 0;
    } catch (error) {
      console.error("Error loading key status:", error);
      hasSecureKey.value = false;
    }
  };

  const saveModelConfig = async (provider: string, model: string) => {
    try {
      await invoke('save_model_pref', { provider, model });
      const config: { provider: string, model: string } = await invoke('get_model_pref');
      selectedAiProvider.value = config.provider;
      selectedAiModel.value = config.model;
    } catch (error) {
      console.error("SQLite save error:", error);
      throw error;
    }
  };

  const loadSettings = async () => {
    try {
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