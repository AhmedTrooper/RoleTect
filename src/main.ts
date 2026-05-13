import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { router } from './router';
import App from './App.vue';
import { useSettingsStore } from './store/settings';
import "./style.css";

(async () => {
	const app = createApp(App);
	const pinia = createPinia();

	app.use(pinia);
	app.use(router);

	const settingsStore = useSettingsStore(pinia);
	try {
		await settingsStore.loadSettings();
	} catch (error) {
		console.error('Failed to load settings on startup:', error);
	}

	app.mount('#app');
})();