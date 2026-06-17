<template>
    <NavBar />
    <RouterView />
    <Footer />
</template>

<script lang="ts" setup>
import Footer from './components/Footer.vue';
import NavBar from './components/NavBar.vue';


</script>

<style lang="scss">
@use "@/style/main.scss" as *;
</style>