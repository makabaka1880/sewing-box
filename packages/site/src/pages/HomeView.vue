<template>
    <main>
        <div>
            <h1>Welcome</h1>
            <p>Welcome to Sewing Box, a collection of simple interpreters and their debuggers.</p>
            <div class="buttons">
                <button class="primary" @click="goRandomLanguage">Random Language -></button>
                <button class="secondary" @click="goLang">All Languages -></button>
            </div>
        </div>
    </main>
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router';
import { profiles } from '@/cfg/langs';

const router = useRouter();

function goRandomLanguage() {
    const profile = profiles[Math.floor(Math.random() * profiles.length)]!;
    router.push('/lang/' + profile.name.toLowerCase());
}
function goLang() {
    router.push('/lang/');
}

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

main {
    margin-left: var(--padding-safe);
    display: flex;
    flex-direction: column;
    justify-content: space-around;
}

.buttons {
    margin-bottom: var(--padding-safe);
    display: flex;
    gap: 0.5rem;
}
</style>
