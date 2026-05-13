<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useResumesStore, type ResumeDetail } from '../store/resumes';

const router = useRouter();
const resumesStore = useResumesStore();

const props = defineProps<{ id: string }>();

const isLoading = ref(true);
const isEditing = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);

const resume = ref<ResumeDetail | null>(null);
const editedName = ref('');
const editedCategory = ref('');
const editedLatex = ref('');

onMounted(async () => {
  try {
    const data = await resumesStore.getResumeById(props.id);
    resume.value = data;
    editedName.value = data.name;
    editedCategory.value = data.category;
    editedLatex.value = data.latex_content;
  } catch (err: any) {
    error.value = err.toString();
  } finally {
    isLoading.value = false;
  }
});

const goBack = () => {
  router.push('/resumes');
};

const toggleEditMode = () => {
  if (isEditing.value) {
    // Reset to original values if canceling
    if (resume.value) {
      editedName.value = resume.value.name;
      editedCategory.value = resume.value.category;
      editedLatex.value = resume.value.latex_content;
    }
  }
  isEditing.value = !isEditing.value;
};

const handleSave = async () => {
  if (!resume.value || !editedName.value.trim() || !editedCategory.value.trim()) {
    error.value = 'Name and category are required';
    return;
  }

  isSaving.value = true;
  error.value = null;

  try {
    await resumesStore.updateResume(
      resume.value.id,
      editedName.value,
      editedCategory.value,
      editedLatex.value
    );

    // Reload the resume
    const updated = await resumesStore.getResumeById(props.id);
    resume.value = updated;
    isEditing.value = false;
  } catch (err: any) {
    error.value = err.toString();
  } finally {
    isSaving.value = false;
  }
};
</script>

<template>
  <div class="detail-container" v-if="!isLoading">
    <header class="detail-header">
      <button class="back-btn" @click="goBack">← Back to Templates</button>
      <div class="title-section">
        <h1 v-if="!isEditing">{{ resume?.name }}</h1>
        <input 
          v-else 
          v-model="editedName" 
          type="text" 
          class="edit-input title-input"
        />
        <span v-if="!isEditing" class="category">{{ resume?.category }}</span>
        <input 
          v-else 
          v-model="editedCategory" 
          type="text" 
          class="edit-input category-input"
          placeholder="Category"
        />
      </div>
      <div class="actions-top">
        <button v-if="!isEditing" class="btn-edit" @click="toggleEditMode">✏️ Edit</button>
        <div v-else class="edit-actions">
          <button class="btn-cancel" @click="toggleEditMode">Cancel</button>
          <button class="btn-save" @click="handleSave" :disabled="isSaving">
            {{ isSaving ? 'Saving...' : 'Save' }}
          </button>
        </div>
      </div>
    </header>

    <div v-if="error" class="error-banner">
      {{ error }}
    </div>

    <div class="meta-info">
      <span class="meta-item">
        <strong>ID:</strong> {{ resume?.id }}
      </span>
      <span class="meta-item">
        <strong>Created:</strong> {{ new Date(resume?.created_at || '').toLocaleString() }}
      </span>
      <span class="meta-item">
        <strong>Updated:</strong> {{ new Date(resume?.updated_at || '').toLocaleString() }}
      </span>
    </div>

    <div class="editor-section">
      <div class="editor-header">
        <h2>LaTeX Template</h2>
      </div>
      
      <textarea 
        v-if="isEditing"
        v-model="editedLatex" 
        class="latex-editor edit-mode"
        placeholder="Enter your LaTeX code here..."
        spellcheck="false"
      ></textarea>
      
      <pre v-else class="latex-preview">{{ resume?.latex_content }}</pre>
    </div>

    <div class="footer-info">
      <p>💡 Tip: This LaTeX template will be used when tailoring resumes for job applications.</p>
    </div>
  </div>

  <div v-else class="loading">
    Loading resume...
  </div>
</template>

<style scoped>
.detail-container {
  padding: 40px;
  max-width: 1400px;
  margin: 0 auto;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 30px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding-bottom: 20px;
}

.back-btn {
  background: none;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #888;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  transition: 0.2s;
  white-space: nowrap;
}

.back-btn:hover {
  color: #ededed;
  border-color: rgba(255, 255, 255, 0.3);
}

.title-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-grow: 1;
}

.title-section h1 {
  margin: 0;
  font-size: 2rem;
  color: #ededed;
}

.category {
  background: rgba(0, 229, 153, 0.1);
  color: #00e599;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 600;
  width: fit-content;
}

.edit-input {
  background-color: #030303;
  border: 1px solid rgba(0, 229, 153, 0.3);
  border-radius: 6px;
  padding: 12px;
  color: #ededed;
  font-size: 1rem;
  outline: none;
}

.title-input {
  font-size: 1.8rem;
  font-weight: bold;
}

.category-input {
  width: fit-content;
}

.edit-input:focus {
  border-color: #00e599;
  box-shadow: 0 0 0 1px rgba(0, 229, 153, 0.3);
}

.actions-top {
  display: flex;
  gap: 12px;
}

.btn-edit {
  background-color: #89b4fa;
  color: #11111b;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: 0.2s;
}

.btn-edit:hover {
  background-color: #a6c3ff;
}

.edit-actions {
  display: flex;
  gap: 12px;
}

.btn-cancel, .btn-save {
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
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

.error-banner {
  background: rgba(255, 80, 80, 0.1);
  border: 1px solid rgba(255, 80, 80, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 20px;
  color: #ff5555;
}

.meta-info {
  display: flex;
  gap: 24px;
  margin-bottom: 30px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.9rem;
}

.meta-item {
  color: #888;
}

.meta-item strong {
  color: #00e599;
}

.editor-section {
  margin-bottom: 30px;
}

.editor-header {
  margin-bottom: 16px;
}

.editor-header h2 {
  margin: 0;
  color: #ededed;
  font-size: 1.2rem;
}

.latex-editor {
  width: 100%;
  min-height: 500px;
  background-color: #030303;
  border: 1px solid rgba(0, 229, 153, 0.2);
  border-radius: 8px;
  padding: 20px;
  color: #a6adc8;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.9rem;
  line-height: 1.6;
  outline: none;
  resize: vertical;
}

.latex-editor:focus {
  border-color: #00e599;
  box-shadow: 0 0 0 2px rgba(0, 229, 153, 0.1);
}

.latex-editor.edit-mode {
  border-color: #00e599;
}

.latex-preview {
  width: 100%;
  min-height: 500px;
  background-color: #030303;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 20px;
  color: #a6adc8;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.9rem;
  line-height: 1.6;
  overflow: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}

.footer-info {
  padding: 16px;
  background: rgba(0, 229, 153, 0.05);
  border: 1px solid rgba(0, 229, 153, 0.2);
  border-radius: 8px;
  color: #888;
  font-size: 0.9rem;
}

.loading {
  text-align: center;
  color: #888;
  padding: 60px;
}
</style>
