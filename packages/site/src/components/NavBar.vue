<template>
    <nav>
        <span>
            <RouterLink to="/">Sewing Box</RouterLink>
        </span>
        <a class="gh-link" href="https://github.com/makabaka1880/sewing-box" target="_blank" rel="noopener">GitHub</a>
    </nav>
</template>

<script lang="ts" setup>

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

nav {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border);
    padding: 1rem var(--padding-safe);
}

.gh-link {
    margin-left: auto;
    color: var(--text-muted);
    font-size: 0.85rem;
    text-decoration: none;
    transition: color 0.15s;

    &:hover {
        color: var(--text);
    }
}
</style>