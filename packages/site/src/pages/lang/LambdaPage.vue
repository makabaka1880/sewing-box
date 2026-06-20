<template>
    <PlaygroundLayout v-model:show-grammar="showGrammar">
        <template #header>
            <h2>{{ getLang("Lambda")?.name }}</h2>
            <p>{{ getLang("Lambda")?.description }}</p>
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
            <CodeMirror v-model="code" />
        </template>

        <template #result-label>RESULT</template>

        <template #result-header>
            <span v-if="steps > 0" class="step-count">{{ steps }} step{{ steps !== 1 ? 's' : '' }}</span>
            <button v-if="frames.length > 0" class="clear-btn" @click="clearStack" title="Clear stack">&#x2715;</button>
        </template>

        <template #result>
            <div class="result-inner">
                <template v-if="config">
                    <div class="live-term" v-html="liveTermHtml"></div>
                    <LambdaFrameCard v-for="(frame, i) in frames" :key="i" :config="frame" :nf="frame.nf"
                        :error="frame.error" class="frame-entry" />
                    <div v-if="errorReached" class="error-msg">{{ errorMessage }}</div>
                </template>
                <div v-else class="placeholder">Enter a lambda term in the editor</div>
            </div>
        </template>

        <template #grammar-body>
            <pre>{{ grammarEBNF }}</pre>
        </template>
    </PlaygroundLayout>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from 'vue';
import { getLang, getGrammar, generateEBNF, getSample } from '@/cfg/langs';
import { useEditorStore } from '@/stores/editor';
import PlaygroundLayout from '@/components/PlaygroundLayout.vue';
import CodeMirror from '@/components/CodeMirror.vue';
import LambdaFrameCard from '@/components/Lambda/LambdaFrameCard.vue';
import { LambdaProgram } from "@sewing-box/wasm-lambda";
import katex from 'katex';
import 'katex/dist/katex.min.css';

const store = useEditorStore();
const showGrammar = ref(false);
const grammarEBNF = generateEBNF(getGrammar('Lambda') ?? []);
const sample = getSample('Lambda') ?? '';

const code = ref(store.getCode('Lambda', sample));
const program = ref<LambdaProgram | null>(null);
const frames = ref<any[]>([]);
const steps = ref(0);
const nfReached = ref(false);
const errorReached = ref(false);
const errorMessage = ref('');

function termToTexLive(term: any): string {
    if ('Var' in term) return term.Var;
    if ('Lam' in term) {
        const [param, body] = term.Lam;
        return `\\lambda ${param}. ${termToTexLive(body)}`;
    }
    const [M, N] = term.App;
    return `(${termToTexLive(M!)})\\;( ${termToTexLive(N!)})`;
}

watch(code, (src) => {
    try {
        program.value = new LambdaProgram(src);
        frames.value = [];
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
    } catch (e) {
        program.value = null;
        frames.value = [];
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
    }
}, { immediate: true });

watch(code, (v) => store.setCode('Lambda', v));

const halted = computed(() => nfReached.value || errorReached.value);

const config = computed(() => {
    if (!program.value) return null;
    void steps.value; // WASM state is opaque — force recompute when steps changes
    return program.value.state();
});

const liveTermHtml = computed(() => {
    if (!config.value) return '';
    return katex.renderToString(termToTexLive(config.value.term), {
        displayMode: true,
        throwOnError: false,
    });
});

function doStep() {
    if (!program.value || halted.value) return;
    const pre = program.value.state();
    let done: boolean;
    try {
        done = program.value.step();
    } catch (e) {
        errorReached.value = true;
        errorMessage.value = String(e);
        frames.value.push({ ...pre, error: true });
        steps.value++;
        return;
    }
    frames.value.push(pre);
    steps.value++;
    if (done) {
        frames.value[frames.value.length - 1].nf = true;
        nfReached.value = true;
    }
}

const MAX_RUN = 64;

function doRun() {
    if (!program.value || halted.value) return;
    clearStack();
    let prev: any = null;
    for (let i = 0; i < MAX_RUN; i++) {
        prev = program.value.state();
        let done: boolean;
        try {
            done = program.value.step();
        } catch (e) {
            errorReached.value = true;
            errorMessage.value = String(e);
            frames.value.push({ ...prev, error: true });
            steps.value++;
            return;
        }
        frames.value.push(prev);
        steps.value++;
        if (done) {
            frames.value[frames.value.length - 1].nf = true;
            nfReached.value = true;
            return;
        }
    }
    // Timed out after MAX_RUN steps
    errorReached.value = true;
    errorMessage.value = `Terminated after ${MAX_RUN} steps (possible non-termination).`;
    frames.value[frames.value.length - 1].error = true;
}

function clearStack() {
    frames.value = [];
    steps.value = 0;
    nfReached.value = false;
    errorReached.value = false;
    errorMessage.value = '';
    // Re-seed the program from current code to get a fresh machine state
    if (code.value) {
        try {
            program.value = new LambdaProgram(code.value);
        } catch (e) {
            program.value = null;
        }
    }
}

</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

// ---- result panel content (page-local) ----

.result-inner {
    padding-top: var(--padding-safe);
    padding-bottom: var(--padding-safe);
    padding-inline: var(--padding-safe);
}

.live-term {
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
    margin-bottom: 0.5rem;
}

.frame-entry {
    margin-bottom: 0.5rem;
}

.placeholder {
    color: var(--text-muted);
    font-size: 0.75rem;
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
