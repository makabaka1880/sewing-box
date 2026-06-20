<template>
    <NavBar />
    <RouterView />
    <Footer />
    <Teleport to="body">
        <div v-if="grammarModal.show" class="pg-modal-overlay" @click.self="grammarModal.show = false">
            <div class="pg-modal">
                <div class="pg-modal-header">
                    <span>Grammar (EBNF)</span>
                    <button class="pg-modal-close" @click="grammarModal.show = false">&times;</button>
                </div>
                <div class="pg-modal-body" v-html="grammarModal.content" />
            </div>
        </div>
    </Teleport>
</template>

<script lang="ts" setup>
import { provide, reactive } from 'vue';
import Footer from './components/Footer.vue';
import NavBar from './components/NavBar.vue';

const grammarModal = reactive<{ show: boolean; content: string }>({
    show: false,
    content: '',
});
provide('grammarModal', grammarModal);
</script>

<style lang="scss">
@use "@/style/main.scss" as *;

// Grammar modal — teleported to body, must be unscoped.
.pg-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
}

.pg-modal {
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: 6px;
    width: min(520px, 90vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.pg-modal-header {
    padding: 0.6rem var(--padding-safe);
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;
}

.pg-modal-close {
    background: none;
    border: none;
    font-size: 1.5em;
    cursor: pointer;
    color: var(--text);
}

.pg-modal-body {
    padding: var(--padding-safe);
    margin: 0;
    font-family: var(--mono);
    font-size: 0.8rem;
    line-height: 1.6;
    overflow: auto;
    white-space: pre-wrap;
    color: var(--text-muted);
}
</style>