<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { useSettingsStore } from '../store/settings';
import { CheckCircle, Info, Save, RotateCcw } from '@lucide/vue';
import { Motion, AnimatePresence } from 'motion-v';

const store = useSettingsStore();

// Tooltip State
const activeTooltip = ref<string | null>(null);

// --- 1. Draft State (Local only) ---
const providerInput = ref('');
const modelInput = ref('');
const apiKeyInput = ref('');

// UI feedback states
const isSaving = ref(false);
const showSuccess = ref(false);
const saveError = ref('');

// --- 2. Configuration Data ---
const providers = [
  { id: 'openai', name: 'OpenAI' },
  { id: 'gemini', name: 'Google Gemini' },
  { id: 'anthropic', name: 'Anthropic Claude' },
  { id: 'groq', name: 'Groq (Ultra-Fast)' }
];

const modelsByProvider: Record<string, {id: string, name: string}[]> = {
  openai: [
    { id: 'gpt-5.5-pro', name: 'GPT-5.5 Pro (Thinking)' },
    { id: 'gpt-5.5-thinking', name: 'GPT-5.5 Thinking' },
    { id: 'gpt-5.3-instant', name: 'GPT-5.3 Instant' },
    { id: 'gpt-5-main', name: 'GPT-5 Foundation' },
    { id: 'gpt-4.5', name: 'GPT-4.5 (Legacy)' },
    { id: 'o3', name: 'o3 (Reasoning)' }
  ],
  gemini: [
    { id: 'gemini-3.1-pro-preview', name: 'Gemini 3.1 Pro (Preview)' },
    { id: 'gemini-3.1-flash-preview', name: 'Gemini 3.1 Flash (Preview)' },
    { id: 'gemini-3.1-flash-lite-preview', name: 'Gemini 3.1 Flash-Lite (Preview)' },
    { id: 'gemini-3-pro-preview', name: 'Gemini 3.0 Pro (Preview)' },
    { id: 'gemini-3-flash-preview', name: 'Gemini 3.0 Flash (Preview)' },
    { id: 'gemini-2.5-pro', name: 'Gemini 2.5 Pro (Stable)' },
    { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash (Stable)' },
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro (Legacy)' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash (Legacy)' }
  ],
  anthropic: [
    { id: 'claude-opus-4-7', name: 'Claude Opus 4.7 (Power)' },
    { id: 'claude-sonnet-4-6', name: 'Claude Sonnet 4.6 (Balanced)' },
    { id: 'claude-opus-4-6', name: 'Claude Opus 4.6' },
    { id: 'claude-haiku-4-5', name: 'Claude Haiku 4.5 (Speed)' },
    { id: 'claude-sonnet-4-5', name: 'Claude Sonnet 4.5' },
    { id: 'claude-opus-4-5', name: 'Claude Opus 4.5' },
    { id: 'claude-3-7-sonnet-latest', name: 'Claude 3.7 Sonnet' },
    { id: 'claude-3-5-sonnet-latest', name: 'Claude 3.5 Sonnet (Legacy)' }
  ],
  groq: [
    { id: 'openai/gpt-oss-120b', name: 'GPT-OSS 120B (Groq Flagship)' },
    { id: 'meta-llama/llama-4-scout-17b-16e-instruct', name: 'Llama 4 Scout' },
    { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B' },
    { id: 'llama-3.3-70b-specdec', name: 'Llama 3.3 SpecDec' },
    { id: 'llama-3.1-70b-versatile', name: 'Llama 3.1 70B' },
    { id: 'mistral-medium-3.5', name: 'Mistral Medium 3.5' },
    { id: 'mistral-small-4', name: 'Mistral Small 4' }
  ]
};

// --- 3. Logic & Helpers ---

// Check if the current draft differs from the saved store
const hasChanges = computed(() => {
  return (
    providerInput.value !== store.selectedAiProvider ||
    modelInput.value !== store.selectedAiModel ||
    apiKeyInput.value.length > 0
  );
});

const providerName = computed(() => 
  providers.find(p => p.id === providerInput.value)?.name || 'AI'
);

const currentModels = computed(() => 
  modelsByProvider[providerInput.value] || []
);

// Resets local UI state to match the DB
const syncFromStore = async () => {
  await store.loadSettings();
  providerInput.value = store.selectedAiProvider;
  modelInput.value = store.selectedAiModel;
  apiKeyInput.value = ''; // Reset the input buffer
  await store.loadProviderKeyStatus(providerInput.value);
};

onMounted(syncFromStore);

// When provider changes, adjust the model but DON'T wipe the Store state yet
watch(providerInput, async (newProvider) => {
  if (!newProvider) return;
  const availableModels = modelsByProvider[newProvider];
  if (!availableModels.find(m => m.id === modelInput.value)) {
    modelInput.value = availableModels[0].id;
  }
  // Check if THIS specific provider has a key saved in Stronghold
  await store.loadProviderKeyStatus(newProvider);
});

const handleSave = async () => {
  isSaving.value = true;
  saveError.value = '';
  
  try {
    // 1. If user typed a new key, save it
    if (apiKeyInput.value.trim() !== '') {
      await store.saveApiKey(providerInput.value, apiKeyInput.value.trim());
    }
    
    // 2. Save the provider/model choice
    await store.saveModelConfig(providerInput.value, modelInput.value);
    
    // 3. Re-sync everything and show success
    await syncFromStore();
    showSuccess.value = true;
    setTimeout(() => { showSuccess.value = false; }, 3000);
  } catch (error: any) {
    saveError.value = error.message || 'Failed to save configuration.';
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="header">
      <h2>Engine Configuration</h2>
      <p class="subtitle">Customize how the AI intelligence layer behaves.</p>
    </div>

    <div class="settings-grid">
      <!-- Intelligence Engine -->
      <div class="settings-card">
        <div class="card-header">
          <h3>Intelligence Engine</h3>
          <p>The neural network used for parsing and tailoring.</p>
        </div>
        
        <div class="input-row">
          <div class="input-group">
            <div class="label-row" @mouseenter="activeTooltip = 'provider'" @mouseleave="activeTooltip = null">
              <label>Provider</label>
              <div class="tooltip-trigger">
                <Info :size="12" />
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'provider'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message"
                  >
                    Select AI Service
                  </Motion>
                </AnimatePresence>
              </div>
            </div>
            <select v-model="providerInput" class="custom-select">
              <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>

          <div class="input-group">
            <div class="label-row" @mouseenter="activeTooltip = 'model'" @mouseleave="activeTooltip = null">
              <label>Active Model</label>
              <div class="tooltip-trigger">
                <Info :size="12" />
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'model'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message"
                  >
                    Choose Model Logic
                  </Motion>
                </AnimatePresence>
              </div>
            </div>
            <select v-model="modelInput" class="custom-select">
              <option v-for="m in currentModels" :key="m.id" :value="m.id">{{ m.name }}</option>
            </select>
          </div>
        </div>
      </div>

      <!-- API Credentials -->
      <div class="settings-card">
        <div class="card-header">
          <div class="title-row">
            <h3>API Credentials</h3>
            <button 
              v-if="apiKeyInput" 
              class="text-btn" 
              @click="apiKeyInput = ''"
            >Clear Input</button>
          </div>
          <p>Your {{ providerName }} key is stored in an encrypted vault. It is never sent to our servers.</p>
        </div>
        
        <div class="input-group">
          <label>{{ providerName }} Secret Key</label>
          <input 
            v-model="apiKeyInput" 
            type="password" 
            :placeholder="store.hasSecureKey ? '•••••••••••••••• (Key saved)' : 'Enter API Key...'"
            spellcheck="false"
            class="form-input"
          />
        </div>
      </div>
    </div>

    <div class="actions-footer">
      <div class="status-area">
        <span v-if="saveError" class="error-msg">{{ saveError }}</span>
        <transition name="fade">
          <span v-if="showSuccess" class="success-msg">
            <CheckCircle :size="16" /> Configuration Saved
          </span>
        </transition>
      </div>
      
      <div class="button-group">
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'discard'" @mouseleave="activeTooltip = null">
          <button 
            v-if="hasChanges" 
            class="btn-secondary" 
            @click="syncFromStore" 
            :disabled="isSaving"
          >
            <RotateCcw :size="16" />
          </button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'discard' && hasChanges"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message"
            >
              Discard Changes
            </Motion>
          </AnimatePresence>
        </div>
        
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save'" @mouseleave="activeTooltip = null">
          <button 
            class="btn-primary" 
            @click="handleSave" 
            :disabled="isSaving || (providerInput === store.selectedAiProvider && modelInput === store.selectedAiModel && !apiKeyInput)"
          >
            <Save v-if="!isSaving" :size="16" />
            <RotateCcw v-else :size="16" class="spinner" />
          </button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'save'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message"
            >
              Save Configuration
            </Motion>
          </AnimatePresence>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Keeping your existing variables, adding layout improvements */
.settings-container {
  padding: 40px;
  max-width: 900px;
  margin: 0 auto;
}

.header { margin-bottom: 32px; }
.header h2 { font-size: 2rem; margin: 0; color: var(--ink); }
.subtitle { color: var(--muted); margin: 8px 0 0; }

.settings-grid { display: flex; flex-direction: column; gap: 24px; }

.settings-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--shadow);
}

.title-row { display: flex; justify-content: space-between; align-items: center; }

.text-btn {
  background: none; border: none;
  color: var(--warning); font-weight: 700; font-size: 0.75rem;
  text-transform: uppercase; cursor: pointer;
}

.input-row { display: flex; gap: 20px; margin-top: 20px; }
.input-group { flex: 1; display: flex; flex-direction: column; gap: 8px; }

.label-row {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: help;
}

.tooltip-trigger {
  color: var(--muted);
  display: flex;
  align-items: center;
  position: relative;
}

.flying-message {
  position: absolute;
  bottom: 140%;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: white;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.65rem;
  font-weight: 700;
  white-space: nowrap;
  pointer-events: none;
  z-index: 100;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  text-transform: none;
  letter-spacing: normal;
}

.flying-message::after {
  content: '';
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  border: 4px solid transparent;
  border-top-color: var(--accent);
}

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
}

label {
  color: var(--accent); font-weight: 700; font-size: 0.7rem;
  text-transform: uppercase; letter-spacing: 0.1em;
}

.form-input, .custom-select {
  width: 100%;
  padding: 12px 16px;
  font-size: 1rem;
}

.custom-select {
  cursor: pointer;
}

.actions-footer {
  margin-top: 40px;
  padding-top: 24px;
  border-top: 1px solid var(--line);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.button-group { display: flex; gap: 12px; }

.btn-primary, .btn-secondary {
  padding: 12px;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.btn-primary { background: var(--accent); color: white; border: none; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-secondary { background: none; border: 1px solid var(--line); color: var(--muted); }
.btn-secondary:hover { border-color: var(--ink); color: var(--ink); }

.success-msg { color: var(--accent); font-weight: 600; display: flex; align-items: center; gap: 8px; }

@media (max-width: 600px) {
  .input-row { flex-direction: column; }
  .actions-footer { flex-direction: column; gap: 20px; align-items: flex-start; }
}
</style>