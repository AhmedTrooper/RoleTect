<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../store/settings';
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

// Load job details and base resumes on mount
onMounted(async () => {
  try {
    // TODO: Fetch job details from backend
    // const job = await invoke('get_job_details', { jobId: props.id });
    // jobDetails.value = job;
    
    // TODO: Fetch base resumes from backend
    // const resumes = await invoke('get_base_resumes');
    // standardResumes.value = resumes;
    // if (resumes.length > 0) selectedStandardResume.value = resumes[0].id;
    
    // For now, mock data
    jobDetails.value = {
      title: 'Senior Rust Developer',
      company: 'TechCorp',
      requirements: ['Rust', 'Tauri', 'SQLite', 'System Architecture'],
      core_responsibilities: ['Design systems', 'Lead team', 'Code review'],
      raw_job_content: 'We are looking for a Senior Rust Developer...'
    };
    
    standardResumes.value = [
      { id: 'base_1', name: 'Software Engineer Base' },
      { id: 'base_2', name: 'Frontend Heavy Base' }
    ];
    selectedStandardResume.value = 'base_1';
  } catch (err: any) {
    error.value = err.toString();
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
    
    const tailoredId = await invoke<string>('tailor_resume', {
      api_key: apiKey,
      job_id: props.id,
      base_resume_id: selectedStandardResume.value,
      custom_instruction: customInstruction.value || null,
    });
    
    // TODO: Fetch the generated LaTeX content by tailored_id
    // For now, display a placeholder
    generatedLatex.value = `% Tailored Resume (ID: ${tailoredId})\n\\documentclass{article}\n\\begin{document}\n% Content will be populated after fetching from DB\n\\end{document}`;
  } catch (err: any) {
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
          <select v-model="selectedStandardResume" class="dropdown">
            <option v-for="resume in standardResumes" :key="resume.id" :value="resume.id">
              {{ resume.name }}
            </option>
          </select>

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
.workspace { display: flex; flex-direction: column; height: 100%; }
.workspace-header { display: flex; align-items: center; gap: 20px; margin-bottom: 20px; border-bottom: 1px solid #313244; padding-bottom: 15px;}
.back-btn { background: none; border: none; color: #a6adc8; cursor: pointer; font-size: 1rem; }
.back-btn:hover { color: #cdd6f4; }
.workspace-header h1 { margin: 0; font-size: 1.8rem; }
.company { margin: 0; color: #a6e3a1; font-weight: bold; }

.split-view { display: flex; gap: 20px; flex-grow: 1; height: calc(100vh - 150px); }
.panel { display: flex; flex-direction: column; gap: 15px; }
.controls-panel { width: 35%; min-width: 300px; }
.preview-panel { width: 65%; background-color: #11111b; border-radius: 10px; border: 1px solid #313244; padding: 15px; display: flex; flex-direction: column; }

.card { background-color: #313244; padding: 20px; border-radius: 10px; }
.card h3 { margin-top: 0; color: #89b4fa; }

.skills-list { padding-left: 20px; color: #cdd6f4; font-size: 0.9rem; }
.dropdown { width: 100%; padding: 10px; background-color: #1e1e2e; color: #cdd6f4; border: 1px solid #45475a; border-radius: 5px; margin-top: 5px; }

.primary-btn { width: 100%; padding: 12px; background-color: #cba6f7; color: #11111b; font-weight: bold; border: none; border-radius: 5px; cursor: pointer; transition: 0.2s; }
.primary-btn:hover:not(:disabled) { background-color: #b4befe; }
.primary-btn:disabled { opacity: 0.6; cursor: not-allowed; }

.tabs { display: flex; justify-content: space-between; margin-bottom: 10px; }
.tab { background: none; color: #cdd6f4; border: none; font-weight: bold; cursor: pointer; padding: 5px 10px; }
.tab.active { border-bottom: 2px solid #89b4fa; color: #89b4fa; }
.secondary-btn { background-color: #89b4fa; color: #11111b; border-radius: 5px; padding: 5px 15px; }

.code-editor { flex-grow: 1; background-color: #1e1e2e; color: #a6adc8; font-family: monospace; padding: 15px; border: 1px solid #45475a; border-radius: 5px; resize: none; font-size: 0.9rem; }
.code-editor:focus { outline: none; border-color: #cba6f7; }

.pdf-viewer { flex-grow: 1; border: none; border-radius: 5px; background-color: white; margin-top: 10px; }
.mt-4 { margin-top: 1rem; }

.error-banner { background-color: rgba(255, 80, 80, 0.1); border: 1px solid rgba(255, 80, 80, 0.3); border-radius: 6px; padding: 12px 16px; margin-bottom: 15px; display: flex; justify-content: space-between; align-items: center; }
.error-banner span { color: #ff5555; font-size: 0.9rem; }
.error-banner button { background: none; border: none; color: #ff5555; cursor: pointer; font-size: 1rem; }

.instruction-input { width: 100%; margin-top: 8px; padding: 10px; background-color: #1e1e2e; color: #cdd6f4; border: 1px solid #45475a; border-radius: 5px; font-family: monospace; font-size: 0.85rem; resize: vertical; max-height: 120px; }
.instruction-input:focus { outline: none; border-color: #cba6f7; box-shadow: 0 0 0 1px rgba(203, 166, 247, 0.3); }
</style>