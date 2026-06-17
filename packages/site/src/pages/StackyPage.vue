<template>
    <PlaygroundLayout>
        <template #header>
            <h2>{{ getLang("Stacky")?.name }}</h2>
            <p>{{ getLang("Stacky")?.description }}</p>
        </template>

        <template #header-actions>
            <button class="secondary" @click="showGrammar = true">Spec &#x2197;</button>
        </template>

        <template #editor-label>EDITOR</template>

        <template #editor-tools>
            <button class="primary" title="Step" @click="doStep" :disabled="halted">STEP &#x2192;</button>
            <button class="secondary" title="Run" @click="doRun" :disabled="halted">RUN &#x21D3;</button>
        </template>

        <template #editor>
            <CodeMirror v-model="code" :default-content="sample" />
        </template>

        <template #result-label>STACK</template>

        <template #result-header>
            <span v-if="steps > 0" class="step-count">{{ steps }} step{{ steps !== 1 ? 's' : '' }}</span>
            <button v-if="steps > 0" class="clear-btn" @click="clearStack" title="Clear">&#x2715;</button>
        </template>

        <template #result>
            <div class="result-inner">
                <template v-if="config">
                    <div class="tree-stack">
                        <div v-for="(node, i) in config.stack" :key="i" class="stack-cell">
                            <span class="stack-index">{{ i }}</span>
                            <TreeCard :node="node" />
                        </div>
                        <div v-if="config.stack.length === 0" class="placeholder">Stack empty</div>
                    </div>
                    <div v-if="nfReached" class="nf-msg">Normal form reached.</div>
                    <div v-if="errorReached" class="error-msg">{{ errorMessage }}</div>
                </template>
                <div v-else class="placeholder">Enter a Stacky program in the editor</div>
            </div>
        </template>
    </PlaygroundLayout>

    <Teleport to="body">
        <div v-if="showGrammar" class="modal-overlay" @click.self="showGrammar = false">
            <div class="modal">
                <div class="modal-header">
                    <span>Grammar (EBNF)</span>
                    <button class="modal-close" @click="showGrammar = false">&times;</button>
                </div>
                <pre class="modal-body">{{ grammarEBNF }}</pre>
            </div>
        </div>
    </Teleport>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from 'vue';
import { getLang, getGrammar, generateEBNF, getSample } from '@/cfg/langs';
import PlaygroundLayout from '@/components/PlaygroundLayout.vue';
import CodeMirror from '@/components/CodeMirror.vue';
import TreeCard from '@/components/Stacky/TreeCard.vue';
import { StackyProgramWasm } from "@sewing-box/wasm-stacky";

const showGrammar = ref(false);
const grammarEBNF = generateEBNF(getGrammar('Stacky') ?? []);
const sample = getSample('Stacky') ?? '';

const code = ref('');
const program = ref<StackyProgramWasm | null>(null);
const steps = ref(0);
const nfReached = ref(false);
const errorReached = ref(false);
const errorMessage = ref('');

watch(code, (src) => {
    try {
        program.value = new StackyProgramWasm(src);
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
    } catch (e) {
        program.value = null;
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
    }
}, { immediate: true });

const halted = computed(() => nfReached.value || errorReached.value);

const config = computed(() => {
    if (!program.value) return null;
    void steps.value; // WASM state is opaque — force recompute when steps changes
    const raw = program.value.state() as { remaining: number; stack: any[] };
    return { remaining: raw.remaining, stack: [...raw.stack].reverse() };
});

function doStep() {
    if (!program.value || halted.value) return;
    let done: boolean;
    try {
        done = program.value.step();
    } catch (e) {
        errorReached.value = true;
        errorMessage.value = String(e);
        return;
    }
    steps.value++;
    if (done) {
        nfReached.value = true;
    }
}

const MAX_RUN = 64;

function doRun() {
    if (!program.value || halted.value) return;
    clearStack();
    let done = false;
    for (let i = 0; i < MAX_RUN; i++) {
        try {
            done = program.value.step();
        } catch (e) {
            errorReached.value = true;
            errorMessage.value = String(e);
            steps.value++;
            return;
        }
        steps.value++;
        if (done) {
            nfReached.value = true;
            return;
        }
    }
    errorReached.value = true;
    errorMessage.value = `Terminated after ${MAX_RUN} steps (possible non-termination).`;
}

function clearStack() {
    steps.value = 0;
    nfReached.value = false;
    errorReached.value = false;
    errorMessage.value = '';
    if (code.value) {
        try {
            program.value = new StackyProgramWasm(code.value);
        } catch (e) {
            program.value = null;
        }
    }
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
}

.modal {
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: 6px;
    width: min(520px, 90vw);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.modal-header {
    padding: 0.6rem var(--padding-safe);
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8rem;

    .modal-close {
        background: none;
        border: none;
        font-size: 1.5em;
    }
}

.modal-body {
    padding: var(--padding-safe);
    margin: 0;
    font-family: var(--mono);
    font-size: 0.8rem;
    line-height: 1.6;
    overflow: auto;
    white-space: pre-wrap;
    color: var(--text-muted);
}

.result-inner {
    padding-bottom: var(--padding-safe);
}

.tree-stack {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.stack-cell {
    position: relative;
    padding: 0.5rem 0;
    border-bottom: 1px var(--border) solid;
}

.stack-index {
    position: absolute;
    top: .75rem;
    left: -1rem;
    font-size: 0.55rem;
    color: var(--text-muted);
    font-family: var(--mono);
}

.placeholder {
    padding-top: var(--padding-safe);
    color: var(--text-muted);
    font-size: 0.75rem;
}

.nf-msg {
    color: var(--accent);
    font-size: 0.7rem;
    font-family: var(--mono);
    margin-top: 0.5rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--accent);
    border-radius: 4px;
}

.error-msg {
    color: var(--error);
    font-size: 0.7rem;
    font-family: var(--mono);
    margin-top: 0.5rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--error);
    border-radius: 4px;
}

.step-count {
    font-size: 0.6rem;
    color: var(--text-muted);
}

.clear-btn {
    background: none;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    font-size: 0.6rem;
    padding: 0.1rem 0.35rem;
    cursor: pointer;
    line-height: 1;

    &:hover {
        color: var(--text);
        border-color: var(--text-muted);
    }
}
</style>
