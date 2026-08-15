import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './main.less';

document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
  return false;
});

createApp(App).use(createPinia()).mount('#app');
