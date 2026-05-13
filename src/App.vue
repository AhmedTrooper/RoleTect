<script setup lang="ts">
const tabs = [
  { path: '/', label: 'Saved Jobs', icon: '🏠' },
  { path: '/resumes', label: 'Resume Templates', icon: '📄' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
];
</script>


<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo-container">
        <div class="logo-icon"></div>
      </div>
      
      <nav class="nav-menu">
        <router-link 
          v-for="tab in tabs" 
          :key="tab.path"
          :to="tab.path"
          class="nav-button"
          active-class="active"
        >
          <span class="icon">{{ tab.icon }}</span> 
          <span class="tab-label">{{ tab.label }}</span>
        </router-link>
      </nav>
    </aside>

    <main class="content-area">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
/* PREMIUM LIGHT "EDITORIAL" VIBE */
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: #fcfcf9; /* Warm 'Paper' White */
  color: #1a1a1a; /* Soft Charcoal instead of pure black */
  font-family: 'Inter', -apple-system, sans-serif;
  overflow: hidden;
}

.sidebar {
  width: 72px;
  background: #f4f4f2; /* Subtle warm gray */
  border-right: 1px solid #e5e5e0;
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 10;
}

.logo-container {
  padding: 30px 0;
}

.logo-icon {
  width: 32px;
  height: 32px;
  background: #1a1a1a;
  border-radius: 8px;
  /* No glow, just a sharp, clean shape */
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 10px 0;
}

.nav-button {
  width: 44px;
  height: 44px;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 8px;
  color: #71716e;
  transition: all 0.15s ease;
  text-decoration: none;
}

.nav-button:hover {
  background: #eaea00;
  color: #1a1a1a;
}

.nav-button.active {
  background: #ffffff;
  color: #1a1a1a;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05), 0 0 0 1px rgba(0,0,0,0.05);
}

.icon { font-size: 1.2rem; }

.tab-label {
  position: absolute;
  left: 60px;
  background: #1a1a1a;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease;
}

.nav-button:hover .tab-label { opacity: 1; }

.content-area {
  flex-grow: 1;
  overflow-y: auto;
}
</style>