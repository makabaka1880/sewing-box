import { createApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import { createPinia } from 'pinia'

import HomeView from './pages/HomeView.vue'
import App from './App.vue'
import LambdaPage from '@/pages/lang/LambdaPage.vue'
import StackyPage from '@/pages/lang/StackyPage.vue'
import BrainfuckPage from '@/pages/lang/BrainfuckPage.vue'
import LangIndexPage from '@/pages/lang/LangIndexPage.vue'
import NotFoundPage from '@/pages/NotFoundPage.vue'

const routes = [
    {
        path: '/lang',
        children: [
            {
                path: '',
                component: LangIndexPage
            },
            {
                path: 'lambda',
                component: LambdaPage
            },
            {
                path: 'stacky',
                component: StackyPage
            },
            {
                path: 'brainfuck',
                component: BrainfuckPage
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
    history: createWebHashHistory(),
    routes
})

app.use(createPinia())
app.use(router)

app.mount('#app')
