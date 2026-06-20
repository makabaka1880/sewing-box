<template>
    <main>
        <section class="playground-header">
            <div>
                <slot name="header" />
            </div>
            <slot name="header-actions" />
        </section>
        <section class="workspace">
            <div class="panel">
                <div class="panel-header">
                    <slot name="editor-label">EDITOR</slot>
                </div>
                <div class="editor-area">
                    <div v-if="hasEditorTools" class="panel-tools">
                        <slot name="editor-tools" />
                    </div>
                    <slot name="editor" />
                </div>
                <slot name="editor-below" />
            </div>
            <div class="panel">
                <div class="panel-header">
                    <slot name="result-label">RESULT</slot>
                    <div class="panel-header-right">
                        <slot name="result-header" />
                    </div>
                </div>
                <div class="result-area">
                    <slot name="result" />
                </div>
            </div>
        </section>
    </main>
</template>

<script lang="ts" setup>
import { computed, useSlots } from 'vue';

const slots = useSlots();
const hasEditorTools = computed(() => !!slots['editor-tools']);
</script>

<style lang="scss" scoped>
@use "@/style/playground.scss" as *;
</style>

<style lang="scss">
// Grammar modal — teleported to body, must be unscoped.
@use "@/style/main.scss" as *;

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
