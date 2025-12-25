import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { create } from 'naive-ui'
import App from './App.vue'
import { setupI18n } from './locales'
import './styles/style.css'
import router from './router'

const naive = create()
const pinia = createPinia()
const app = createApp(App).use(pinia).use(router).use(naive)
setupI18n(app)
app.mount('#app')
