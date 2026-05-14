<script setup lang="ts">
import { ref } from 'vue';
import { Motion, AnimatePresence } from 'motion-v';
import { open } from '@tauri-apps/plugin-shell';
import { 
  Home, 
  Briefcase, 
  FileText, 
  Settings, 
  Code, 
  Video 
} from '@lucide/vue';

const tabs = [
  { path: '/', label: 'Home', icon: Home },
  { path: '/jobs', label: 'Jobs', icon: Briefcase },
  { path: '/resumes', label: 'Templates', icon: FileText },
  { path: '/settings', label: 'Settings', icon: Settings },
];

const externalLinks = [
  { url: 'https://github.com/AhmedTrooper/CVSynth', label: 'Source', icon: Code },
  { url: 'https://www.youtube.com/@AhmedTrooper', label: 'YouTube', icon: Video },
];

const activeTooltip = ref<string | null>(null);

const handleExternalClick = (url: string) => {
  open(url).catch(err => console.error('Failed to open URL:', err));
};
</script>

<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="logo-section">
        <div class="logo-dot"></div>
      </div>
      
      <nav class="nav-menu">
        <router-link 
          v-for="tab in tabs" 
          :key="tab.path"
          :to="tab.path"
          class="nav-item"
          active-class="active"
          @mouseenter="activeTooltip = tab.label"
          @mouseleave="activeTooltip = null"
        >
          <div class="icon-wrapper">
            <component :is="tab.icon" :size="20" stroke-width="2" />
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === tab.label"
                :initial="{ opacity: 0, x: 5, scale: 0.9 }"
                :animate="{ opacity: 1, x: 12, scale: 1 }"
                :exit="{ opacity: 0, x: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message sidebar-tooltip"
              >
                {{ tab.label }}
              </Motion>
            </AnimatePresence>
          </div>
        </router-link>

        <div class="nav-divider"></div>

        <button 
          v-for="link in externalLinks" 
          :key="link.url"
          @click="handleExternalClick(link.url)"
          class="nav-item external"
          @mouseenter="activeTooltip = link.label"
          @mouseleave="activeTooltip = null"
        >
          <div class="icon-wrapper">
            <component :is="link.icon" :size="20" stroke-width="2" />
            <AnimatePresence>
              <Motion
                v-if="activeTooltip === link.label"
                :initial="{ opacity: 0, x: 5, scale: 0.9 }"
                :animate="{ opacity: 1, x: 12, scale: 1 }"
                :exit="{ opacity: 0, x: 5, scale: 0.9 }"
                :transition="{ duration: 0.15 }"
                class="flying-message sidebar-tooltip"
              >
                {{ link.label }}
              </Motion>
            </AnimatePresence>
          </div>
        </button>
      </nav>
    </aside>

    <main class="content-area">
      <router-view v-slot="{ Component, route }">
        <transition mode="out-in">
          <Motion
            :key="route.path"
            :initial="{ opacity: 0, y: 5 }"
            :animate="{ opacity: 1, y: 0 }"
            :transition="{ duration: 0.15, ease: 'easeOut' }"
            class="route-wrapper"
          >
            <component :is="Component" />
          </Motion>
        </transition>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  background: var(--bg);
  color: var(--ink);
  overflow: hidden;
}

.sidebar {
  order: 2;
  background: var(--bg-accent);
  border-top: 1px solid var(--line);
  z-index: 100;
  display: flex;
}

.logo-section {
  display: none;
}

.nav-menu {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  width: 100%;
  padding: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 0;
  color: var(--muted);
  text-decoration: none;
  transition: 0.15s;
  background: none;
  border: none;
  cursor: pointer;
  width: 100%;
}

.nav-divider {
  display: none;
}

.nav-item.external {
  opacity: 0.8;
}

.nav-item.external:hover {
  opacity: 1;
  color: var(--accent);
}

.nav-item:hover {
  color: var(--ink);
}

.nav-item.active {
  color: var(--accent);
}

.icon-wrapper {
  font-size: 1.2rem;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
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
  display: none; /* Hidden by default, shown on desktop */
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

.sidebar-tooltip {
  left: 100%;
  top: 50%;
  bottom: auto;
  transform: translateY(-50%);
  margin-left: 12px;
}

.sidebar-tooltip::after {
  top: 50%;
  right: 100%;
  left: auto;
  bottom: auto;
  transform: translateY(-50%);
  border-top-color: transparent;
  border-right-color: var(--accent);
}

.content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.route-wrapper {
  height: 100%;
  width: 100%;
  overflow-y: auto;
}

@media (min-width: 960px) {
  .flying-message {
    display: block;
  }
  .app-container {
    flex-direction: row;
  }

  .sidebar {
    order: 0;
    width: 48px;
    height: 100vh;
    flex-direction: column;
    border-top: none;
    border-right: 1px solid var(--line);
    padding: 12px 0;
  }

  .logo-section {
    display: flex;
    justify-content: center;
    margin-bottom: 24px;
  }

  .logo-dot {
    width: 6px;
    height: 6px;
    background: var(--accent);
    border-radius: 50%;
  }

  .nav-menu {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0;
  }

  .nav-divider {
    display: block;
    height: 1px;
    background: var(--line);
    margin: 8px 12px;
  }

  .nav-item {
    width: 100%;
    padding: 8px 0;
    position: relative;
  }

  .nav-label {
    display: none;
  }

  .nav-item.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    background: var(--accent);
  }

  .icon-wrapper {
    font-size: 1.1rem;
  }
}
</style>