import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './main.less';

createApp(App).use(createPinia()).mount("#app");
