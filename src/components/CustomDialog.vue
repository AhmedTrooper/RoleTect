<script setup lang="ts">
import { useDialogStore } from '../store/dialog';
import { Motion, AnimatePresence } from 'motion-v';
import { X, Info, HelpCircle, FileInput } from '@lucide/vue';
import { ref, onMounted, onUnmounted } from 'vue';

const store = useDialogStore();
const inputRef = ref<HTMLInputElement | null>(null);

const handleConfirm = () => {
  if (store.options?.type === 'prompt') {
    store.options.onConfirm(store.inputValue);
  } else {
    store.options?.onConfirm();
  }
};

const handleCancel = () => {
  store.options?.onCancel();
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') handleCancel();
  if (e.key === 'Enter' && store.options?.type !== 'prompt') handleConfirm();
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <AnimatePresence>
    <div v-if="store.isOpen" class="dialog-overlay">
      <Motion
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1 }"
        :exit="{ opacity: 0 }"
        class="backdrop"
        @click="handleCancel"
      />
      
      <Motion
        :initial="{ opacity: 0, scale: 0.9, y: 20 }"
        :animate="{ opacity: 1, scale: 1, y: 0 }"
        :exit="{ opacity: 0, scale: 0.9, y: 20 }"
        :transition="{ type: 'spring', damping: 25, stiffness: 300 }"
        class="dialog-card"
      >
        <div class="dialog-header">
          <div class="header-left">
            <Info v-if="store.options?.type === 'alert'" :size="18" class="icon alert-icon" />
            <HelpCircle v-else-if="store.options?.type === 'confirm'" :size="18" class="icon confirm-icon" />
            <FileInput v-else :size="18" class="icon prompt-icon" />
            <span class="dialog-title">{{ store.options?.title || 'System Message' }}</span>
          </div>
          <button class="close-btn" @click="handleCancel">
            <X :size="16" />
          </button>
        </div>

        <div class="dialog-body">
          <p class="dialog-message">{{ store.options?.message }}</p>
          
          <div v-if="store.options?.type === 'prompt'" class="prompt-input-wrapper">
            <input 
              ref="inputRef"
              v-model="store.inputValue" 
              class="dialog-input" 
              :placeholder="store.options.defaultValue"
              @keyup.enter="handleConfirm"
              autofocus
            />
          </div>
        </div>

        <div class="dialog-footer">
          <button 
            v-if="store.options?.type !== 'alert'" 
            class="btn-cancel" 
            @click="handleCancel"
          >
            {{ store.options?.cancelText || 'Cancel' }}
          </button>
          <button class="btn-confirm" @click="handleConfirm">
            {{ store.options?.confirmText || (store.options?.type === 'alert' ? 'Got it' : 'Confirm') }}
          </button>
        </div>
      </Motion>
    </div>
  </AnimatePresence>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 100000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.backdrop {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
}

.dialog-card {
  position: relative;
  width: 100%;
  max-width: 420px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dialog-title {
  font-size: 0.8rem;
  font-weight: 800;
  color: var(--ink);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.icon {
  color: var(--accent);
}

.alert-icon { color: var(--accent); }
.confirm-icon { color: #4cc9f0; }
.prompt-icon { color: #a371f7; }

.close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  padding: 4px;
  border-radius: 6px;
}

.close-btn:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.dialog-body {
  padding: 24px;
}

.dialog-message {
  margin: 0;
  font-size: 0.95rem;
  line-height: 1.6;
  color: var(--ink);
}

.prompt-input-wrapper {
  margin-top: 16px;
}

.dialog-input {
  width: 100%;
  padding: 12px 16px;
  background: var(--bg);
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
}

.dialog-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-soft);
}

.dialog-footer {
  padding: 16px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background: var(--bg-accent);
  border-top: 1px solid var(--line);
}

.btn-confirm, .btn-cancel {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-confirm {
  background: var(--accent);
  color: white;
  border: none;
}

.btn-confirm:hover {
  filter: brightness(1.1);
}

.btn-cancel {
  background: var(--surface-soft);
  border: 1px solid var(--line);
  color: var(--ink);
}

.btn-cancel:hover {
  background: var(--surface);
  border-color: var(--muted);
}
</style>