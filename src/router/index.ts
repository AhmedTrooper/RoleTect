import { createRouter, createWebHashHistory } from 'vue-router';
import HomeTab from '../components/HomeTab.vue';
import SettingsTab from '../components/SettingsTab.vue';
import JobDetailView from '../components/JobDetailView.vue';
import ResumesTab from '../components/ResumesTab.vue';
import ResumeDetailView from '../components/ResumeDetailView.vue';

const routes = [
  { 
    path: '/', 
    name: 'Home', 
    component: HomeTab 
  },
  { 
    path: '/settings', 
    name: 'Settings', 
    component: SettingsTab 
  },
  {
    path: '/resumes',
    name: 'Resumes',
    component: ResumesTab
  },
  {
    path: '/resume/:id',
    name: 'ResumeDetail',
    component: ResumeDetailView,
    props: true
  },
  { 
    path: '/parse', 
    name: 'JobParser', 
    component: () => import('../components/JobParserView.vue') 
  },
  { 
    // Dynamic route for specific job details
    path: '/job/:id', 
    name: 'JobDetail', 
    component: JobDetailView, 
    props: true // Passes the :id as a prop to the component
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});