<template>
    <main>
        <section class="header">
            <h1>Welcome</h1>
            <p>Welcome to Sewing Box, a collection of simple interpreters and their debuggers.</p>
            <div class="buttons">
                <button class="primary" @click="goRandomLanguage">Random Language -></button>
                <button class="secondary" @click="goGitHub">GitHub -></button>
            </div>
        </section>
        <div class="langcards">
            <LangCard v-for="profile in profiles" :profile="profile" />
        </div>
        <p class="muted-info">{{ profiles.length }} languages available in the sewing box.</p>
    </main>
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router';
import { profiles } from '@/cfg/langs';
import LangCard from '@/components/LangCard.vue';

const router = useRouter();

function goRandomLanguage() {
    const profile = profiles[Math.floor(Math.random() * profiles.length)]!;
    router.push('/lang/' + profile.name.toLowerCase());
}

function goGitHub() {
    window.open('https://github.com/makabaka1880/sewing-box', '_blank');
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

section.header {
    margin-left: var(--padding-safe);
}

.langcards {
    border-bottom: 1px var(--border) solid;
}

.muted-info {
    font-size: 0.65rem;
    margin: var(--padding-safe);
    color: var(--text-muted)
}

.buttons {
    margin-bottom: var(--padding-safe);
    display: flex;
    gap: 0.5rem;
}
</style>