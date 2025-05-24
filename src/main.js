import Antd from 'ant-design-vue'
import { createApp } from 'vue'
import App from './App.vue'
import 'ant-design-vue/dist/reset.css'
import './style.css'

const app = createApp(App)
app.use(Antd)
app.mount('#app')
