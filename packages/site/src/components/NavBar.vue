<template>
    <nav>
        <span>
            <RouterLink to="/">Sewing Box</RouterLink>
        </span>
    </nav>
</template>

<script lang="ts" setup>
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const parts = route.fullPath;
console.log(parts)

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

nav {
    border-bottom: 1px solid var(--border);
    padding: 1rem var(--padding-safe);

}
</style>