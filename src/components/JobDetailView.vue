<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';
import { useResumesStore } from '../store/resumes';
import { useJobsStore, Job } from '../store/jobs';
import { useRouter } from 'vue-router';

interface BaseResume {
  id: string;
  name: string;
}

const router = useRouter();
const settingsStore = useSettingsStore();
const resumesStore = useResumesStore();
const jobsStore = useJobsStore();

const props = defineProps<{ id: string }>();

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

import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';

// ... inside script setup ...
const isDownloading = ref(false);
const pdfBytesBuffer = ref<Uint8Array | null>(null);

const isFixing = ref(false);
const compilationError = ref<string | null>(null);

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
        <button class="back-btn" @click="goBack">← Back</button>
        <div>
          <h1>{{ jobDetails?.job_title }}</h1>
          <p class="company">@ {{ jobDetails?.company_name }}</p>
        </div>
      </div>
      <button class="delete-btn" @click="deleteJob">🗑️ Delete Job</button>
    </header>

    <div class="error-banner" v-if="error">
      <span>{{ error }}</span>
      <button @click="error = null">✕</button>
    </div>

    <div class="split-view">
      
      <div class="panel controls-panel">
        <div class="card">
          <h3>Job Details</h3>
          <div class="detail-row">
            <span class="label">Model:</span>
            <span class="value">{{ jobDetails?.work_model }}</span>
          </div>
          <div class="detail-row">
            <span class="label">Type:</span>
            <span class="value">{{ jobDetails?.employment_type }}</span>
          </div>
          <div class="detail-row" v-if="jobDetails?.reference_name">
            <span class="label">Referral:</span>
            <span class="value">{{ jobDetails?.reference_name }}</span>
          </div>
        </div>

        <div class="card" v-if="jobDetails?.requirements">
          <h3>Requirements</h3>
          <ul class="skills-list">
            <li v-for="req in JSON.parse(jobDetails.requirements)" :key="req">{{ req }}</li>
          </ul>
        </div>

        <div class="card" v-if="jobDetails?.core_responsibilities">
          <h3>Responsibilities</h3>
          <ul class="skills-list">
            <li v-for="res in JSON.parse(jobDetails.core_responsibilities)" :key="res">{{ res }}</li>
          </ul>
        </div>

        <div class="card">
          <h3>Raw Description</h3>
          <div class="raw-jd-preview">
            {{ jobDetails?.raw_jd }}
          </div>
        </div>

        <div class="card">
          <h3>Tailor Settings</h3>
          <label>Select Base Resume:</label>
          <select v-model="selectedStandardResume" class="dropdown" :disabled="isLoadingResumes">
            <option v-for="resume in standardResumes" :key="resume.id" :value="resume.id">
              {{ resume.name }}
            </option>
          </select>

          <p v-if="isLoadingResumes" class="inline-warning">Loading resume templates...</p>
          <p v-else-if="resumesLoadError" class="inline-warning">{{ resumesLoadError }}</p>

          <label style="margin-top: 16px;">Custom Instructions (Optional):</label>
          <textarea 
            v-model="customInstruction" 
            class="instruction-input" 
            placeholder="Add any custom tailoring instructions..."
            spellcheck="false"
          ></textarea>

          <button class="primary-btn mt-4" @click="generateResume" :disabled="isGenerating || !selectedStandardResume">
            {{ isGenerating ? '✨ AI is Tailoring...' : '✨ Generate Tailored Resume' }}
          </button>
        </div>
      </div>

      <div class="panel preview-panel">
        <div class="tabs">
          <div class="tab-group">
            <button class="tab active">LaTeX Source</button>
            <button 
              class="tab save-btn-inline" 
              @click="saveLatexContent"
            >💾 Save Code</button>
          </div>
          <div class="action-group">
            <button 
              v-if="compilationError" 
              class="tab ai-fix-btn" 
              @click="fixWithAi" 
              :disabled="isFixing"
            >
              {{ isFixing ? '✨ Fixing...' : '✨ AI Fix' }}
            </button>
            <button class="tab secondary-btn" @click="compilePdf" :disabled="!generatedLatex || isCompilingPDF">
              {{ isCompilingPDF ? '⚙️ Compiling...' : '📄 Compile to PDF' }}
            </button>
            <button 
              v-if="pdfBytesBuffer" 
              class="tab download-btn" 
              @click="downloadPdf" 
              :disabled="isDownloading"
            >
              {{ isDownloading ? '📥 Saving...' : '📥 Download PDF' }}
            </button>
          </div>
        </div>

        <div v-if="compilationError" class="compilation-error-log">
          <header>
            <strong>Compilation Log</strong>
            <button @click="compilationError = null">✕</button>
          </header>
          <pre>{{ compilationError }}</pre>
        </div>

        <textarea 
          v-model="generatedLatex" 
          class="code-editor" 
          placeholder="Generated LaTeX code will appear here..."
          spellcheck="false"
        ></textarea>

        <iframe v-if="pdfUrl" :src="pdfUrl" class="pdf-viewer"></iframe>
      </div>

    </div>
  </div>
</template>

<style scoped>
.detail-row { display: flex; justify-content: space-between; margin-bottom: 8px; font-size: 0.9rem; }
.detail-row .label { color: var(--muted); font-weight: 600; }
.detail-row .value { color: var(--ink); font-weight: 700; }

.raw-jd-preview {
  font-size: 0.85rem;
  color: var(--muted);
  line-height: 1.5;
  max-height: 150px;
  overflow-y: auto;
  white-space: pre-wrap;
  background: var(--surface-soft);
  padding: 10px;
  border-radius: 8px;
}

.workspace { display: flex; flex-direction: column; height: 100%; padding: 24px 20px 40px; }
.workspace-header { display: flex; flex-direction: row; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 16px; border-bottom: 1px solid var(--line); padding-bottom: 12px; }
.header-left { display: flex; align-items: center; gap: 16px; }
.back-btn { background: var(--surface); border: 1px solid var(--line); color: var(--muted); cursor: pointer; font-size: 0.95rem; padding: 8px 12px; border-radius: 10px; width: fit-content; }
.back-btn:hover { color: var(--ink); border-color: var(--accent); }
.delete-btn { background: rgba(255, 107, 107, 0.1); border: 1px solid var(--warning); color: var(--warning); padding: 8px 14px; border-radius: 10px; cursor: pointer; font-weight: 600; font-size: 0.9rem; transition: 0.2s; }
.delete-btn:hover { background: var(--warning); color: white; }
.workspace-header h1 { margin: 0; font-size: 1.6rem; color: var(--ink); }
.company { margin: 0; color: var(--accent); font-weight: 700; }

.split-view { display: flex; flex-direction: column; gap: 16px; flex-grow: 1; }
.panel { display: flex; flex-direction: column; gap: 14px; }
.controls-panel { width: 100%; }
.preview-panel { width: 100%; background-color: var(--surface); border-radius: 16px; border: 1px solid var(--line); padding: 14px; display: flex; flex-direction: column; box-shadow: var(--shadow); }

.card { background-color: var(--surface); padding: 16px; border-radius: 14px; border: 1px solid var(--line); box-shadow: var(--shadow); }
.card h3 { margin-top: 0; color: var(--ink); font-size: 1.05rem; }

.skills-list { padding-left: 18px; color: var(--muted); font-size: 0.95rem; }
.dropdown { width: 100%; padding: 12px; background-color: var(--surface); color: var(--ink); border: 1px solid var(--line); border-radius: 10px; margin-top: 6px; }

.primary-btn { width: 100%; padding: 12px; background-color: var(--accent); color: #fff; font-weight: 700; border: none; border-radius: 10px; cursor: pointer; transition: 0.2s; }
.primary-btn:hover:not(:disabled) { background-color: #0a6b5e; }
.primary-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.tabs { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.tab-group, .action-group { display: flex; gap: 8px; align-items: center; }
.tab { background: none; color: var(--muted); border: none; font-weight: 700; cursor: pointer; padding: 6px 8px; }
.tab.active { border-bottom: 2px solid var(--accent); color: var(--ink); }
.secondary-btn { background-color: var(--accent); color: #fff; border-radius: 10px; padding: 6px 14px; font-size: 0.9rem; }
.ai-fix-btn { background-color: #7048e8; color: white; border-radius: 10px; padding: 6px 14px; font-size: 0.9rem; transition: 0.2s; border: none; }
.ai-fix-btn:hover:not(:disabled) { background-color: #5f3dc4; }
.ai-fix-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.save-btn-inline { font-size: 0.8rem; color: var(--accent); opacity: 0.8; transition: 0.2s; margin-left: 8px; }
.save-btn-inline:hover { opacity: 1; transform: translateY(-1px); }

.download-btn { background-color: var(--surface-soft); color: var(--accent); border: 1px solid var(--accent); border-radius: 10px; padding: 6px 14px; font-size: 0.9rem; transition: 0.2s; }

.compilation-error-log { background: #2b1d1d; border: 1px solid #ff6b6b44; border-radius: 10px; margin-bottom: 12px; overflow: hidden; }
.compilation-error-log header { background: #3d2323; padding: 6px 12px; display: flex; justify-content: space-between; align-items: center; }
.compilation-error-log header strong { font-size: 0.75rem; text-transform: uppercase; color: #ff6b6b; letter-spacing: 0.05em; }
.compilation-error-log header button { background: none; border: none; color: #ff6b6b; cursor: pointer; padding: 0 4px; }
.compilation-error-log pre { margin: 0; padding: 12px; font-family: 'Monaco', monospace; font-size: 0.8rem; color: #fcc; white-space: pre-wrap; max-height: 120px; overflow-y: auto; }
.download-btn:hover { background-color: var(--accent); color: white; }

.code-editor { flex-grow: 1; background-color: var(--surface); color: var(--ink); font-family: 'Monaco', 'Menlo', monospace; padding: 14px; border: 1px solid var(--line); border-radius: 10px; resize: none; font-size: 0.9rem; min-height: 260px; }
.code-editor:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2); }

.pdf-viewer { flex-grow: 1; border: none; border-radius: 8px; background-color: white; margin-top: 10px; min-height: 260px; }
.mt-4 { margin-top: 1rem; }

.error-banner { background-color: rgba(180, 35, 24, 0.1); border: 1px solid rgba(180, 35, 24, 0.2); border-radius: 10px; padding: 10px 12px; margin-bottom: 14px; display: flex; justify-content: space-between; align-items: center; }
.error-banner span { color: var(--warning); font-size: 0.9rem; }
.error-banner button { background: none; border: none; color: var(--warning); cursor: pointer; font-size: 1rem; }

.instruction-input { width: 100%; margin-top: 8px; padding: 10px; background-color: var(--surface); color: var(--ink); border: 1px solid var(--line); border-radius: 10px; font-family: 'Monaco', 'Menlo', monospace; font-size: 0.85rem; resize: vertical; max-height: 140px; }
.instruction-input:focus { outline: none; border-color: var(--accent); box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2); }

label {
  color: var(--accent);
  font-weight: 700;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.inline-warning {
  margin: 10px 0 0;
  color: var(--warning);
  font-size: 0.85rem;
}

@media (min-width: 960px) {
  .workspace { padding: 40px 32px 60px; }
  .workspace-header { flex-direction: row; align-items: center; justify-content: space-between; }
  .split-view { flex-direction: row; gap: 20px; }
  .controls-panel { width: 38%; min-width: 320px; }
  .preview-panel { width: 62%; }
}
</style>