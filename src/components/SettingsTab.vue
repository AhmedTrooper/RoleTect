<script setup lang="ts">
import { ref } from 'vue';

const apiKey = ref('');
const isSaved = ref(false);

const saveSettings = async () => {
  // TODO: Call Tauri backend to securely save this key to local storage/keyring
  // await invoke('save_api_key', { key: apiKey.value });
  
  isSaved.value = true;
  setTimeout(() => isSaved.value = false, 3000); // Hide success message after 3s
};
</script>

<template>
  <div class="settings-tab">
    <h1>Settings</h1>
    
    <div class="settings-card">
      <h3>AI Configuration</h3>
      <p class="description">Enter your API key. This key is stored securely on your local machine and never sent to our servers.</p>
      
      <div class="input-group">
        <label for="apiKey">API Key</label>
        <input 
          id="apiKey" 
          v-model="apiKey" 
          type="password" 
          placeholder="sk-..."
        />
      </div>

      <button @click="saveSettings" class="save-btn">Save Key</button>
      
      <p v-if="isSaved" class="success-msg">✅ Settings saved successfully!</p>
    </div>
  </div>
</template>

<style scoped>
h1 { margin-top: 0; }

.settings-card {
  background-color: #313244;
  padding: 30px;
  border-radius: 10px;
  max-width: 600px;
}

.description {
  color: #a6adc8;
  font-size: 0.9rem;
  margin-bottom: 20px;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
}

input {
  padding: 10px;
  border-radius: 5px;
  border: 1px solid #45475a;
  background-color: #1e1e2e;
  color: #cdd6f4;
  font-family: monospace;
}

input:focus {
  outline: none;
  border-color: #89b4fa;
}

.save-btn {
  padding: 10px 20px;
  background-color: #a6e3a1;
  color: #11111b;
  border: none;
  border-radius: 5px;
  font-weight: bold;
  cursor: pointer;
}
.save-btn:hover { background-color: #94e2d5; }

.success-msg {
  color: #a6e3a1;
  margin-top: 15px;
  font-size: 0.9rem;
}
</style>