<template>
    <main>
        <div>
            <h1>404</h1>
            <p>
                Binder "{{ route.fullPath }}" not in scope.
            </p>
            <div class="buttons">
                <button class="primary" @click="goHome">Home -></button>
            </div>
        </div>
    </main>
</template>

<script lang="ts" setup>
import { useRoute, useRouter } from 'vue-router';

const route = useRoute();
const router = useRouter();

function goHome() {
    router.push('/');
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