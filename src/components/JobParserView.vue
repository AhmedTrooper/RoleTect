<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useJobsStore } from '../store/jobs';
import { Motion, AnimatePresence } from 'motion-v';
import { ArrowLeft, Cpu, RotateCw } from '@lucide/vue';

const router = useRouter();
const jobsStore = useJobsStore();
const rawJobDescription = ref('');
const jobUrl = ref('');

// Tooltip State
const activeTooltip = ref<string | null>(null);

const handleParse = async () => {
  if (!rawJobDescription.value.trim() && !jobUrl.value.trim()) return;
  
  try {
    const slug = await jobsStore.parseNewJob(rawJobDescription.value, jobUrl.value);
    router.push(`/job/${slug}`);
  } catch (e) {
    console.error(e);
  }
};
</script>

<template>
  <div class="parser-container">
    <header class="header">
      <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'back'" @mouseleave="activeTooltip = null">
        <button class="back-btn" @click="router.push('/')"><ArrowLeft :size="16" /></button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'back'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message header-tooltip"
          >
            Back to Home
          </Motion>
        </AnimatePresence>
      </div>
      <h2>NEW APPLICATION</h2>
    </header>

    <div class="workspace">
      <div class="input-panel">
        <div class="field-group">
          <label>URL (OPTIONAL)</label>
          <input 
            v-model="jobUrl" 
            type="url" 
            placeholder="Link to job posting..."
            class="native-input"
          />
        </div>

        <div class="field-group expand">
          <label>RAW DESCRIPTION</label>
          <textarea 
            v-model="rawJobDescription" 
            placeholder="Paste description, requirements, etc..."
            spellcheck="false"
            class="native-textarea"
          ></textarea>
        </div>
      </div>

      <div class="side-panel">
        <div class="info-card">
          <h3>INTELLIGENCE</h3>
          <p>The AI will extract structured data to automate your resume tailoring.</p>
        </div>
        
        <div v-if="jobsStore.error" class="error-msg">
          {{ jobsStore.error }}
        </div>

        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'run-extraction'" @mouseleave="activeTooltip = null">
          <button 
            class="btn-primary w-full" 
            @click="handleParse" 
            :disabled="jobsStore.isLoading || (!rawJobDescription && !jobUrl)"
          >
            <RotateCw
              v-if="jobsStore.isLoading"
              :size="16"
              class="spinner"
            />
            <Cpu v-else :size="16" />
          </button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'run-extraction'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message"
            >
              {{ jobsStore.isLoading ? 'PARSING...' : 'RUN EXTRACTION' }}
            </Motion>
          </AnimatePresence>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.parser-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg);
}

.header {
  height: 36px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header h2 { font-size: 0.65rem; color: var(--muted); margin: 0; letter-spacing: 0.05em; }

.back-btn { background: none; border: none; color: var(--muted); cursor: pointer; display: flex; align-items: center; justify-content: center; width: 24px; height: 24px; }
.back-btn:hover { color: var(--ink); }

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
  align-items: center;
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
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
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

.header-tooltip { bottom: auto; top: 140%; }
.header-tooltip::after { top: auto; bottom: 100%; border-top-color: transparent; border-bottom-color: var(--accent); }

.workspace {
  flex: 1;
  display: flex;
  min-height: 0;
}

.input-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  gap: 20px;
}

.field-group { display: flex; flex-direction: column; gap: 8px; }
.field-group.expand { flex: 1; min-height: 0; }

label {
  font-size: 0.65rem;
  font-weight: 700;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.native-input, .native-textarea {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  color: var(--ink);
  padding: 10px 12px;
  font-size: 0.85rem;
  outline: none;
  transition: 0.15s;
}
.native-input:focus, .native-textarea:focus { border-color: var(--accent); }

.native-textarea {
  flex: 1;
  resize: none;
  font-family: 'JetBrains Mono', monospace;
  line-height: 1.5;
}

.side-panel {
  width: 280px;
  background: var(--bg-accent);
  border-left: 1px solid var(--line);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.info-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  padding: 12px;
}
.info-card h3 { font-size: 0.65rem; color: var(--accent); margin: 0 0 6px 0; }
.info-card p { font-size: 0.75rem; color: var(--muted); margin: 0; line-height: 1.4; }

.error-msg {
  font-size: 0.75rem;
  color: var(--warning);
  background: rgba(248, 81, 73, 0.1);
  padding: 8px;
  border-radius: var(--radius-sm);
}

.btn-primary {
  margin-top: auto;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-md);
  padding: 10px;
  font-weight: 700;
  font-size: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.w-full { width: 100%; }

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 960px) {
  .workspace { flex-direction: column; }
  .side-panel { width: 100%; border-left: none; border-top: 1px solid var(--line); }
}
</style>