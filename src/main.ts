import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { router } from './router';
import App from './App.vue';
import VueVirtualScroller from 'vue-virtual-scroller';
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css';
import "./style.css";

(async () => {
	const app = createApp(App);
	const pinia = createPinia();

	app.use(pinia);
	app.use(router);
	app.use(VueVirtualScroller);

	app.mount('#app');
})();