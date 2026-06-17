<template>
    <main>
        <section class="header">
            <div>
                <h2>{{ getLang("Lambda")?.name }}</h2>
                <p>{{ getLang("Lambda")?.description }}</p>
            </div>
            <button class="secondary" @click="showGrammar = true">Spec &#x2197;</button>
        </section>
        <section class="workspace">
            <div class="panel">
                <div class="panel-header">
                    <span>EDITOR</span>
                </div>
                <div class="editor-area">
                    <div class="panel-tools">
                        <button class="primary" title="Step" @click="doStep" :disabled="halted">STEP
                            &#x2192;</button>
                        <button class="secondary" title="Run" :disabled="halted">RUN &#x21D3;</button>
                    </div>
                    <CodeMirror v-model="code" default-content="(app (lam a a) (lam x (app x x)))" />
                </div>
            </div>
            <div class="panel">
                <div class="panel-header">
                    <span>RESULT</span>
                    <span v-if="steps > 0" class="step-count">{{ steps }} step{{ steps !== 1 ? 's' : '' }}</span>
                </div>
                <div id="exec-stack">
                    <template v-if="config">
                        <div class="live-term" v-html="liveTermHtml"></div>
                        <LambdaFrameCard v-for="(frame, i) in frames" :key="i" :config="frame" :nf="frame.nf"
                            :error="frame.error" class="frame-entry" />
                        <div v-if="errorReached" class="error-msg">{{ errorMessage }}</div>
                    </template>
                    <div v-else class="placeholder">Enter a lambda term in the editor</div>
                </div>
            </div>
        </section>

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
    </main>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from 'vue';
import { getLang, getGrammar, generateEBNF } from '@/cfg/langs';
import CodeMirror from '@/components/CodeMirror.vue';
import LambdaFrameCard from '@/components/Lambda/LambdaFrameCard.vue';
import { LambdaProgram } from "@sewing-box/wasm-lambda";
import katex from 'katex';
import 'katex/dist/katex.min.css';

const showGrammar = ref(false);
const grammarEBNF = generateEBNF(getGrammar('Lambda') ?? []);

const code = ref('');
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

const halted = computed(() => nfReached.value || errorReached.value);

const config = computed(() => {
    if (!program.value) return null;
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
    try {
        program.value.step();
    } catch (e) {
        errorReached.value = true;
        errorMessage.value = String(e);
        frames.value.push({ ...pre, error: true });
        steps.value++;
        return;
    }
    frames.value.push(pre);
    steps.value++;
    const post = program.value.state();
    if (JSON.stringify(post.term) === JSON.stringify(pre.term)) {
        frames.value[frames.value.length - 1].nf = true;
        nfReached.value = true;
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

main {
    width: 100%;
    height: 100%;
}

section.header {
    margin: 0 var(--padding-safe);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
}

section.workspace {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 320px;
    min-height: 0;

    @media (max-width: 720px) {
        grid-template-columns: 1fr;

    }

    .panel {
        display: grid;
        width: 100%;
        height: 100%;
        grid-template-rows: auto 1fr;
        min-height: 0;
        overflow: hidden;
        border-left: 1px var(--border) solid;

        &:first-child {
            border-left: none;
        }
    }

    .panel-header {
        padding: .5rem var(--padding-safe);
        border-bottom: 1px var(--border) solid;
        border-top: 1px var(--border) solid;
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.8rem;
    }

    .editor-area {
        position: relative;
        min-height: 0;
        overflow: hidden;
    }

    #exec-stack {
        padding: var(--padding-safe);
        overflow-y: auto; // Restrict scrolling explicitly to the Y axis
        height: 100%; // Take up exactly the remaining 1fr space
        min-height: 0;
    }

    #exec-stack .placeholder {
        color: var(--text-muted);
        font-size: 0.75rem;
    }

    .live-term {
        border-bottom: 1px solid var(--border);
        padding-bottom: 0.5rem;
        margin-bottom: 0.5rem;
    }

    .frame-entry {
        margin-bottom: 0.5rem;
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

    .panel-tools {
        position: absolute;
        top: 0.35rem;
        right: 0.5rem;
        z-index: 1;
        display: flex;
        gap: 0.35rem;
        background: var(--surface-0);
        border: 1px solid var(--border);
        border-radius: 4px;
        padding: 0.2rem 0.3rem;


        button {
            font-size: 0.6rem;
            padding: .15rem .45rem;

            &:disabled {
                opacity: 0.4;
                cursor: not-allowed;
            }
        }
    }
}
</style>