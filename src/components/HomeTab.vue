<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useSettingsStore } from '../store/settings';

const router = useRouter();
const route = useRoute();
const settingsStore = useSettingsStore();
const settingsError = ref('');
const isLoadingSettings = ref(false);

// Mock data (we will replace this with Pinia store data later)
const savedJobs = ref([
  { id: 'nano_1', title: 'Senior Rust Developer', company: 'TechCorp', date: '2026-05-13' },
  { id: 'nano_2', title: 'Frontend Engineer', company: 'VueMastery', date: '2026-05-12' },
]);

const navigateToJob = (id: string) => {
  router.push(`/job/${id}`);
};

const refreshSettings = async () => {
  isLoadingSettings.value = true;
  settingsError.value = '';
  try {
    await settingsStore.loadSettings();
    // Re-check key status for the now-active provider.
    // loadSettings() updates selectedAiProvider from SQLite, but
    // hasSecureKey may still reflect the old provider — re-sync it.
    await settingsStore.loadProviderKeyStatus(settingsStore.selectedAiProvider);
  } catch (err: any) {
    settingsError.value = err?.message || 'Failed to load settings.';
  } finally {
    isLoadingSettings.value = false;
  }
};

onMounted(async () => {
  await refreshSettings();
});

// Refresh every time the user navigates back to the home route.
// No path guard needed — this watch only triggers on path changes
// and onMounted already handles the initial load.
watch(
  () => route.fullPath,
  async () => {
    await refreshSettings();
  }
);

const activeProvider = computed(() => settingsStore.selectedAiProvider);
const activeModel = computed(() => settingsStore.selectedAiModel);
const providerLabel = computed(() => {
  const map: Record<string, string> = {
    openai: 'OpenAI',
    gemini: 'Google Gemini',
    groq: 'Groq',
  };
  return map[activeProvider.value] || activeProvider.value || 'Not set';
});

const modelLabel = computed(() => activeModel.value || 'Not set');
</script>
<template>
  <div class="home-container">
    <div class="hero-section">
      <div class="status-pill">
        <span class="pulse"></span> 
        Engine Status: Ready
      </div>
      
      <h1 class="main-title">
        Crafting resumes with <br />
        <span class="serif-italic">Intentional Design.</span>
      </h1>
      
      <p class="description">
        CVSynth uses refined AI models to distill job descriptions into precise 
        data points, helping you build a targeted professional narrative.
      </p>

      <div class="actions">
        <button class="btn btn-dark" @click="$router.push('/parse')">New Application</button>
        <button class="btn btn-outline" @click="$router.push('/resumes')">Manage Templates</button>
      </div>
    </div>

    <div class="bento-grid">
      <div class="bento-item large">
        <h3>Recent Applications</h3>
        <div v-if="savedJobs.length === 0" class="empty-block">
          No jobs yet. Paste a job description to start.
        </div>
        <div v-else class="job-list-minimal">
          <button
            v-for="job in savedJobs"
            :key="job.id"
            class="job-item"
            @click="navigateToJob(job.id)"
            type="button"
          >
            <span class="job-dot"></span>
            <div class="job-info">
              <span class="j-title">{{ job.title }}</span>
              <span class="j-company">{{ job.company }}</span>
            </div>
            <span class="j-date">{{ job.date }}</span>
          </button>
        </div>
      </div>
      <div class="bento-item small">
        <h3>Model Config</h3>
        <p v-if="settingsError" class="error-inline">{{ settingsError }}</p>
        <p v-else class="model-line">Provider: {{ providerLabel }}</p>
        <p v-if="!settingsError" class="model-line">Model: {{ modelLabel }}</p>
        <div class="model-actions">
          <button class="link-btn" @click="$router.push('/settings')" type="button">Edit Settings</button>
          <button class="link-btn" @click="refreshSettings" type="button" :disabled="isLoadingSettings">
            {{ isLoadingSettings ? 'Refreshing...' : 'Refresh' }}
          </button>
        </div>
      </div>
      <div class="bento-item small">
        <h3>Resume Templates</h3>
        <p class="model-line">Manage your base templates for tailoring.</p>
        <button class="link-btn" @click="$router.push('/resumes')" type="button">Open Templates</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 32px 20px 60px;
}

.hero-section {
  text-align: left;
  margin-bottom: 48px;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: var(--surface-soft);
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--muted);
  margin-bottom: 18px;
  border: 1px solid var(--line);
}

.pulse {
  width: 6px;
  height: 6px;
  background: #10b981;
  border-radius: 50%;
  animation: pulse-ring 2s infinite;
}

@keyframes pulse-ring {
  0% { transform: scale(0.9); opacity: 0.7; }
  50% { transform: scale(1.1); opacity: 1; }
  100% { transform: scale(0.9); opacity: 0.7; }
}

.main-title {
  font-size: 2.4rem;
  font-weight: 800;
  color: var(--ink);
  letter-spacing: -0.03em;
  line-height: 1.1;
  margin-bottom: 18px;
}

.serif-italic {
  font-family: 'Merriweather', serif;
  font-style: italic;
  font-weight: 400;
  color: var(--muted);
}

.description {
  font-size: 1rem;
  color: var(--muted);
  max-width: 540px;
  margin: 0 0 28px;
  line-height: 1.6;
}

.actions { display: flex; gap: 12px; flex-wrap: wrap; }

.btn {
  padding: 12px 18px;
  border-radius: 12px;
  font-size: 0.95rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-dark {
  background: var(--accent);
  color: #fff;
  border: none;
}

.btn-dark:hover { background: #0a6b5e; transform: translateY(-1px); }

.btn-outline {
  background: var(--surface);
  color: var(--ink);
  border: 1px solid var(--line);
}

.btn-outline:hover { background: var(--surface-soft); }

/* Bento Grid */
.bento-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
}

.bento-item {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 20px;
  box-shadow: var(--shadow);
}

.bento-item h3 {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
  margin: 0 0 16px 0;
}

.job-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
  border-bottom: 1px solid var(--line);
  background: transparent;
  border-top: none;
  border-left: none;
  border-right: none;
  width: 100%;
  text-align: left;
  cursor: pointer;
}

.j-title { font-weight: 700; display: block; color: var(--ink); }
.j-company { font-size: 0.85rem; color: var(--muted); }
.j-date { margin-left: auto; font-size: 0.8rem; color: #8c857a; }

.job-item:hover {
  background: var(--surface-soft);
  border-radius: 8px;
  padding: 10px 8px;
}

.empty-block {
  color: var(--muted);
  font-size: 0.95rem;
  padding: 8px 0 4px;
}

.model-line {
  color: var(--ink);
  font-weight: 600;
  margin: 8px 0;
}

.model-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.error-inline {
  color: var(--warning);
  font-weight: 600;
  margin: 8px 0;
}

.link-btn {
  margin-top: 8px;
  background: var(--surface-soft);
  color: var(--ink);
  border: 1px solid var(--line);
  padding: 8px 12px;
  border-radius: 10px;
  font-weight: 700;
  cursor: pointer;
}

.link-btn:hover {
  background: var(--surface);
}

@media (min-width: 960px) {
  .home-container { padding: 80px 32px 100px; }
  .hero-section { text-align: center; }
  .description { margin: 0 auto 40px; }
  .actions { justify-content: center; }
  .main-title { font-size: 3.1rem; }
  .bento-grid { grid-template-columns: 2fr 1fr; }
}
</style>