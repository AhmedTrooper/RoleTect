<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useResumesStore } from '../store/resumes';

const router = useRouter();
const resumesStore = useResumesStore();

const showNewResumeForm = ref(false);
const newResumeName = ref('');
const newResumeCategory = ref('');
const isCreating = ref(false);

onMounted(async () => {
  await resumesStore.loadAllResumes();
});

const navigateToResume = (resumeId: string) => {
  router.push(`/resume/${resumeId}`);
};

const toggleNewForm = () => {
  showNewResumeForm.value = !showNewResumeForm.value;
  if (!showNewResumeForm.value) {
    newResumeName.value = '';
    newResumeCategory.value = '';
  }
};

const handleCreateResume = async () => {
  if (!newResumeName.value.trim() || !newResumeCategory.value.trim()) {
    return;
  }
  
  isCreating.value = true;
  try {
    // Start with a basic LaTeX template
    const templateLatex = `\\documentclass[11pt]{article}
\\usepackage[margin=0.5in]{geometry}
\\usepackage{hyperref}

\\pagestyle{empty}

\\begin{document}

\\centerline{\\textbf{\\Large ${newResumeName.value}}}
\\vspace{0.5em}

\\section*{Contact}
Email: your.email@example.com | Phone: (123) 456-7890

\\section*{Professional Summary}
Brief overview of your professional background and career goals.

\\section*{Experience}
\\textbf{Job Title} | Company Name | 2020-Present
\\begin{itemize}
  \\item Achievement or responsibility
  \\item Achievement or responsibility
\\end{itemize}

\\section*{Education}
\\textbf{Degree} | University Name | 2020

\\section*{Skills}
Skill 1, Skill 2, Skill 3

\\end{document}`;

    const resumeId = await resumesStore.createNewResume(newResumeName.value, newResumeCategory.value, templateLatex);
    showNewResumeForm.value = false;
    newResumeName.value = '';
    newResumeCategory.value = '';
    
    // Navigate to the new resume's detail page
    router.push(`/resume/${resumeId}`);
  } catch (err: any) {
    console.error(err);
  } finally {
    isCreating.value = false;
  }
};
</script>

<template>
  <div class="resumes-container">
    <div class="header">
      <h2>Resume Templates</h2>
      <button class="btn-add" @click="toggleNewForm">+ New Template</button>
    </div>

    <div v-if="resumesStore.error" class="error-banner">
      {{ resumesStore.error }}
    </div>

    <div v-if="showNewResumeForm" class="form-card">
      <h3>Create New Resume Template</h3>
      <div class="form-group">
        <label>Template Name:</label>
        <input 
          v-model="newResumeName" 
          type="text" 
          placeholder="e.g., Senior Engineer Base"
          class="form-input"
        />
      </div>
      <div class="form-group">
        <label>Category:</label>
        <input 
          v-model="newResumeCategory" 
          type="text" 
          placeholder="e.g., Software Engineering"
          class="form-input"
        />
      </div>
      <div class="form-actions">
        <button 
          class="btn-cancel" 
          @click="toggleNewForm"
        >
          Cancel
        </button>
        <button 
          class="btn-save" 
          @click="handleCreateResume"
          :disabled="isCreating || !newResumeName || !newResumeCategory"
        >
          {{ isCreating ? 'Creating...' : 'Create & Edit' }}
        </button>
      </div>
    </div>

    <div v-if="resumesStore.isLoading" class="loading">
      Loading resumes...
    </div>

    <div v-else-if="resumesStore.resumes.length === 0" class="empty-state">
      <p>No resume templates yet. Create your first one!</p>
    </div>

    <div v-else class="resumes-grid">
      <div 
        v-for="resume in resumesStore.resumes" 
        :key="resume.id"
        class="resume-card"
        @click="navigateToResume(resume.id)"
      >
        <div class="card-header">
          <h3>{{ resume.name }}</h3>
          <span class="category">{{ resume.category }}</span>
        </div>
        <p class="date">{{ new Date(resume.created_at).toLocaleDateString() }}</p>
        <p class="id">ID: {{ resume.id }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.resumes-container {
  padding: 40px;
  max-width: 1200px;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.header h2 {
  margin: 0;
  font-size: 2rem;
  color: #ededed;
}

.btn-add {
  background-color: #00e599;
  color: #000;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: 0.2s;
}

.btn-add:hover {
  background-color: #00c785;
}

.error-banner {
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 20px;
  color: #ff5555;
}

.form-card {
  background-color: #0a0a0a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 30px;
  margin-bottom: 30px;
}

.form-card h3 {
  margin-top: 0;
  color: #ededed;
}

.form-group {
  margin-bottom: 16px;
  display: flex;
  flex-direction: column;
}

.form-group label {
  color: #00e599;
  font-weight: 600;
  margin-bottom: 8px;
  font-size: 0.9rem;
}

.form-input {
  background-color: #030303;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 12px;
  color: #ededed;
  font-size: 1rem;
  outline: none;
  transition: 0.2s;
}

.form-input:focus {
  border-color: #00e599;
  box-shadow: 0 0 0 1px rgba(0, 229, 153, 0.3);
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 20px;
}

.btn-cancel, .btn-save {
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: 0.2s;
}

.btn-cancel {
  background: rgba(255, 255, 255, 0.1);
  color: #ededed;
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-save {
  background-color: #00e599;
  color: #000;
}

.btn-save:hover:not(:disabled) {
  background-color: #00c785;
}

.btn-save:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading {
  text-align: center;
  color: #888;
  padding: 40px;
}

.empty-state {
  text-align: center;
  color: #888;
  padding: 60px 40px;
  font-size: 1.1rem;
}

.resumes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.resume-card {
  background-color: #0a0a0a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 24px;
  cursor: pointer;
  transition: all 0.2s;
}

.resume-card:hover {
  border-color: #00e599;
  box-shadow: 0 0 16px rgba(0, 229, 153, 0.2);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 12px;
}

.card-header h3 {
  margin: 0;
  color: #ededed;
  font-size: 1.2rem;
}

.category {
  background: rgba(0, 229, 153, 0.1);
  color: #00e599;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.date {
  color: #888;
  font-size: 0.9rem;
  margin: 8px 0 0 0;
}

.id {
  color: #666;
  font-size: 0.8rem;
  margin: 4px 0 0 0;
  font-family: monospace;
}
</style>
