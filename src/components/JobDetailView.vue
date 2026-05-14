<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { save, message } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { open } from '@tauri-apps/plugin-shell';
import { Motion, AnimatePresence } from 'motion-v';
import { useSettingsStore } from '../store/settings';
import { useResumesStore } from '../store/resumes';
import { useJobsStore, Job } from '../store/jobs';

import { 
  ArrowLeft, 
  Trash2, 
  ExternalLink, 
  Save, 
  Hammer, 
  Download, 
  Wand2, 
  Check, 
  X,
  Play,
  RotateCw,
  Info,
  ListChecks,
  Target,
  Settings,
  FileCode,
  Briefcase,
  Layers,
  Activity
} from '@lucide/vue';

interface BaseResume {
  id: string;
  name: string;
}

const router = useRouter();
const settingsStore = useSettingsStore();
const resumesStore = useResumesStore();
const jobsStore = useJobsStore();

const props = defineProps<{ id: string }>();

// Tooltip State
const activeTooltip = ref<string | null>(null);

// State
const isLoading = ref(true);
const isGenerating = ref(false);
const isCompilingPDF = ref(false);
const error = ref<string | null>(null);
const selectedStandardResume = ref<string | null>(null);
const customInstruction = ref('');
const generatedLatex = ref('');
const pdfUrl = ref<string | null>(null);

// Data
const jobDetails = ref<Job | null>(null);
const standardResumes = ref<BaseResume[]>([]);
const resumesLoadError = ref<string | null>(null);
const isLoadingResumes = ref(false);

// Load job details and base resumes on mount
onMounted(async () => {
  try {
    // 1. Fetch job details from backend
    jobDetails.value = await jobsStore.getJobById(props.id);
    customInstruction.value = jobDetails.value.custom_instruction || '';
    
    // 2. Load base resumes from store
    isLoadingResumes.value = true;
    await resumesStore.loadAllResumes();
    const withContent: BaseResume[] = [];
    for (const resume of resumesStore.resumes) {
      const detail = await resumesStore.getResumeById(resume.id);
      if (detail.latex_content && detail.latex_content.trim().length > 0) {
        withContent.push({ id: resume.id, name: resume.name });
      }
    }
    standardResumes.value = withContent;
    if (standardResumes.value.length > 0) {
      selectedStandardResume.value = standardResumes.value[0].id;
    }
    if (!standardResumes.value.length) {
      resumesLoadError.value = 'No resume templates with LaTeX content found. Add LaTeX in Resume Templates.';
    }
    isLoadingResumes.value = false;

    // 3. Fetch latest tailored resume if it exists
    const latest = await invoke<string | null>('get_latest_tailored_resume', { jobId: props.id });
    if (latest) {
      generatedLatex.value = latest;
    }
  } catch (err: any) {
    error.value = err.toString();
    isLoadingResumes.value = false;
  } finally {
    isLoading.value = false;
  }
});

// Trigger AI Generation
const generateResume = async () => {
  if (!jobDetails.value || !selectedStandardResume.value) return;
  
  isGenerating.value = true;
  error.value = null;
  
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");
    
    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const tailoredId = await invoke<string>('tailor_resume', {
      provider,
      model,
      apiKey: apiKey,
      jobId: props.id,
      baseResumeId: selectedStandardResume.value,
      customInstruction: customInstruction.value || null,
    });
    
    // Fetch the generated LaTeX content by tailored_id
    generatedLatex.value = await invoke<string>('get_tailored_resume', { id: tailoredId });
  } catch (err: any) {
    console.error("Tailoring Error:", err);
    error.value = err.toString();
  } finally {
    isGenerating.value = false;
  }
};


const isDownloading = ref(false);
const pdfBytesBuffer = ref<Uint8Array | null>(null);

const isFixing = ref(false);
const isRefining = ref(false);
const refinementInstruction = ref('');
const compilationError = ref<string | null>(null);

const refineWithAi = async () => {
  if (!generatedLatex.value || !refinementInstruction.value.trim()) return;
  isRefining.value = true;
  error.value = null;

  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const refinedCode = await invoke<string>('refine_latex_with_ai', {
      provider,
      model,
      apiKey,
      currentLatex: generatedLatex.value,
      instruction: refinementInstruction.value.trim()
    });

    generatedLatex.value = refinedCode;
    refinementInstruction.value = ''; // Clear after success
    error.value = "AI has refined the resume. Re-compiling...";
    
    await compilePdf();
  } catch (err: any) {
    console.error("AI Refinement Error:", err);
    error.value = `AI Refinement failed: ${err.toString()}`;
  } finally {
    isRefining.value = false;
  }
};

// Trigger PDF Compilation
const compilePdf = async () => {
  if (!generatedLatex.value) return;
  isCompilingPDF.value = true;
  error.value = null;
  compilationError.value = null;
  
  try {
    const pdfBytes = await invoke<number[]>('compile_resume_to_pdf', { 
      latexCode: generatedLatex.value 
    });
    
    // Store bytes for downloading later
    pdfBytesBuffer.value = new Uint8Array(pdfBytes);
    
    // Convert byte array to Blob and then to a URL for preview
    const blob = new Blob([pdfBytesBuffer.value], { type: 'application/pdf' });
    
    // Revoke old URL if it exists to avoid memory leaks
    if (pdfUrl.value) {
      URL.revokeObjectURL(pdfUrl.value);
    }
    
    pdfUrl.value = URL.createObjectURL(blob);

    // Auto-save on successful compilation
    await saveLatexContent();
  } catch (err: any) {
    console.error("PDF Compilation Error:", err);
    compilationError.value = err.toString();
    error.value = "LaTeX Compilation Failed. You can try 'AI Fix' or manually edit and Save.";
  } finally {
    isCompilingPDF.value = false;
  }
};

const saveLatexContent = async () => {
  try {
    await invoke('update_tailored_resume', {
      jobId: props.id,
      latexContent: generatedLatex.value
    });
    await message('Resume content saved successfully.', { title: 'Success', kind: 'info' });
  } catch (err: any) {
    console.error("Save Error:", err);
    error.value = `Failed to save changes: ${err.toString()}`;
  }
};

const fixWithAi = async () => {
  if (!generatedLatex.value || !compilationError.value) return;
  isFixing.value = true;
  error.value = null;

  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const fixedCode = await invoke<string>('fix_latex_with_ai', {
      provider,
      model,
      apiKey,
      brokenLatex: generatedLatex.value,
      errorLogs: compilationError.value
    });

    generatedLatex.value = fixedCode;
    error.value = "AI has suggested a fix. Trying to re-compile...";
    
    // Automatically re-compile with the fixed code
    await compilePdf();
  } catch (err: any) {
    console.error("AI Fix Error:", err);
    error.value = `AI Fix failed: ${err.toString()}`;
  } finally {
    isFixing.value = false;
  }
};

const downloadPdf = async () => {
  if (!pdfBytesBuffer.value) return;
  isDownloading.value = true;
  
  try {
    const suggestedName = `${jobDetails.value?.company_name || 'Resume'}_${jobDetails.value?.job_title || 'Tailored'}.pdf`.replace(/[^a-z0-9.]/gi, '_');
    
    const filePath = await save({
      filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
      defaultPath: suggestedName
    });

    if (filePath) {
      await writeFile(filePath, pdfBytesBuffer.value);
      // Optional: show a success message or notification
    }
  } catch (err: any) {
    console.error("Download Error:", err);
    error.value = `Failed to save PDF: ${err.toString()}`;
  } finally {
    isDownloading.value = false;
  }
};

const openJobUrl = async () => {
  if (jobDetails.value?.job_url) {
    try {
      await open(jobDetails.value.job_url);
    } catch (err: any) {
      console.error("Failed to open URL:", err);
      error.value = `Failed to open URL: ${err.toString()}`;
    }
  }
};

const goBack = () => {
  router.push('/');
};

const deleteJob = async () => {
  if (!confirm('Are you sure you want to delete this job application? This action cannot be undone.')) return;
  
  try {
    await jobsStore.deleteJob(props.id);
    router.push('/jobs');
  } catch (err: any) {
    error.value = err.toString();
  }
};
</script>

<template>
  <div class="workspace" v-if="!isLoading">
    <header class="workspace-header">
      <div class="header-left">
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'back'" @mouseleave="activeTooltip = null">
          <button class="back-btn" @click="goBack"><ArrowLeft :size="16" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'back'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message header-tooltip"
            >
              Back to List
            </Motion>
          </AnimatePresence>
        </div>
        <div class="job-info">
          <h1 class="title">{{ jobDetails?.job_title }}</h1>
          <span class="company">{{ jobDetails?.company_name }}</span>
          <div class="btn-tooltip-wrapper" v-if="jobDetails?.job_url" @mouseenter="activeTooltip = 'job-link'" @mouseleave="activeTooltip = null">
            <button class="link-btn" @click="openJobUrl"><ExternalLink :size="14" /></button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'job-link'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message header-tooltip"
              >
                Open Job Link
              </Motion>
            </AnimatePresence>
          </div>
        </div>
      </div>
      <div class="header-actions">
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'delete-job'" @mouseleave="activeTooltip = null">
          <button class="delete-btn" @click="deleteJob"><Trash2 :size="16" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'delete-job'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message header-tooltip delete-tooltip"
            >
              Delete Application
            </Motion>
          </AnimatePresence>
        </div>
      </div>
    </header>

    <AnimatePresence>
      <Motion
        v-if="error"
        :initial="{ height: 0, opacity: 0 }"
        :animate="{ height: 'auto', opacity: 1 }"
        :exit="{ height: 0, opacity: 0 }"
        class="error-banner"
      >
        <span>{{ error }}</span>
        <button @click="error = null">✕</button>
      </Motion>
    </AnimatePresence>

    <div class="split-view">
      <aside class="panel info-panel">
        <div class="section">
          <div class="section-header-icon" @mouseenter="activeTooltip = 'info-sec'" @mouseleave="activeTooltip = null">
            <Info :size="16" />
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'info-sec'"
                :initial="{ opacity: 0, x: 5, scale: 0.9 }"
                :animate="{ opacity: 1, x: 12, scale: 1 }"
                :exit="{ opacity: 0, x: 5, scale: 0.9 }"
                class="flying-message sidebar-tooltip"
              >
                Information
              </Motion>
            </AnimatePresence>
          </div>
          <div class="meta-grid">
            <div class="meta-icon-wrapper" @mouseenter="activeTooltip = 'work-model'" @mouseleave="activeTooltip = null">
              <Briefcase :size="14" />
              <AnimatePresence>
                <Motion v-if="activeTooltip === 'work-model'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Work Model</Motion>
              </AnimatePresence>
            </div>
            <span class="value">{{ jobDetails?.work_model }}</span>
            
            <div class="meta-icon-wrapper" @mouseenter="activeTooltip = 'emp-type'" @mouseleave="activeTooltip = null">
              <Layers :size="14" />
              <AnimatePresence>
                <Motion v-if="activeTooltip === 'emp-type'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Employment Type</Motion>
              </AnimatePresence>
            </div>
            <span class="value">{{ jobDetails?.employment_type }}</span>
            
            <div class="meta-icon-wrapper" @mouseenter="activeTooltip = 'status-meta'" @mouseleave="activeTooltip = null">
              <Activity :size="14" />
              <AnimatePresence>
                <Motion v-if="activeTooltip === 'status-meta'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Application Status</Motion>
              </AnimatePresence>
            </div>
            <span class="value">{{ jobDetails?.status }}</span>
          </div>
        </div>

        <div class="section scroll-section" v-if="jobDetails?.requirements">
          <div class="section-header-icon" @mouseenter="activeTooltip = 'req-sec'" @mouseleave="activeTooltip = null">
            <ListChecks :size="16" />
            <AnimatePresence>
              <Motion v-if="activeTooltip === 'req-sec'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Requirements</Motion>
            </AnimatePresence>
          </div>
          <ul class="tight-list">
            <li v-for="req in JSON.parse(jobDetails.requirements)" :key="req">{{ req }}</li>
          </ul>
        </div>

        <div class="section scroll-section" v-if="jobDetails?.core_responsibilities">
          <div class="section-header-icon" @mouseenter="activeTooltip = 'res-sec'" @mouseleave="activeTooltip = null">
            <Target :size="16" />
            <AnimatePresence>
              <Motion v-if="activeTooltip === 'res-sec'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Responsibilities</Motion>
            </AnimatePresence>
          </div>
          <ul class="tight-list">
            <li v-for="res in JSON.parse(jobDetails.core_responsibilities)" :key="res">{{ res }}</li>
          </ul>
        </div>

        <div class="section footer-section">
          <div class="section-header-icon" @mouseenter="activeTooltip = 'config-sec'" @mouseleave="activeTooltip = null">
            <Settings :size="16" />
            <AnimatePresence>
              <Motion v-if="activeTooltip === 'config-sec'" class="flying-message sidebar-tooltip" :initial="{ opacity: 0, x: 5 }" :animate="{ opacity: 1, x: 12 }">Configuration</Motion>
            </AnimatePresence>
          </div>
          <div class="form-group">
            <label>Base Template</label>
            <select v-model="selectedStandardResume" class="compact-select">
              <option v-for="resume in standardResumes" :key="resume.id" :value="resume.id">
                {{ resume.name }}
              </option>
            </select>
          </div>
          <div class="form-group">
            <label>Tailor Logic</label>
            <textarea 
              v-model="customInstruction" 
              class="compact-textarea" 
              placeholder="Custom tailoring rules..."
            ></textarea>
          </div>
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'run-intelligence'" @mouseleave="activeTooltip = null">
            <button class="btn-accent w-full" @click="generateResume" :disabled="isGenerating || !selectedStandardResume">
              <Play v-if="!isGenerating" :size="14" />
              <RotateCw v-else :size="14" class="spinner" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'run-intelligence'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message info-tooltip"
              >
                {{ isGenerating ? 'Tailoring...' : 'Run Intelligence' }}
              </Motion>
            </AnimatePresence>
          </div>
        </div>
      </aside>

      <div class="panel main-panel">
        <div class="panel-tabs">
          <div class="left-tabs">
            <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'source-tab'" @mouseleave="activeTooltip = null">
              <button class="tab active"><FileCode :size="14" /></button>
              <AnimatePresence>
                <Motion v-if="activeTooltip === 'source-tab'" class="flying-message tab-tooltip" :initial="{ opacity: 0, y: 5 }" :animate="{ opacity: 1, y: 0 }">Source Code</Motion>
              </AnimatePresence>
            </div>
            <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save-latex'" @mouseleave="activeTooltip = null">
              <button class="tab-btn" @click="saveLatexContent"><Save :size="14" /></button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'save-latex'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tab-tooltip"
                >
                  Save LaTeX
                </Motion>
              </AnimatePresence>
            </div>
          </div>
          <div class="right-tabs">
            <AnimatePresence>
              <div class="btn-tooltip-wrapper" v-if="compilationError" @mouseenter="activeTooltip = 'ai-fix'" @mouseleave="activeTooltip = null">
                <Motion
                  :initial="{ scale: 0.9, opacity: 0 }"
                  :animate="{ scale: 1, opacity: 1 }"
                  class="tab-btn ai-btn"
                  @click="fixWithAi"
                  :disabled="isFixing"
                >
                  <Wand2 :size="14" />
                </Motion>
                <AnimatePresence>
                  <Motion
                    v-if="activeTooltip === 'ai-fix'"
                    :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                    :animate="{ opacity: 1, y: 0, scale: 1 }"
                    :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                    :transition="{ duration: 0.15 }"
                    class="flying-message tab-tooltip"
                  >
                    AI Debug & Fix
                  </Motion>
                </AnimatePresence>
              </div>
            </AnimatePresence>
            <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'build-pdf'" @mouseleave="activeTooltip = null">
              <button class="tab-btn accent-btn" @click="compilePdf" :disabled="!generatedLatex || isCompilingPDF">
                <Hammer v-if="!isCompilingPDF" :size="14" />
                <RotateCw v-else :size="14" class="spinner" />
              </button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'build-pdf'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tab-tooltip"
                >
                  Compile PDF
                </Motion>
              </AnimatePresence>
            </div>
            <div class="btn-tooltip-wrapper" v-if="pdfBytesBuffer" @mouseenter="activeTooltip = 'export-pdf'" @mouseleave="activeTooltip = null">
              <button class="tab-btn" @click="downloadPdf" :disabled="isDownloading">
                <Download v-if="!isDownloading" :size="14" />
                <RotateCw v-else :size="14" class="spinner" />
              </button>
              <AnimatePresence>
                <Motion
                  v-if="activeTooltip === 'export-pdf'"
                  :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                  :animate="{ opacity: 1, y: 0, scale: 1 }"
                  :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                  :transition="{ duration: 0.15 }"
                  class="flying-message tab-tooltip"
                >
                  Download PDF
                </Motion>
              </AnimatePresence>
            </div>
          </div>
        </div>

        <AnimatePresence>
          <Motion
            v-if="compilationError"
            :initial="{ height: 0 }"
            :animate="{ height: 'auto' }"
            :exit="{ height: 0 }"
            class="error-log"
          >
            <header>
              <span>COMPILATION ERROR</span>
              <button @click="compilationError = null">✕</button>
            </header>
            <pre>{{ compilationError }}</pre>
          </Motion>
        </AnimatePresence>

        <div class="editor-container">
          <textarea v-model="generatedLatex" class="native-editor" spellcheck="false"></textarea>
          
          <AnimatePresence>
            <Motion 
              v-if="generatedLatex"
              class="refinement-bar"
              :initial="{ opacity: 0, y: 10, x: '-50%' }"
              :animate="{ opacity: 1, y: 0, x: '-50%' }"
              :exit="{ opacity: 0, y: 10, x: '-50%' }"
            >
              <input 
                v-model="refinementInstruction" 
                placeholder="Refine tailored resume (e.g. 'Shorten summary')..."
                @keyup.enter="refineWithAi"
              />
              <button @click="refineWithAi" :disabled="isRefining">
                {{ isRefining ? '...' : '→' }}
              </button>
            </Motion>
          </AnimatePresence>
        </div>

        <div v-if="pdfUrl" class="preview-pane">
          <iframe :src="pdfUrl"></iframe>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
}

.workspace-header {
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left { display: flex; align-items: center; gap: 12px; }
.back-btn { background: none; border: none; color: var(--muted); cursor: pointer; font-size: 1.2rem; padding: 0 4px; }
.back-btn:hover { color: var(--ink); }

.job-info { display: flex; align-items: center; gap: 8px; }
.title { font-size: 0.8rem; font-weight: 600; color: var(--ink); margin: 0; }
.company { font-size: 0.8rem; color: var(--muted); }
.link-btn { background: none; border: none; cursor: pointer; padding: 2px; font-size: 0.8rem; opacity: 0.7; }
.link-btn:hover { opacity: 1; }

.header-actions { display: flex; gap: 8px; }
.delete-btn { background: none; border: none; color: var(--warning); font-size: 0.7rem; font-weight: 600; cursor: pointer; text-transform: uppercase; }

.error-banner {
  background: var(--warning);
  color: #fff;
  padding: 4px 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
  z-index: 10;
}
.error-banner button { background: none; border: none; color: #fff; cursor: pointer; }

.split-view {
  flex: 1;
  display: flex;
  min-height: 0;
}

.panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.section-header-icon {
  color: var(--accent);
  margin-bottom: 12px;
  display: flex;
  position: relative;
  cursor: help;
}

.meta-icon-wrapper {
  color: var(--muted);
  display: flex;
  align-items: center;
  position: relative;
  cursor: help;
}

.sidebar-tooltip {
  left: 100%;
  top: 50%;
  bottom: auto;
  transform: translateY(-50%);
  margin-left: 12px;
  z-index: 2000;
}

.sidebar-tooltip::after {
  top: 50%;
  right: 100%;
  left: auto;
  bottom: auto;
  transform: translateY(-50%);
  border-top-color: transparent;
  border-right-color: var(--accent);
}

.info-panel {
  width: 260px;
  background: var(--bg-accent);
  border-right: 1px solid var(--line);
  padding: 12px;
  gap: 20px;
  overflow-y: auto;
}

.section h3 {
  font-size: 0.65rem;
  text-transform: uppercase;
  color: var(--muted);
  letter-spacing: 0.05em;
  margin: 0 0 8px 0;
}

.meta-grid {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 6px;
  font-size: 0.75rem;
}
.meta-grid .label { color: var(--muted); }
.meta-grid .value { color: var(--ink); font-weight: 500; }

.tight-list {
  padding-left: 12px;
  margin: 0;
  font-size: 0.75rem;
  color: var(--ink);
  opacity: 0.85;
}
.tight-list li { margin-bottom: 4px; }

.form-group { margin-bottom: 12px; }
.form-group label { display: block; font-size: 0.65rem; color: var(--muted); margin-bottom: 4px; }

.compact-select, .compact-textarea {
  width: 100%;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius-sm);
  color: var(--ink);
  font-size: 0.75rem;
  padding: 6px;
  outline: none;
}
.compact-textarea { height: 60px; resize: none; }

.btn-accent {
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  padding: 6px;
  font-weight: 600;
  font-size: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}
.btn-accent:disabled { opacity: 0.5; }

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
.delete-tooltip { background: var(--warning); left: auto; right: 0; transform: none; }
.delete-tooltip::after { border-bottom-color: var(--warning); left: auto; right: 8px; transform: none; }

.tab-tooltip { bottom: 140%; left: 50%; }
.info-tooltip { bottom: 140%; left: 50%; }

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.main-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.panel-tabs {
  height: 32px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--bg-accent);
  padding: 0 4px;
  border-bottom: 1px solid var(--line);
}

.left-tabs, .right-tabs { display: flex; align-items: center; }

.tab {
  height: 32px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--ink);
  background: var(--bg);
  border: none;
  border-top: 1px solid var(--accent);
}

.tab-btn {
  padding: 0 10px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--muted);
  background: none;
  border: none;
  cursor: pointer;
  transition: 0.15s;
}
.tab-btn:hover { color: var(--ink); }

.accent-btn { color: var(--accent); }
.ai-btn { color: #a371f7; }

.error-log {
  background: #1e1e1e;
  border-bottom: 1px solid var(--warning);
  max-height: 200px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.error-log header {
  padding: 4px 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.65rem;
  color: var(--warning);
  font-weight: 700;
}
.error-log header button { background: none; border: none; color: var(--muted); cursor: pointer; }
.error-log pre {
  margin: 0;
  padding: 8px 12px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.7rem;
  color: #f85149;
  overflow: auto;
}

.editor-container {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

.native-editor {
  flex: 1;
  width: 100%;
  background: var(--bg);
  border: none;
  color: var(--ink);
  padding: 16px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.85rem;
  line-height: 1.6;
  resize: none;
  outline: none;
}

.refinement-bar {
  position: absolute;
  bottom: 24px;
  left: 50%;
  width: 400px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 20px;
  display: flex;
  padding: 4px 12px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  z-index: 20;
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

.preview-pane {
  flex: 1;
  border-top: 1px solid var(--line);
  background: #525659;
  min-height: 200px;
}
.preview-pane iframe { width: 100%; height: 100%; border: none; }

.w-full { width: 100%; }

@media (max-width: 960px) {
  .info-panel { display: none; }
  .workspace-header { height: 44px; }
}
</style>