<template>
    <div class="cm-wrapper">
        <codemirror v-model="code" placeholder="Code goes here..." :style="{ height: '100%' }" :autofocus="true"
            :indent-with-tab="true" :tab-size="2" :extensions="extensions" @ready="handleReady" />
        <Transition name="save-fade">
            <div v-if="saved" class="save-hint">File Saved!</div>
        </Transition>
    </div>
</template>

<script lang="ts" setup>
import { shallowRef, ref } from 'vue'
import { Codemirror } from 'vue-codemirror'
import { keymap } from '@codemirror/view'

const code = defineModel<string>({ default: '' })

const emit = defineEmits<{
    save: []
}>()

const saved = ref(false)
let saveTimer: ReturnType<typeof setTimeout> | null = null

function showSaved() {
    saved.value = true
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => { saved.value = false }, 1800)
    emit('save')
}

const saveKeymap = keymap.of([{
    key: 'Mod-s',
    run: () => { showSaved(); return true },
    preventDefault: true,
}])

const extensions = [saveKeymap]

const view = shallowRef()
const handleReady = (payload: any) => {
    view.value = payload.view
}
</script>

<style lang="scss" scoped>
.cm-wrapper {
    position: relative;
    height: 100%;
}

.save-hint {
    position: absolute;
    bottom: .5rem;
    right: .75rem;
    z-index: 2;
    font-size: 0.6rem;
    font-family: var(--mono);
    color: var(--accent);
    background: var(--surface-0);
    border: 1px solid var(--accent);
    border-radius: 3px;
    padding: 0.15rem 0.4rem;
    pointer-events: none;
}

.save-fade-enter-active {
    transition: opacity 0.15s ease;
}

.save-fade-leave-active {
    transition: opacity 0.6s ease;
}

.save-fade-enter-from,
.save-fade-leave-to {
    opacity: 0;
}
</style>
