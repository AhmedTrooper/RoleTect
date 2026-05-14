<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useResumesStore, type ResumeDetail } from '../store/resumes';
import { Motion, AnimatePresence } from 'motion-v';
import { 
  ArrowLeft, 
  Edit, 
  Trash2, 
  X, 
  Save, 
  RotateCw 
} from '@lucide/vue';

const router = useRouter();
const resumesStore = useResumesStore();

const props = defineProps<{ id: string }>();

// Tooltip State
const activeTooltip = ref<string | null>(null);

const isLoading = ref(true);
const isEditing = ref(false);
const isSaving = ref(false);
const isDeleting = ref(false);
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
    editedLatex.value = data.latex_content || '';
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

const handleDelete = async () => {
  if (!resume.value) return;
  const confirmed = window.confirm('Delete this resume template? This cannot be undone.');
  if (!confirmed) return;

  isDeleting.value = true;
  error.value = null;

  try {
    await resumesStore.deleteResume(resume.value.id);
    router.push('/resumes');
  } catch (err: any) {
    error.value = err.toString();
  } finally {
    isDeleting.value = false;
  }
};

const hasLatexContent = () => {
  const content = resume.value?.latex_content || '';
  return content.trim().length > 0;
};
</script>

<template>
  <div class="detail-container" v-if="!isLoading">
    <header class="detail-header">
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
            Back to Templates
          </Motion>
        </AnimatePresence>
      </div>
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
        <div v-if="!isEditing" class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'edit-template'" @mouseleave="activeTooltip = null">
          <button class="btn-edit" @click="toggleEditMode"><Edit :size="16" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'edit-template'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message"
            >
              Edit Template
            </Motion>
          </AnimatePresence>
        </div>
        <div v-if="!isEditing" class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'delete-template'" @mouseleave="activeTooltip = null">
          <button class="btn-delete" @click="handleDelete" :disabled="isDeleting">
            <Trash2 v-if="!isDeleting" :size="16" />
            <RotateCw v-else :size="16" class="spinner" />
          </button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'delete-template'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message delete-tooltip"
            >
              Delete Template
            </Motion>
          </AnimatePresence>
        </div>
        
        <div v-else class="edit-actions">
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'cancel-edit'" @mouseleave="activeTooltip = null">
            <button class="btn-cancel" @click="toggleEditMode"><X :size="16" /></button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'cancel-edit'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Cancel Changes
              </Motion>
            </AnimatePresence>
          </div>
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save-template'" @mouseleave="activeTooltip = null">
            <button class="btn-save" @click="handleSave" :disabled="isSaving">
              <Save v-if="!isSaving" :size="16" />
              <RotateCw v-else :size="16" class="spinner" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'save-template'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Save Template
              </Motion>
            </AnimatePresence>
          </div>
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
        <div v-if="isEditing" class="editor-actions">
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'cancel-edit-editor'" @mouseleave="activeTooltip = null">
            <button class="btn-cancel" @click="toggleEditMode"><X :size="16" /></button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'cancel-edit-editor'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Cancel Changes
              </Motion>
            </AnimatePresence>
          </div>
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save-template-editor'" @mouseleave="activeTooltip = null">
            <button class="btn-save" @click="handleSave" :disabled="isSaving">
              <Save v-if="!isSaving" :size="16" />
              <RotateCw v-else :size="16" class="spinner" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'save-template-editor'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Save Template
              </Motion>
            </AnimatePresence>
          </div>
        </div>
      </div>
      
      <textarea 
        v-if="isEditing"
        v-model="editedLatex" 
        class="latex-editor edit-mode"
        placeholder="Enter your LaTeX code here..."
        spellcheck="false"
      ></textarea>
      
      <div v-else-if="!hasLatexContent()" class="empty-latex">
        <p>No LaTeX content yet.</p>
        <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'add-latex'" @mouseleave="activeTooltip = null">
          <button class="btn-edit" @click="toggleEditMode"><Edit :size="16" /></button>
          <AnimatePresence>
            <Motion
              v-if="activeTooltip === 'add-latex'"
              :initial="{ opacity: 0, y: 5, scale: 0.9 }"
              :animate="{ opacity: 1, y: 0, scale: 1 }"
              :exit="{ opacity: 0, y: 5, scale: 0.9 }"
              :transition="{ duration: 0.15 }"
              class="flying-message"
            >
              Add LaTeX
            </Motion>
          </AnimatePresence>
        </div>
      </div>
      
      <pre v-else class="latex-preview">{{ resume?.latex_content }}</pre>
    </div>

    <div class="footer-info">
      <p>💡 Tip: This LaTeX template will be used when tailoring resumes for job applications.</p>
    </div>

    <div v-if="isEditing" class="edit-bar">
      <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'cancel-edit-footer'" @mouseleave="activeTooltip = null">
        <button class="btn-cancel" @click="toggleEditMode"><X :size="16" /></button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'cancel-edit-footer'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message"
          >
            Cancel Changes
          </Motion>
        </AnimatePresence>
      </div>
      <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'save-template-footer'" @mouseleave="activeTooltip = null">
        <button class="btn-save" @click="handleSave" :disabled="isSaving">
          <Save v-if="!isSaving" :size="16" />
          <RotateCw v-else :size="16" class="spinner" />
        </button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'save-template-footer'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message"
          >
            Save Template
          </Motion>
        </AnimatePresence>
      </div>
    </div>
  </div>

  <div v-else class="loading">
    Loading resume...
  </div>
</template>

<style scoped>
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
.delete-tooltip { background: var(--warning); }
.delete-tooltip::after { border-bottom-color: var(--warning); }

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.detail-container {
  padding: 24px 20px 40px;
  max-width: 1400px;
  margin: 0 auto;
}

.detail-header {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 16px;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--line);
}

.back-btn {
  background: var(--surface);
  border: 1px solid var(--line);
  color: var(--muted);
  padding: 8px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: 0.2s;
  white-space: nowrap;
}

.back-btn:hover { color: var(--ink); border-color: var(--accent); }

.title-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.title-section h1 {
  margin: 0;
  font-size: 1.6rem;
  color: var(--ink);
}

.category {
  background: rgba(11, 123, 107, 0.12);
  color: var(--accent);
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 700;
  width: fit-content;
}

.edit-input {
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 10px 12px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
}

.title-input {
  font-size: 1.4rem;
  font-weight: 700;
}

.category-input { width: fit-content; }

.edit-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.2);
}

.actions-top {
  display: flex;
  gap: 10px;
  align-items: center;
}

.btn-edit {
  background-color: var(--accent);
  color: #fff;
  border: none;
  padding: 10px 16px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-edit:hover { background-color: #0a6b5e; }

.edit-actions {
  display: flex;
  gap: 10px;
}

.btn-cancel, .btn-save {
  border: none;
  padding: 10px 16px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-delete {
  background: #fff0ef;
  color: var(--warning);
  border: 1px solid rgba(180, 35, 24, 0.2);
  padding: 10px 16px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
}

.btn-delete:hover:not(:disabled) {
  background: #ffe5e2;
}

.btn-delete:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-cancel {
  background: var(--surface-soft);
  color: var(--muted);
}

.btn-cancel:hover { color: var(--ink); }

.btn-save {
  background-color: var(--accent);
  color: #fff;
}

.btn-save:hover:not(:disabled) { background-color: #0a6b5e; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }

.error-banner {
  background: rgba(180, 35, 24, 0.1);
  border: 1px solid rgba(180, 35, 24, 0.2);
  border-radius: 10px;
  padding: 12px 16px;
  margin-bottom: 16px;
  color: var(--warning);
}

.meta-info {
  display: grid;
  gap: 12px;
  margin-bottom: 24px;
  padding: 16px;
  background: var(--surface);
  border-radius: 12px;
  border: 1px solid var(--line);
  font-size: 0.9rem;
  box-shadow: var(--shadow);
}

.meta-item { color: var(--muted); }
.meta-item strong { color: var(--accent); }

.editor-section { margin-bottom: 24px; }

.editor-header {
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.editor-header h2 {
  margin: 0;
  color: var(--ink);
  font-size: 1.1rem;
}

.editor-actions { display: flex; gap: 10px; }

.latex-editor {
  width: 100%;
  min-height: 340px;
  max-height: 520px;
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 16px;
  color: var(--ink);
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.9rem;
  line-height: 1.6;
  outline: none;
  resize: vertical;
  overflow: auto;
}

.latex-editor:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(11, 123, 107, 0.12);
}

.latex-preview {
  width: 100%;
  min-height: 340px;
  max-height: 520px;
  background-color: var(--surface);
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 16px;
  color: var(--ink);
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.9rem;
  line-height: 1.6;
  overflow: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}

.empty-latex {
  width: 100%;
  min-height: 220px;
  background-color: var(--surface);
  border: 1px dashed var(--line);
  border-radius: 12px;
  padding: 20px;
  color: var(--muted);
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
}

.footer-info {
  padding: 14px 16px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 12px;
  color: var(--muted);
  font-size: 0.9rem;
}

.edit-bar {
  position: sticky;
  bottom: 12px;
  display: flex;
  gap: 10px;
  justify-content: flex-end;
  margin-top: 20px;
  padding: 12px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid var(--line);
  box-shadow: var(--shadow);
}

.loading {
  text-align: center;
  color: var(--muted);
  padding: 40px;
}

@media (min-width: 960px) {
  .detail-container { padding: 40px 32px 60px; }
  .detail-header { grid-template-columns: auto 1fr auto; align-items: center; }
  .meta-info { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  .latex-editor, .latex-preview { min-height: 520px; }
  .edit-bar { position: static; margin-top: 24px; background: transparent; border: none; box-shadow: none; padding: 0; }
}
</style>
