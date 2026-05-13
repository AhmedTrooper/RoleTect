<script setup lang="ts">
import { ref, computed } from 'vue';
import HomeTab from './components/HomeTab.vue';
import SettingsTab from './components/SettingsTab.vue';
import JobDetailView from './components/JobDetailView.vue';

const tabs = [
  { id: 'home', label: 'Saved Jobs', icon: '🏠' },
  { id: 'settings', label: 'Settings', icon: '⚙️' },
];

const activeTabId = ref('home');
const selectedJobId = ref<number | null>(null);

const openJobDetails = (id: number) => {
  selectedJobId.value = id;
  activeTabId.value = 'job-detail';
};

const activeComponent = computed(() => {
  if (activeTabId.value === 'job-detail') return JobDetailView;
  switch (activeTabId.value) {
    case 'home': return HomeTab;
    case 'settings': return SettingsTab;
    default: return HomeTab;
  }
});
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo-container">
        <div class="logo-icon"></div>
      </div>
      
      <nav class="nav-menu">
        <button 
          v-for="tab in tabs" :key="tab.id"
          @click="activeTabId = tab.id; selectedJobId = null" 
          :class="['nav-button', { active: activeTabId === tab.id }]"
        >
          <span class="icon">{{ tab.icon }}</span> 
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </nav>
    </aside>

    <main class="content-area">
      <div class="glass-container">
        <component 
          :is="activeComponent" 
          :jobId="selectedJobId" 
          @open-job="openJobDetails"
          @go-back="activeTabId = 'home'"
        />
      </div>
    </main>
  </div>
</template>

<style scoped>
/* COLOR PALETTE: Deep Marine & Teal/Cyan */

.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: #060e1a; 
  background-image: radial-gradient(circle at top right, #0a1f3d, #060e1a);
  color: #e2e8f0;
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  overflow: hidden;
}

/* --- SLIM SIDEBAR --- */
.sidebar {
  width: 80px; /* Shrunk from 260px */
  background: rgba(13, 22, 43, 0.6);
  backdrop-filter: blur(12px);
  border-right: 1px solid rgba(0, 245, 212, 0.1);
  display: flex;
  flex-direction: column;
  align-items: center; /* Center everything horizontally */
  box-shadow: 4px 0 15px rgba(0, 0, 0, 0.2);
  z-index: 10;
}

.logo-container {
  padding: 25px 0;
  display: flex;
  justify-content: center;
  width: 100%;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.logo-icon {
  width: 28px;
  height: 28px;
  background: linear-gradient(135deg, #00F5D4, #0E79B2);
  border-radius: 8px;
  box-shadow: 0 0 12px rgba(0, 245, 212, 0.5);
}

/* --- NAVIGATION --- */
.nav-menu {
  display: flex;
  flex-direction: column;
  padding: 20px 0;
  gap: 15px;
  width: 100%;
  align-items: center;
}

.nav-button {
  position: relative; /* Crucial for absolute tooltip positioning */
  background: transparent;
  border: 1px solid transparent;
  color: #94a3b8;
  padding: 0;
  width: 48px;
  height: 48px;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-button:hover {
  background: rgba(14, 121, 178, 0.1);
}

.nav-button.active {
  background: linear-gradient(135deg, rgba(0, 245, 212, 0.15), rgba(14, 121, 178, 0.1));
  border: 1px solid rgba(0, 245, 212, 0.4);
  box-shadow: inset 0 0 10px rgba(0, 245, 212, 0.1);
}

.nav-button .icon {
  font-size: 1.4rem;
  filter: grayscale(100%) brightness(150%);
  transition: all 0.3s ease;
}

.nav-button.active .icon {
  filter: none;
  transform: scale(1.1); /* Slight pop effect when active */
}

/* --- TOOLTIP LOGIC --- */
.tab-label {
  position: absolute;
  left: 100%; /* Push it completely to the right of the button */
  margin-left: 15px; /* Add a gap between button and tooltip */
  background: rgba(15, 23, 42, 0.95);
  border: 1px solid rgba(0, 245, 212, 0.3);
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 600;
  color: #00F5D4;
  white-space: nowrap;
  
  /* Hidden by default */
  opacity: 0;
  pointer-events: none;
  transform: translateX(-10px);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 100;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.4);
}

/* Tooltip Arrow */
.tab-label::before {
  content: '';
  position: absolute;
  top: 50%;
  right: 100%;
  margin-top: -6px;
  border-width: 6px;
  border-style: solid;
  border-color: transparent rgba(0, 245, 212, 0.3) transparent transparent;
}

/* Show tooltip on hover */
.nav-button:hover .tab-label {
  opacity: 1;
  transform: translateX(0);
}

/* --- MAIN CONTENT AREA --- */
.content-area {
  flex-grow: 1;
  padding: 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.glass-container {
  flex-grow: 1;
  background: rgba(15, 23, 42, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.03);
  border-radius: 16px;
  padding: 30px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
}

::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: rgba(0, 0, 0, 0.1); }
::-webkit-scrollbar-thumb { background: rgba(14, 121, 178, 0.5); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: rgba(0, 245, 212, 0.5); }
</style>

<style>
/* UNSCOPED global styles to reset the browser defaults */
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden; 
}
* { box-sizing: border-box; }
</style>