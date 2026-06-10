<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { ChevronDown } from '@lucide/vue';
import { AnimatePresence } from 'motion-v';

interface Option {
  value: any;
  label: string;
}

const props = defineProps<{
  modelValue: any;
  options: Option[];
  placeholder?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: any): void;
  (e: 'change', value: any): void;
}>();

const selectRef = ref<HTMLElement | null>(null);
const isOpen = ref(false);

const selectedLabel = computed(() => {
  const selected = props.options.find(opt => opt.value === props.modelValue);
  return selected ? selected.label : '';
});

const toggleDropdown = () => {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
};

const selectOption = (value: any) => {
  emit('update:modelValue', value);
  emit('change', value);
  isOpen.value = false;
};

const handleClickOutside = (event: MouseEvent) => {
  if (selectRef.value && !selectRef.value.contains(event.target as Node)) {
    isOpen.value = false;
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div class="custom-select-container" ref="selectRef">
    <button
      type="button"
      class="custom-select-trigger"
      :class="{ open: isOpen, disabled: disabled }"
      @click="toggleDropdown"
      :disabled="disabled"
    >
      <span class="trigger-content">
        <slot name="icon"></slot>
        <span class="custom-select-value">{{ selectedLabel || placeholder || 'Select...' }}</span>
      </span>
      <ChevronDown class="custom-select-arrow" :size="14" />
    </button>
    
    <AnimatePresence>
      <ul v-if="isOpen" class="custom-select-options">
        <li 
          v-for="opt in options" 
          :key="opt.value"
          class="custom-select-option"
          :class="{ selected: opt.value === modelValue }"
          @click="selectOption(opt.value)"
        >
          {{ opt.label }}
        </li>
      </ul>
    </AnimatePresence>
  </div>
</template>

<style scoped>
.custom-select-container {
  position: relative;
  width: 100%;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 8px 12px;
  background: var(--surface-soft, #1e1e24);
  border: 1px solid var(--border, #32323c);
  border-radius: 6px;
  color: var(--text, #f1f1f1);
  font-family: inherit;
  font-size: 14px;
  cursor: pointer;
  transition: border-color 0.15s ease, background-color 0.15s ease;
  outline: none;
  text-align: left;
}

.custom-select-trigger:hover:not(:disabled) {
  background: var(--surface, #121214);
  border-color: var(--accent, #238636);
}

.custom-select-trigger:focus:not(:disabled) {
  border-color: var(--accent, #238636);
  box-shadow: 0 0 0 2px var(--accent-soft, rgba(35, 134, 54, 0.25));
}

.custom-select-trigger.disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.trigger-content {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-arrow {
  flex-shrink: 0;
  color: var(--text-muted, #8b949e);
  transition: transform 0.15s ease;
  margin-left: 8px;
}

.custom-select-trigger.open .custom-select-arrow {
  transform: rotate(180deg);
}

.custom-select-options {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  width: 100%;
  max-height: 220px;
  overflow-y: auto;
  background: var(--surface, #121214);
  border: 1px solid var(--border, #32323c);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  padding: 4px;
  margin: 0;
  list-style: none;
}

.custom-select-option {
  padding: 8px 12px;
  border-radius: 4px;
  color: var(--text, #f1f1f1);
  font-size: 14px;
  cursor: pointer;
  transition: background 0.1s ease, color 0.1s ease;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-select-option:hover {
  background: var(--surface-soft, #1e1e24);
  color: var(--accent, #238636);
}

.custom-select-option.selected {
  background: var(--accent-soft, rgba(35, 134, 54, 0.15));
  color: var(--accent, #238636);
  font-weight: 600;
}
</style>
