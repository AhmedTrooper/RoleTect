import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from './settings';

export const useJobsStore = defineStore('jobs', () => {
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const settingsStore = useSettingsStore();

  const parseNewJob = async (rawJd: string): Promise<string> => {
    isLoading.value = true;
    error.value = null;
    
    try {
      await settingsStore.loadSettings();
      // Get the decrypted API key from Stronghold
      const apiKey = await settingsStore.getDecryptedKey(settingsStore.selectedAiProvider);
      if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

      // Send to Rust backend for AI parsing and DB storage
      const newSlug: string = await invoke('parse_and_save_job', { 
        apiKey, 
        rawJd 
      });
      
      return newSlug; 
    } catch (err: any) {
      error.value = err.toString();
      throw err;
    } finally {
      isLoading.value = false;
    }
  };

  return { isLoading, error, parseNewJob };
});