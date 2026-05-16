<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save, message } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { Motion, AnimatePresence } from 'motion-v';
import { useSettingsStore } from '../store/settings';
import { useDialogStore } from '../store/dialog';
import { 
  Hammer, 
  Download, 
  Wand2, 
  RotateCw, 
  X,
  FileCode,
  Terminal
} from '@lucide/vue';

// Codemirror imports
import { Codemirror } from 'vue-codemirror';
import { latex, latexLanguage, autoCloseTags } from 'codemirror-lang-latex';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from '@codemirror/view';

const settingsStore = useSettingsStore();
const dialog = useDialogStore();

// Codemirror Extensions
const extensions = [
  latex(),
  latexLanguage,
  ...autoCloseTags,
  oneDark,
  EditorView.lineWrapping
];

// State
const latexCode = ref(`\\documentclass{article}
\\begin{document}
Hello from CVSynth LaTeX Compiler!
\\end{document}`);

const pdfUrl = ref<string | null>(null);
const pdfBytesBuffer = ref<Uint8Array | null>(null);
const isCompiling = ref(false);
const isFixing = ref(false);
const isRefining = ref(false);
const refinementInstruction = ref('');
const isDownloading = ref(false);
const compilationError = ref<string | null>(null);
const isAutoCompileEnabled = ref(true);
const isDirty = ref(false);
const editorContainer = ref<HTMLElement | null>(null);

// Persistence
onMounted(async () => {
  try {
    const savedCode = await invoke<string | null>('get_compiler_state');
    if (savedCode) {
      latexCode.value = savedCode;
      // Ensure initial load doesn't mark it as dirty
      setTimeout(() => { isDirty.value = false; }, 0);
    }
  } catch (err) {
    console.error('Failed to load compiler state:', err);
  }
});

// Auto-save logic
const saveState = async () => {
  try {
    await invoke('save_compiler_state', { latexContent: latexCode.value });
  } catch (err) {
    console.error('Failed to save compiler state:', err);
  }
};

watch(latexCode, () => {
  isDirty.value = true;
});

// Auto Compile on Blur
const handleBlur = () => {
  if (isDirty.value) {
    saveState();
    if (isAutoCompileEnabled.value) {
      compilePdf();
    }
    isDirty.value = false;
  }
};

// AI Refinement
const refineWithAi = async () => {
  if (!latexCode.value || !refinementInstruction.value.trim() || isRefining.value) return;
  
  isRefining.value = true;
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const refinedCode = await invoke<string>('refine_latex_with_ai', {
      provider,
      model,
      apiKey,
      currentLatex: latexCode.value,
      instruction: refinementInstruction.value.trim()
    });

    latexCode.value = refinedCode;
    refinementInstruction.value = ''; // Clear after success
    await saveState(); // Save after AI refinement
    await compilePdf();
  } catch (err: any) {
    console.error("AI Refinement Error:", err);
    await dialog.showAlert(err.toString(), 'AI Refinement Failed');
  } finally {
    isRefining.value = false;
  }
};

// Compile PDF
const compilePdf = async () => {
  if (!latexCode.value.trim()) return;
  
  isCompiling.value = true;
  compilationError.value = null;
  
  try {
    const pdfBytes = await invoke<number[]>('compile_resume_to_pdf', { 
      latexCode: latexCode.value 
    });
    
    pdfBytesBuffer.value = new Uint8Array(pdfBytes);
    const blob = new Blob([pdfBytesBuffer.value], { type: 'application/pdf' });
    
    if (pdfUrl.value) {
      URL.revokeObjectURL(pdfUrl.value);
    }
    
    pdfUrl.value = URL.createObjectURL(blob);
    await saveState(); // Save after successful compilation
  } catch (err: any) {
    console.error("Compilation Error:", err);
    compilationError.value = err.toString();
  } finally {
    isCompiling.value = false;
  }
};

// AI Fix
const fixWithAi = async () => {
  if (!latexCode.value || !compilationError.value || isFixing.value) return;
  
  isFixing.value = true;
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const fixedCode = await invoke<string>('fix_latex_with_ai', {
      provider,
      model,
      apiKey,
      brokenLatex: latexCode.value,
      errorLogs: compilationError.value
    });

    latexCode.value = fixedCode;
    compilationError.value = null;
    await saveState(); // Save after AI fix
    await compilePdf();
  } catch (err: any) {
    console.error("AI Fix Error:", err);
    await dialog.showAlert(err.toString(), 'AI Fix Failed');
  } finally {
    isFixing.value = false;
  }
};

// Download PDF
const downloadPdf = async () => {
  if (!pdfBytesBuffer.value) return;
  isDownloading.value = true;
  
  try {
    const now = new Date();
    const timestamp = `${now.getFullYear()}${(now.getMonth() + 1).toString().padStart(2, '0')}${now.getDate().toString().padStart(2, '0')}_${now.getHours().toString().padStart(2, '0')}${now.getMinutes().toString().padStart(2, '0')}${now.getSeconds().toString().padStart(2, '0')}`;
    const defaultName = `document_whole_${timestamp}.pdf`;

    const filePath = await save({
      filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
      defaultPath: defaultName
    });

    if (filePath) {
      await writeFile(filePath, pdfBytesBuffer.value);
      
      // Extract filename from path
      const filename = filePath.split(/[/\\]/).pop() || defaultName;
      
      await invoke('record_download', {
        filename,
        downloadType: 'compiler',
        jobId: null,
        contentId: null
      });

      await dialog.showAlert('PDF downloaded successfully.', 'Success');
    }
  } catch (err: any) {
    console.error("Download Error:", err);
    await dialog.showAlert(err.toString(), 'Download Failed');
  } finally {
    isDownloading.value = false;
  }
};
</script>

<template>
  <div class="compiler-container">
    <!-- Loading Overlay -->
    <AnimatePresence>
      <Motion
        v-if="isCompiling || isFixing || isRefining"
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1 }"
        :exit="{ opacity: 0 }"
        class="loading-overlay"
      >
        <div class="loader-content">
          <RotateCw :size="48" class="spinner" />
          <h2>{{ isFixing ? 'AI DEBUGGING...' : isRefining ? 'AI REFINING...' : 'COMPILING LATEX...' }}</h2>
          <p>Please wait while the engine processes your code.</p>
        </div>
      </Motion>
    </AnimatePresence>

    <header class="compiler-header">
      <div class="header-left">
        <FileCode :size="20" class="header-icon" />
        <h1>Standalone Compiler</h1>
      </div>
      
      <div class="header-actions">
        <label class="auto-compile-toggle">
          <input type="checkbox" v-model="isAutoCompileEnabled">
          <span>Auto Compile</span>
        </label>
        
        <button 
          v-if="compilationError" 
          class="action-btn ai-btn" 
          @click="fixWithAi" 
          :disabled="isFixing"
        >
          <Wand2 :size="16" />
          <span>AI Fix</span>
        </button>
        
        <button 
          class="action-btn compile-btn" 
          @click="compilePdf" 
          :disabled="isCompiling"
        >
          <Hammer :size="16" />
          <span>Compile</span>
        </button>
        
        <button 
          v-if="pdfUrl" 
          class="action-btn download-btn" 
          @click="downloadPdf" 
          :disabled="isDownloading"
        >
          <Download :size="16" />
          <span>Download</span>
        </button>
      </div>
    </header>

    <main class="compiler-main">
      <div class="split-pane">
        <!-- Editor Section -->
        <section class="editor-section">
          <div class="pane-header">
            <FileCode :size="14" />
            <span>LATEX SOURCE</span>
          </div>
          <div class="editor-relative-wrapper" ref="editorContainer">
            <codemirror
              v-model="latexCode"
              placeholder="Enter your LaTeX code here..."
              :style="{ height: '100%' }"
              :autofocus="true"
              :indent-with-tab="true"
              :tab-size="2"
              :extensions="extensions"
              @blur="handleBlur"
              class="latex-editor-cm"
            />

            <AnimatePresence>
              <Motion 
                v-if="latexCode"
                class="refinement-bar"
                drag
                :drag-constraints="editorContainer || undefined"
                :drag-elastic="0.1"
                :initial="{ opacity: 0, y: -10, x: '-50%' }"
                :animate="{ opacity: 1, y: 0, x: '-50%' }"
                :exit="{ opacity: 0, y: -10, x: '-50%' }"
              >
                <input 
                  v-model="refinementInstruction" 
                  placeholder="Refine code (e.g. 'Add a table of contents')..."
                  @keyup.enter="refineWithAi"
                />
                <button @click="refineWithAi" :disabled="isRefining">
                  {{ isRefining ? '...' : '→' }}
                </button>
              </Motion>
            </AnimatePresence>
          </div>
        </section>

        <!-- Preview Section -->
        <section class="preview-section">
          <div class="pane-header">
            <Terminal :size="14" />
            <span>PDF PREVIEW</span>
          </div>
          <div v-if="pdfUrl" class="pdf-viewer">
            <iframe :src="pdfUrl"></iframe>
          </div>
          <div v-else class="empty-preview">
            <div class="placeholder-content">
              <Hammer :size="48" />
              <h3>No PDF generated</h3>
              <p>Click "Compile" to generate a preview of your LaTeX code.</p>
            </div>
          </div>
        </section>
      </div>

      <!-- Error Console -->
      <AnimatePresence>
        <Motion
          v-if="compilationError"
          :initial="{ y: 100, opacity: 0 }"
          :animate="{ y: 0, opacity: 1 }"
          :exit="{ y: 100, opacity: 0 }"
          class="error-console"
        >
          <div class="console-header">
            <div class="title">
              <X :size="14" class="error-icon" />
              <span>COMPILATION ERROR</span>
            </div>
            <button class="close-btn" @click="compilationError = null">
              <X :size="14" />
            </button>
          </div>
          <pre class="error-logs">{{ compilationError }}</pre>
        </Motion>
      </AnimatePresence>
    </main>
  </div>
</template>

<style scoped>
.compiler-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg);
}

.compiler-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  color: var(--accent);
}

.header-left h1 {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0;
  letter-spacing: 0.02em;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.auto-compile-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--muted);
  cursor: pointer;
  user-select: none;
}

.auto-compile-toggle input {
  width: 14px;
  height: 14px;
  cursor: pointer;
  accent-color: var(--accent);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  border: 1px solid var(--line);
  background: var(--surface-soft);
  color: var(--ink);
}

.action-btn:hover:not(:disabled) {
  border-color: var(--muted);
  background: var(--surface);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.compile-btn {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.compile-btn:hover:not(:disabled) {
  opacity: 0.9;
  background: var(--accent);
}

.ai-btn {
  color: #a371f7;
  border-color: rgba(163, 113, 247, 0.3);
}

.ai-btn:hover:not(:disabled) {
  background: rgba(163, 113, 247, 0.1);
  border-color: #a371f7;
}

.compiler-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.split-pane {
  flex: 1;
  display: flex;
  min-height: 0;
}

.editor-section, .preview-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.editor-section {
  border-right: 1px solid var(--line);
}

.pane-header {
  height: 32px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.editor-relative-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #282c34; /* One Dark background */
}

.latex-editor-cm {
  flex: 1;
  width: 100%;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
}

:deep(.cm-editor) {
  height: 100%;
  outline: none !important;
}

:deep(.cm-scroller) {
  font-family: inherit;
}

:deep(.cm-content) {
  padding: 20px 0;
}

:deep(.cm-gutters) {
  background-color: #282c34 !important;
  border-right: 1px solid #3e4451 !important;
  color: #abb2bf !important;
}

.refinement-bar {
  position: absolute;
  top: 16px;
  left: 50%;
  width: 90%;
  max-width: 440px;
  background: var(--surface-soft);
  border: 1px solid var(--accent-soft);
  border-radius: 20px;
  display: flex;
  padding: 4px 14px;
  box-shadow: 0 12px 40px rgba(0,0,0,0.5);
  z-index: 20;
  cursor: grab;
  touch-action: none;
}

.refinement-bar:active {
  cursor: grabbing;
}

.refinement-bar input {
  flex: 1;
  background: none;
  border: none;
  color: var(--ink);
  font-size: 0.8rem;
  padding: 6px 0;
  outline: none;
}

.refinement-bar button {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 1rem;
  cursor: pointer;
}

.pdf-viewer {
  flex: 1;
  background: #525659;
}

.pdf-viewer iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.empty-preview {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-accent);
  color: var(--muted);
  text-align: center;
  padding: 40px;
}

.placeholder-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  max-width: 300px;
}

.placeholder-content h3 {
  font-size: 1rem;
  color: var(--ink);
  margin: 0;
}

.placeholder-content p {
  font-size: 0.8rem;
  margin: 0;
}

.error-console {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  max-height: 30%;
  background: #1e1e1e;
  border-top: 1px solid var(--warning);
  display: flex;
  flex-direction: column;
  z-index: 50;
}

.console-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: rgba(248, 81, 73, 0.1);
  border-bottom: 1px solid rgba(248, 81, 73, 0.2);
}

.console-header .title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--warning);
}

.close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  align-items: center;
}

.error-logs {
  flex: 1;
  margin: 0;
  padding: 12px;
  overflow-y: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: #f85149;
  line-height: 1.5;
  white-space: pre-wrap;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(13, 17, 23, 0.85);
  backdrop-filter: blur(8px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.loader-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  text-align: center;
}

.loader-content h2 {
  font-size: 1.2rem;
  font-weight: 800;
  color: var(--ink);
  margin: 0;
  letter-spacing: 0.1em;
}

.loader-content p {
  color: var(--muted);
  font-size: 0.9rem;
  margin: 0;
}

.spinner {
  color: var(--accent);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 768px) {
  .split-pane {
    flex-direction: column;
  }
  
  .editor-section {
    border-right: none;
    border-bottom: 1px solid var(--line);
  }
  
  .header-actions span {
    display: none;
  }
  
  .action-btn {
    padding: 8px;
  }
}
</style>
