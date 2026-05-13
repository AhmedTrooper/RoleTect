<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useSettingsStore } from '../store/settings';

const store = useSettingsStore();

const providerInput = ref('openai');
const modelInput = ref('gpt-4o');
const apiKeyInput = ref('');
const isSaving = ref(false);
const showSuccess = ref(false);
const saveError = ref('');

// Dynamic Configuration
const providers = [
  { id: 'openai', name: 'OpenAI' },
  { id: 'gemini', name: 'Google Gemini' },
  { id: 'groq', name: 'Groq (Ultra-Fast)' }
];

const modelsByProvider: Record<string, {id: string, name: string}[]> = {
  openai: [
    { id: 'gpt-4o', name: 'GPT-4o' },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini' },
    { id: 'gpt-4.1', name: 'GPT-4.1' },
    { id: 'gpt-4.1-mini', name: 'GPT-4.1 Mini' },
    { id: 'gpt-4-turbo', name: 'GPT-4 Turbo' },
    { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo' }
  ],
  gemini: [
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash' },
    { id: 'gemini-2.0-flash', name: 'Gemini 2.0 Flash' },
    { id: 'gemini-2.0-pro', name: 'Gemini 2.0 Pro' }
  ],
  groq: [
    { id: 'llama3-70b-8192', name: 'Llama 3 70B' },
    { id: 'llama3-8b-8192', name: 'Llama 3 8B' },
    { id: 'mixtral-8x7b-32768', name: 'Mixtral 8x7B' },
    { id: 'gemma2-9b-it', name: 'Gemma 2 9B' },
    { id: 'llama-3.1-70b-versatile', name: 'Llama 3.1 70B' },
    { id: 'llama-3.1-8b-instant', name: 'Llama 3.1 8B' }
  ]
};

// Auto-switch to the first model if the provider changes
watch(providerInput, async (newProvider) => {
  const availableModels = modelsByProvider[newProvider];
  if (!availableModels.find(m => m.id === modelInput.value)) {
    modelInput.value = availableModels[0].id;
  }
  await store.loadProviderKeyStatus(newProvider);
  apiKeyInput.value = '';
});

const currentModels = computed(() => modelsByProvider[providerInput.value] || []);
const providerName = computed(() => providers.find(p => p.id === providerInput.value)?.name || 'AI');

onMounted(async () => {
  await store.loadSettings();
  providerInput.value = store.selectedAiProvider;
  modelInput.value = store.selectedAiModel;
  await store.loadProviderKeyStatus(providerInput.value);
});

const handleSave = async () => {
  isSaving.value = true;
  showSuccess.value = false;
  saveError.value = '';
  
  try {
    if (apiKeyInput.value.trim() !== '') {
      await store.saveApiKey(providerInput.value, apiKeyInput.value.trim());
      apiKeyInput.value = ''; 
    }
    await store.saveModelConfig(providerInput.value, modelInput.value);
    await store.loadSettings();
    providerInput.value = store.selectedAiProvider;
    modelInput.value = store.selectedAiModel;
    
    showSuccess.value = true;
    setTimeout(() => { showSuccess.value = false; }, 3000);
  } catch (error) {
    console.error(error);
    const message =
      typeof error === 'string'
        ? error
        : (error as Error).message || JSON.stringify(error);
    saveError.value = message || 'Failed to save settings.';
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="header">
      <h2>Engine Configuration</h2>
    </div>

    <div class="settings-grid">
      
      <div class="settings-card">
        <div class="card-header">
          <h3>Intelligence Engine</h3>
          <p>Select the AI network used for parsing and tailoring.</p>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <label>Provider</label>
            <select v-model="providerInput" class="custom-select">
              <option v-for="p in providers" :key="p.id" :value="p.id">
                {{ p.name }}
              </option>
            </select>
          </div>

          <div class="input-group">
            <label>Active Model</label>
            <select v-model="modelInput" class="custom-select">
              <option v-for="m in currentModels" :key="m.id" :value="m.id">
                {{ m.name }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <div class="settings-card">
        <div class="card-header">
          <h3>API Credentials</h3>
          <p>Your {{ providerName }} key is encrypted locally using AES-256. It never leaves your machine.</p>
        </div>
        
        <div class="input-group">
          <label>{{ providerName }} Secret Key</label>
          <input 
            v-model="apiKeyInput" 
            type="password" 
            :placeholder="store.hasSecureKey ? '•••••••••••••••• (Key saved for this provider)' : 'Paste API Key here...'"
            spellcheck="false"
          />
        </div>
      </div>

    </div>

    <div class="actions-footer">
      <span v-if="saveError" class="error-msg">{{ saveError }}</span>
      <transition name="fade">
        <span class="success-msg" v-if="showSuccess">
          <span class="dot"></span> Configuration Saved
        </span>
      </transition>
      
      <button class="btn-primary save-btn" @click="handleSave" :disabled="isSaving">
        {{ isSaving ? 'Securing...' : 'Save Configuration &rarr;' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.input-row {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.input-row > .input-group {
  flex: 1;
}

.settings-container {
  padding: 24px 20px 40px;
  height: 100%;
  display: flex;
  flex-direction: column;
  max-width: 960px;
  margin: 0 auto;
}

.header {
  margin-bottom: 24px;
}

.header h2 {
  margin: 0;
  font-weight: 700;
  color: var(--ink);
  font-size: 1.6rem;
}

.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 20px;
  flex-grow: 1;
}

.settings-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--shadow);
}

.card-header { margin-bottom: 16px; }
.card-header h3 { margin: 0 0 6px 0; color: var(--ink); font-size: 1.1rem; }
.card-header p { margin: 0; color: var(--muted); font-size: 0.95rem; line-height: 1.5; }

.input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  color: var(--accent);
  font-weight: 700;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

input, .custom-select {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 14px 16px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
  transition: all 0.2s ease;
  width: 100%;
}

input:focus, .custom-select:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2);
}

.custom-select { cursor: pointer; appearance: none; }

.actions-footer {
  margin-top: 28px;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 12px;
  padding-top: 16px;
  border-top: 1px solid var(--line);
}

.save-btn {
  background-color: var(--accent);
  color: #fff;
  border: none;
  border-radius: 12px;
  padding: 14px 18px;
  font-weight: 700;
  font-size: 1rem;
  cursor: pointer;
  transition: 0.2s;
}

.save-btn:hover:not(:disabled) { background-color: #0a6b5e; }
.save-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.success-msg {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent);
  font-weight: 600;
  font-size: 0.95rem;
}

.error-msg {
  color: var(--warning);
  font-weight: 600;
  font-size: 0.9rem;
}

.success-msg .dot {
  width: 6px;
  height: 6px;
  background-color: var(--accent);
  border-radius: 50%;
  box-shadow: 0 0 6px rgba(11, 123, 107, 0.5);
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

@media (min-width: 960px) {
  .settings-container { padding: 40px 32px 60px; }
  .header h2 { font-size: 2rem; }
  .input-row { flex-direction: row; gap: 20px; }
  .actions-footer { flex-direction: row; justify-content: flex-end; align-items: center; }
}
</style>