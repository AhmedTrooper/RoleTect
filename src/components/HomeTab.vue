<script setup lang="ts">
import { ref } from 'vue';

// Mock data: Later we will fetch this from Rust/SQLite using Tauri invoke()
const savedJobs = ref([
  { id: 1, title: 'Senior Rust Developer', company: 'TechCorp', date: '2026-05-13' },
  { id: 2, title: 'Frontend Engineer', company: 'VueMastery', date: '2026-05-12' },
]);
</script>

<template>
  <div class="home-tab">
    <h1>Saved Jobs</h1>
    <p class="subtitle">Select a job to view its parsed JSON or generate a resume.</p>

    <div class="job-grid">
      <div v-for="job in savedJobs" :key="job.id" class="job-card">
        <h3>{{ job.title }}</h3>
        <p class="company">🏢 {{ job.company }}</p>
        <p class="date">📅 {{ job.date }}</p>
        <button class="action-btn">View Details</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
h1 { margin-top: 0; }
.subtitle { color: #a6adc8; margin-bottom: 20px; }

.job-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 20px;
}

.job-card {
  background-color: #313244;
  padding: 20px;
  border-radius: 10px;
  border: 1px solid #45475a;
}

.job-card h3 { margin: 0 0 10px 0; color: #f38ba8; }
.company, .date { margin: 5px 0; font-size: 0.9rem; }

.action-btn {
  margin-top: 15px;
  width: 100%;
  padding: 8px;
  background-color: #89b4fa;
  border: none;
  border-radius: 5px;
  color: #11111b;
  font-weight: bold;
  cursor: pointer;
}
.action-btn:hover { background-color: #b4befe; }
</style>