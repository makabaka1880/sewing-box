import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia } from 'pinia'

import HomeView from './pages/HomeView.vue'
import App from './App.vue'
import LambdaPage from '@/pages/LambdaPage.vue'
import StackyPage from '@/pages/StackyPage.vue'
import NotFoundPage from '@/pages/NotFoundPage.vue'

const routes = [
    {
        path: '/lang',
        children: [
            {
                path: 'lambda',
                component: LambdaPage
            },
            {
                path: 'stacky',
                component: StackyPage
            },
        ]
    },
    {
        path: '/',
        component: HomeView
    },
    {
        path: '/:pathMatch(.*)*',
        component: NotFoundPage
    }
]

const app = createApp(App)

const router = createRouter({
    history: createWebHistory(),
    routes
})

app.use(createPinia())
app.use(router)

app.mount('#app')
