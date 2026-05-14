<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useResumesStore } from '../store/resumes';
import { Plus, Tag, Calendar, Hash, FileText, X, Info, Save, RotateCw } from '@lucide/vue';
import { Motion, AnimatePresence } from 'motion-v';

const router = useRouter();
const resumesStore = useResumesStore();

// Tooltip State
const activeTooltip = ref<string | null>(null);

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
    const resumeId = await resumesStore.createNewResume(
      newResumeName.value,
      newResumeCategory.value,
      ''
    );
    showNewResumeForm.value = false;
    newResumeName.value = '';
    newResumeCategory.value = '';
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
    <header class="page-header">
      <div class="title-group">
        <h1>Resume Templates</h1>
        <p class="subtitle">Your blueprint collection for high-performance CVs.</p>
      </div>
      <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'new-template'" @mouseleave="activeTooltip = null">
        <button class="btn-primary" @click="toggleNewForm">
          <Plus :size="18" />
        </button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'new-template'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message"
          >
            New Template
          </Motion>
        </AnimatePresence>
      </div>
    </header>

    <div v-if="resumesStore.error" class="error-banner">
      {{ resumesStore.error }}
    </div>

    <transition name="slide-down">
      <div v-if="showNewResumeForm" class="form-card">
        <div class="form-header">
          <h3>Create New Template</h3>
          <button class="close-btn" @click="toggleNewForm"><X :size="18" /></button>
        </div>
        
        <div class="form-grid">
          <div class="form-group">
            <label>Template Name</label>
            <input 
              v-model="newResumeName" 
              type="text" 
              placeholder="e.g., Senior Full-Stack Base"
              class="form-input"
            />
          </div>
          <div class="form-group">
            <label>Category</label>
            <input 
              v-model="newResumeCategory" 
              type="text" 
              placeholder="e.g., Engineering"
              class="form-input"
            />
          </div>
        </div>

        <div class="form-actions">
          <div class="btn-tooltip-wrapper" @mouseenter="activeTooltip = 'initialize-template'" @mouseleave="activeTooltip = null">
            <button class="btn-save" @click="handleCreateResume" :disabled="isCreating || !newResumeName || !newResumeCategory">
              <RotateCw v-if="isCreating" :size="16" class="spinner" />
              <Save v-else :size="16" />
            </button>
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === 'initialize-template'"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                {{ isCreating ? 'Initializing...' : 'Initialize Template' }}
              </Motion>
            </AnimatePresence>
          </div>
        </div>
      </div>
    </transition>

    <div v-if="resumesStore.isLoading" class="loading-state">
      Accessing blueprint vault...
    </div>

    <div v-else-if="resumesStore.resumes.length === 0" class="empty-state">
      <FileText :size="48" class="empty-icon" />
      <h3>No templates found</h3>
      <p>Create your first base resume to start tailoring.</p>
      <div class="btn-tooltip-wrapper" style="margin: 0 auto;" @mouseenter="activeTooltip = 'init-first'" @mouseleave="activeTooltip = null">
        <button class="btn-primary" @click="toggleNewForm">
          <Plus :size="18" />
        </button>
        <AnimatePresence>
          <Motion
            v-if="activeTooltip === 'init-first'"
            :initial="{ opacity: 0, y: 5, scale: 0.9 }"
            :animate="{ opacity: 1, y: 0, scale: 1 }"
            :exit="{ opacity: 0, y: 5, scale: 0.9 }"
            :transition="{ duration: 0.15 }"
            class="flying-message"
          >
            Initialize First Template
          </Motion>
        </AnimatePresence>
      </div>
    </div>

    <div v-else class="resumes-grid">
      <div 
        v-for="resume in resumesStore.resumes" 
        :key="resume.id"
        class="resume-card"
        @click="navigateToResume(resume.id)"
        @mouseenter="activeTooltip = resume.id"
        @mouseleave="activeTooltip = null"
      >
        <div class="resume-card-top">
          <div class="icon-box">
            <FileText :size="24" />
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === resume.id"
                :initial="{ opacity: 0, y: 5, scale: 0.9 }"
                :animate="{ opacity: 1, y: 0, scale: 1 }"
                :exit="{ opacity: 0, y: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message"
              >
                Click to Edit
              </Motion>
            </AnimatePresence>
          </div>
          <div class="category-badge">
            <Tag :size="12" /> {{ resume.category }}
          </div>
        </div>

        <h3 class="resume-name">{{ resume.name }}</h3>
        
        <div class="resume-meta">
          <div class="meta-item">
            <Calendar :size="14" />
            <span>{{ new Date(resume.created_at).toLocaleDateString() }}</span>
          </div>
          <div class="meta-item id-meta">
            <Hash :size="14" />
            <span>{{ resume.id }}</span>
          </div>
        </div>

        <div class="card-footer">
          <span class="edit-link">Edit Template &rarr;</span>
        </div>
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

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 40px;
}

.page-header h1 { font-size: 2.2rem; margin: 0; color: var(--ink); }
.subtitle { color: var(--muted); margin: 8px 0 0; }

.btn-primary {
  background: var(--accent);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-primary:hover { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(11, 123, 107, 0.2); }

.btn-tooltip-wrapper {
  position: relative;
  display: flex;
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
  z-index: 100;
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

.error-banner {
  background: rgba(248, 51, 73, 0.1);
  border: 1px solid rgba(248, 51, 73, 0.2);
  border-radius: 12px;
  padding: 12px 16px;
  margin-bottom: 24px;
  color: #f85149;
  font-size: 0.9rem;
}

.form-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 32px;
  margin-bottom: 40px;
  box-shadow: var(--shadow);
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.form-header h3 { margin: 0; font-size: 1.25rem; color: var(--ink); }

.close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  transition: 0.2s;
}

.close-btn:hover { color: var(--ink); }

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  margin-bottom: 32px;
}

.form-group { display: flex; flex-direction: column; gap: 8px; }

.form-group label {
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  color: var(--accent);
  letter-spacing: 0.05em;
}

.form-input {
  width: 100%;
  padding: 12px 16px;
  background: var(--surface-soft);
  border: 1px solid var(--line);
  border-radius: 10px;
  color: var(--ink);
  font-size: 1rem;
  outline: none;
  transition: 0.2s;
}

.form-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.form-actions { display: flex; justify-content: flex-end; }
.btn-save {
  background: var(--accent);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.resumes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
}

.resume-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 24px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow);
  overflow: hidden;
  min-width: 0;
}

.resume-card:hover {
  transform: translateY(-4px);
  border-color: var(--accent);
  box-shadow: 0 8px 24px rgba(0,0,0,0.06);
}

.resume-card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.icon-box {
  width: 40px;
  height: 40px;
  background: var(--surface-soft);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  position: relative;
}

.category-badge {
  padding: 4px 10px;
  background: var(--accent-soft);
  color: var(--accent);
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 6px;
}

.resume-name {
  font-size: 1.25rem;
  margin: 0 0 16px 0;
  color: var(--ink);
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resume-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: auto;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--muted);
  font-size: 0.8rem;
}

.id-meta { font-family: monospace; opacity: 0.7; }

.card-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--line);
}

.edit-link {
  font-size: 0.85rem;
  font-weight: 700;
  color: var(--muted);
  transition: 0.2s;
}

.resume-card:hover .edit-link { color: var(--accent); }

.loading-state, .empty-state {
  text-align: center;
  padding: 80px 0;
  color: var(--muted);
}

.empty-icon { margin-bottom: 16px; opacity: 0.3; color: var(--muted); }
.empty-state h3 { color: var(--ink); margin-bottom: 8px; }
.empty-state p { margin-bottom: 24px; }
.empty-state .btn-primary { margin: 0 auto; }

/* Transitions */
.slide-down-enter-active, .slide-down-leave-active { transition: all 0.3s ease-out; }
.slide-down-enter-from, .slide-down-leave-to { opacity: 0; transform: translateY(-20px); }

@media (max-width: 768px) {
  .resumes-container { padding: 20px; }
  .page-header { flex-direction: column; gap: 20px; }
  .form-grid { grid-template-columns: 1fr; }
}
</style>
