<template>
    <footer>
        <p>All Rights Reserved <a href="https://blog.makabaka1880.xyz">Makabaka1880 &copy; 2026 </a></p>
    </footer>
</template>

<script lang="ts" setup>

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

footer {
    p {
        margin: 0;
        color: var(--text-muted);
    }

    height: min-content;
    border-top: 1px solid var(--border);
    padding: var(--padding-safe);
}
</style>