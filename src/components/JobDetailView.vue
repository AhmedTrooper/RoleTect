<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';
import { useResumesStore } from '../store/resumes';
import { useRouter } from 'vue-router';

interface JobData {
  title: string;
  company: string;
  requirements: string[];
  core_responsibilities: string[];
  raw_job_content: string;
}

interface BaseResume {
  id: string;
  name: string;
}

const router = useRouter();
const settingsStore = useSettingsStore();
const resumesStore = useResumesStore();

const props = defineProps<{ id: string }>();
const emit = defineEmits(['go-back']);

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
const jobDetails = ref<JobData | null>(null);
const standardResumes = ref<BaseResume[]>([]);
const resumesLoadError = ref<string | null>(null);
const isLoadingResumes = ref(false);

// Load job details and base resumes on mount
onMounted(async () => {
  try {
    // TODO: Fetch job details from backend
    // const job = await invoke('get_job_details', { jobId: props.id });
    // jobDetails.value = job;
    
    // Load base resumes from store and keep only those with LaTeX content
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
    
    // For now, mock data
    jobDetails.value = {
      title: 'Senior Rust Developer',
      company: 'TechCorp',
      requirements: ['Rust', 'Tauri', 'SQLite', 'System Architecture'],
      core_responsibilities: ['Design systems', 'Lead team', 'Code review'],
      raw_job_content: 'We are looking for a Senior Rust Developer...'
    };
    
    if (isLoadingResumes.value) {
      resumesLoadError.value = null;
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
    // console.log(apiKey)
    
    const tailoredId = await invoke<string>('tailor_resume', {
      apiKey: apiKey,
      jobId: props.id,
      baseResumeId: selectedStandardResume.value,
      customInstruction: customInstruction.value || null,
    });
    
    // TODO: Fetch the generated LaTeX content by tailored_id
    // For now, display a placeholder
    generatedLatex.value = `% Tailored Resume (ID: ${tailoredId})\n\\documentclass{article}\n\\begin{document}\n% Content will be populated after fetching from DB\n\\end{document}`;
  } catch (err: any) {
    console.log("Error found")
    error.value = err.toString();
  } finally {
    isGenerating.value = false;
  }
};

// Trigger PDF Compilation (stub for now)
const compilePdf = async () => {
  if (!generatedLatex.value) return;
  isCompilingPDF.value = true;
  
  try {
    // TODO: Call Tauri backend: const bytes = await invoke('compile_resume_to_pdf', { latexCode: generatedLatex.value })
    // Mocking PDF creation for now:
    setTimeout(() => {
      isCompilingPDF.value = false;
    }, 1000);
  } catch (err: any) {
    error.value = err.toString();
  }
};

const goBack = () => {
  router.push('/');
};
</script>

<template>
  <div class="workspace" v-if="!isLoading">
    <header class="workspace-header">
      <button class="back-btn" @click="goBack">← Back to Jobs</button>
      <div>
        <h1>{{ jobDetails?.title }}</h1>
        <p class="company">@ {{ jobDetails?.company }}</p>
      </div>
    </header>

    <div class="error-banner" v-if="error">
      <span>{{ error }}</span>
      <button @click="error = null">✕</button>
    </div>

    <div class="split-view">
      
      <div class="panel controls-panel">
        <div class="card">
          <h3>Job Context</h3>
          <ul class="skills-list">
            <li v-for="skill in jobDetails?.requirements" :key="skill">{{ skill }}</li>
          </ul>
        </div>

        <div class="card">
          <h3>Core Responsibilities</h3>
          <ul class="skills-list">
            <li v-for="resp in jobDetails?.core_responsibilities" :key="resp">{{ resp }}</li>
          </ul>
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
          <button class="tab active">LaTeX Source</button>
          <button class="tab secondary-btn" @click="compilePdf" :disabled="!generatedLatex || isCompilingPDF">
            {{ isCompilingPDF ? '⚙️ Compiling...' : '📄 Compile to PDF' }}
          </button>
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
.workspace { display: flex; flex-direction: column; height: 100%; padding: 24px 20px 40px; }
.workspace-header { display: flex; flex-direction: column; gap: 10px; margin-bottom: 16px; border-bottom: 1px solid var(--line); padding-bottom: 12px; }
.back-btn { background: var(--surface); border: 1px solid var(--line); color: var(--muted); cursor: pointer; font-size: 0.95rem; padding: 8px 12px; border-radius: 10px; width: fit-content; }
.back-btn:hover { color: var(--ink); border-color: var(--accent); }
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
.tab { background: none; color: var(--muted); border: none; font-weight: 700; cursor: pointer; padding: 6px 8px; }
.tab.active { border-bottom: 2px solid var(--accent); color: var(--ink); }
.secondary-btn { background-color: var(--accent); color: #fff; border-radius: 10px; padding: 6px 14px; }

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