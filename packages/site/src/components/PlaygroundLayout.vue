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
