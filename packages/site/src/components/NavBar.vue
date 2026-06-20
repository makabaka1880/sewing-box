<template>
    <nav>
        <span>
            <RouterLink to="/">Sewing Box</RouterLink>
        </span>
    </nav>
</template>

<script lang="ts" setup>

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

nav {
    border-bottom: 1px solid var(--border);
    padding: 1rem var(--padding-safe);

}
</style>