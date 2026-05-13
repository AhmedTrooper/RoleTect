<script setup lang="ts">
import { ref, onMounted } from 'vue';

const props = defineProps<{ jobId: number | null }>();
const emit = defineEmits(['go-back']);

// State
const isGenerating = ref(false);
const isCompilingPDF = ref(false);
const selectedStandardResume = ref(1);
const generatedLatex = ref('');
const pdfUrl = ref<string | null>(null);

// Mock Data (Later: fetch from SQLite using props.jobId)
const jobDetails = ref({
  title: 'Senior Rust Developer',
  company: 'TechCorp',
  requirements: ['Rust', 'Tauri', 'SQLite', 'System Architecture']
});

const standardResumes = ref([
  { id: 1, name: 'Software Engineer Base' },
  { id: 2, name: 'Frontend Heavy Base' }
]);

// Trigger AI Generation (The rig crate LangChain vibe)
const generateResume = async () => {
  isGenerating.value = true;
  
  // TODO: Call Tauri backend: await invoke('tailor_resume', { jobId: props.jobId, resumeId: selectedStandardResume.value })
  setTimeout(() => {
    generatedLatex.value = `\\documentclass{article}\n\\begin{document}\nHello World - Tailored for ${jobDetails.value.company}\n\\end{document}`;
    isGenerating.value = false;
  }, 1500); // Mock delay
};

// Trigger Tectonic PDF Compilation
const compilePdf = async () => {
  if (!generatedLatex.value) return;
  isCompilingPDF.value = true;
  
  // TODO: Call Tauri backend: const bytes = await invoke('compile_resume_to_pdf', { latexCode: generatedLatex.value })
  // Mocking PDF creation for now:
  setTimeout(() => {
    isCompilingPDF.value = false;
    // In production, you will create a Blob URL from the returned Uint8Array here
  }, 1000);
};
</script>

<template>
  <div class="workspace">
    <header class="workspace-header">
      <button class="back-btn" @click="emit('go-back')">← Back to Jobs</button>
      <div>
        <h1>{{ jobDetails.title }}</h1>
        <p class="company">@ {{ jobDetails.company }}</p>
      </div>
    </header>

    <div class="split-view">
      
      <div class="panel controls-panel">
        <div class="card">
          <h3>Job Context</h3>
          <ul class="skills-list">
            <li v-for="skill in jobDetails.requirements" :key="skill">{{ skill }}</li>
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

          <button class="primary-btn mt-4" @click="generateResume" :disabled="isGenerating">
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
</style>