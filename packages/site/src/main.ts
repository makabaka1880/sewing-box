import { createApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import { createPinia } from 'pinia'

import HomeView from './pages/HomeView.vue'
import App from './App.vue'
import LambdaPage from '@/pages/lang/LambdaPage.vue'
import StackyPage from '@/pages/lang/StackyPage.vue'
import BrainfuckPage from '@/pages/lang/BrainfuckPage.vue'
import Lite80Page from '@/pages/lang/Lite80Page.vue'
import LangIndexPage from '@/pages/lang/LangIndexPage.vue'
import NotFoundPage from '@/pages/NotFoundPage.vue'

const routes = [
    {
        path: '/lang',
        meta: { title: 'Languages' },
        children: [
            {
                path: '',
                meta: { title: 'Languages' },
                component: LangIndexPage
            },
            {
                path: 'lambda',
                meta: { title: 'Lambda' },
                component: LambdaPage
            },
            {
                path: 'stacky',
                meta: { title: 'Stacky' },
                component: StackyPage
            },
            {
                path: 'brainfuck',
                meta: { title: 'Brainfuck' },
                component: BrainfuckPage
            },
            {
                path: 'lite80',
                meta: { title: 'Lite80' },
                component: Lite80Page
            },
        ]
    },
    {
        path: '/',
        meta: { title: 'Home' },
        component: HomeView
    },
    {
        path: '/:pathMatch(.*)*',
        meta: { title: '404' },
        component: NotFoundPage
    }
]

const DEFAULT_TITLE = 'Sewing Box'

const app = createApp(App)

const router = createRouter({
    history: createWebHashHistory(),
    routes
})

router.afterEach((to) => {
    const pageTitle = to.meta.title as string | undefined
    document.title = pageTitle ? `${pageTitle} | ${DEFAULT_TITLE}` : DEFAULT_TITLE
})

app.use(createPinia())
app.use(router)

app.mount('#app')
