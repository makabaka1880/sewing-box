<template>
    <PlaygroundLayout v-model:show-grammar="showGrammar">
        <template #header>
            <h2>{{ getLang("Brainfuck")?.name }}</h2>
            <p>{{ getLang("Brainfuck")?.description }}</p>
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

        <template #result-label>TAPE</template>

        <template #result-header>
            <span v-if="steps > 0" class="step-count">{{ steps }} step{{ steps !== 1 ? 's' : '' }}</span>
            <button v-if="steps > 0" class="clear-btn" @click="doClear" title="Clear">&#x2715;</button>
        </template>

        <template #result>
            <div class="result-inner">
                <template v-if="program">
                    <div class="tape-nav">
                        <button class="nav-btn" @click="pagePrev" :disabled="windowStart <= -128">&#x2190; PREV</button>
                        <button class="nav-btn" @click="goToHead">&#x2316; HEAD</button>
                        <button class="nav-btn" @click="pageNext" :disabled="windowStart >= 128">NEXT &#x2192;</button>
                        <span class="window-range">{{ windowStart }} – {{ windowStart + WINDOW_SIZE - 1 }}</span>
                    </div>
                    <div class="instr-bar">
                        <span>ptr&nbsp;{{ instrStatus.ptr }}</span>
                        <span class="instr-cur">cur&nbsp;{{ instrStatus.cur }}</span>
                        <span v-if="instrStatus.next">next&nbsp;{{ instrStatus.next }}</span>
                    </div>
                    <div class="tape-grid">
                        <div v-for="cell in tapeCells" :key="cell.idx" class="tape-cell"
                            :class="{ active: cell.idx === tapePtr }">
                            <span class="cell-idx">{{ cell.idx }}</span>
                            <span class="cell-val">{{ cell.val }}</span>
                            <span class="cell-glyph">{{ cell.glyph }}</span>
                        </div>
                    </div>
                    <div class="output-box">
                        <!-- <span class="output-label">output</span> -->
                        <pre v-if="outputText" class="output-text">{{ outputText }}</pre>
                        <pre v-else class="output-empty-label">Output Stream Empty.</pre>
                    </div>
                    <div class="input-section">
                        <div class="input-row">
                            <textarea v-model="inputText" class="input-area"
                                placeholder="Input for , (read) instructions…" rows="2" spellcheck="false"></textarea>
                        </div>
                        <div class="input-meta">
                            <button class="feed-btn" @click="doFeed" :disabled="!inputText">FEED &#x21B5;</button>
                            <span v-if="inputRemaining > 0" class="input-remaining">{{ inputRemaining }} byte{{
                                inputRemaining !== 1 ? 's' : '' }} left</span>
                            <span v-if="waitingInput" class="waiting-hint">&#x26A0; program wants input — feed
                                above</span>
                        </div>
                    </div>
                    <div v-if="nfReached" class="nf-msg">Program halted.</div>
                    <div v-if="errorReached" class="error-msg">{{ errorMessage }}</div>
                </template>
                <div v-else class="placeholder">Enter a Brainfuck program in the editor</div>
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
import { BFProgramWasm } from "@sewing-box/wasm-brainfuck";

const WINDOW_SIZE = 24;

const store = useEditorStore();
const showGrammar = ref(false);
const grammarEBNF = generateEBNF(getGrammar('Brainfuck') ?? []);
const sample = getSample('Brainfuck') ?? '';

const code = ref(store.getCode('Brainfuck', sample));
const inputText = ref('');
const windowStart = ref(-11);
const program = ref<BFProgramWasm | null>(null);
const steps = ref(0);
const nfReached = ref(false);
const errorReached = ref(false);
const errorMessage = ref('');
const waitingInput = ref(false);

function resetWindow() {
    windowStart.value = -11;
}

watch(code, (src) => {
    try {
        program.value = new BFProgramWasm(src);
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
        waitingInput.value = false;
        resetWindow();
        if (inputText.value) {
            feedProgram();
        }
    } catch (e) {
        program.value = null;
        steps.value = 0;
        nfReached.value = false;
        errorReached.value = false;
        errorMessage.value = '';
        waitingInput.value = false;
        resetWindow();
    }
}, { immediate: true });

watch(code, (v) => store.setCode('Brainfuck', v));

const halted = computed(() => nfReached.value || errorReached.value || waitingInput.value);

const inputRemaining = computed(() => {
    if (!program.value) return 0;
    void steps.value;
    return program.value.input_len();
});

function feedProgram() {
    if (!program.value || !inputText.value) return;
    const bytes = new Int32Array([...inputText.value].map(c => c.charCodeAt(0)));
    program.value.feed_input(bytes);
}

function doFeed() {
    if (!program.value || !inputText.value) return;
    feedProgram();
    waitingInput.value = false;
    errorReached.value = false;
    errorMessage.value = '';
}

const tapePtr = computed(() => {
    if (!program.value) return 0;
    void steps.value;
    return program.value.tape_ptr();
});

const tapeCells = computed(() => {
    if (!program.value) return [];
    void steps.value;
    const from = windowStart.value;
    const to = from + WINDOW_SIZE;
    const slice = program.value.tape_slice(from, to);
    const cells: { idx: number; val: number; glyph: string }[] = [];
    for (let i = 0; i < slice.length; i++) {
        const idx = from + i;
        const val = slice[i]!;
        cells.push({
            idx,
            val,
            glyph: val >= 32 && val < 127 ? String.fromCharCode(val) : '·',
        });
    }
    return cells;
});

const instrStatus = computed(() => {
    if (!program.value) return { ptr: 0, cur: '—', next: null as string | null };
    void steps.value;
    const p = program.value;
    const ptr = p.prog_ptr();
    const cur = p.instr_at(ptr) ?? '—';
    const next = p.instr_at(ptr + 1) ?? null;
    return { ptr, cur, next };
});

function pagePrev() {
    windowStart.value -= WINDOW_SIZE;
}

function pageNext() {
    windowStart.value += WINDOW_SIZE;
}

function goToHead() {
    if (program.value) {
        windowStart.value = program.value.tape_ptr() - Math.floor(WINDOW_SIZE / 2);
    }
}

const outputText = computed(() => {
    if (!program.value) return '';
    void steps.value;
    const out = program.value.output();
    return Array.from(out).map(v => String.fromCharCode(v)).join('');
});

function doStep() {
    if (!program.value || halted.value) return;
    let done: boolean;
    try {
        done = program.value.step();
    } catch (e) {
        const msg = String(e);
        if (/empty input/i.test(msg)) {
            waitingInput.value = true;
            return;
        }
        errorReached.value = true;
        errorMessage.value = msg;
        return;
    }
    steps.value++;
    waitingInput.value = false;
    console.log("halted", done)
    if (done) {
        nfReached.value = true;
    }
}

const MAX_RUN = 4096;

function doRun() {
    if (!program.value || halted.value) return;
    doClear();
    let done = false;
    for (let i = 0; i < MAX_RUN; i++) {
        try {
            done = program.value.step();
        } catch (e) {
            const msg = String(e);
            console.log(e)
            if (/empty input/i.test(msg)) {
                waitingInput.value = true;
                return;
            }
            errorReached.value = true;
            errorMessage.value = msg;
            steps.value++;
            return;
        }
        steps.value++;
        console.log(done)
        if (done) {
            nfReached.value = true;
            return;
        }
    }
    errorReached.value = true;
    errorMessage.value = `Terminated after ${MAX_RUN} steps (possible non-termination).`;
}

function doClear() {
    steps.value = 0;
    nfReached.value = false;
    errorReached.value = false;
    errorMessage.value = '';
    waitingInput.value = false;
    resetWindow();
    if (code.value) {
        try {
            program.value = new BFProgramWasm(code.value);
            if (inputText.value) {
                feedProgram();
            }
        } catch (e) {
            program.value = null;
        }
    }
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.result-inner {
    padding-top: var(--padding-safe);
    padding-bottom: var(--padding-safe);
    padding-inline: var(--padding-safe);
}

.placeholder {
    color: var(--text-muted);
    font-size: 0.75rem;
}

.tape-nav {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    margin-bottom: 0.35rem;
}

.nav-btn {
    background: var(--surface-0);
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.1rem 0.4rem;
    font-family: var(--mono);
    font-size: 0.55rem;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;

    &:hover:not(:disabled) {
        color: var(--text);
        border-color: var(--text-muted);
    }

    &:disabled {
        opacity: 0.3;
        cursor: default;
    }
}

.instr-bar {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 0.35rem;
    font-family: var(--mono);
    font-size: 0.55rem;
    color: var(--text-muted);
}

.instr-cur {
    color: var(--accent);
}

.window-range {
    margin-left: auto;
    font-size: 0.55rem;
    color: var(--text-muted);
    font-family: var(--mono);
}

.tape-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
}

.tape-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.2rem 0;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--mono);
    font-size: 0.55rem;
    color: var(--text-muted);
    transition: background-color 0.15s, border-color 0.15s;

    &.active {
        background-color: var(--surface-1);
        border-color: var(--accent);
        color: var(--text);

        .cell-glyph {
            color: var(--accent);
        }
    }
}

.cell-idx {
    font-size: 0.45rem;
    opacity: 0.7;
}

.cell-val {
    font-size: 0.65rem;
}

.cell-glyph {
    font-size: 0.7rem;
}

.output-box {
    margin-top: 0.5rem;
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.35rem 0.5rem;
}

.output-label {
    font-size: 0.55rem;
    color: var(--text-muted);
    text-transform: uppercase;
}

.output-text {
    margin: 0.25rem 0 0;
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-all;
}

.output-empty-label {
    margin: 0.25rem 0 0;
    font-family: var(--mono);
    font-size: 0.7rem;
    color: var(--text-muted);
    white-space: pre-wrap;
    word-break: break-all;
}

// ---- input ----

.input-section {
    margin-top: 0.5rem;
}

.input-row {
    display: flex;
    gap: 0.35rem;
}

.input-area {
    flex: 1;
    background: var(--surface-0);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.25rem 0.4rem;
    font-family: var(--mono);
    font-size: 0.65rem;
    resize: vertical;
    outline: none;

    &:focus {
        border-color: var(--accent);
    }

    &::placeholder {
        color: var(--text-muted);
        opacity: 0.6;
    }
}

.input-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
}

.feed-btn {
    background: var(--surface-0);
    color: var(--accent);
    border: 1px solid var(--accent);
    border-radius: 3px;
    padding: 0.15rem 0.5rem;
    font-family: var(--mono);
    font-size: 0.6rem;
    cursor: pointer;
    transition: background-color var(--anim-dur), color var(--anim-dur);

    &:hover:not(:disabled) {
        background: var(--accent);
        color: var(--surface-0);
    }

    &:disabled {
        opacity: 0.4;
        cursor: default;
    }
}

.input-remaining {
    font-size: 0.55rem;
    color: var(--text-muted);
    font-family: var(--mono);
}

.waiting-hint {
    font-size: 0.6rem;
    color: var(--accent);
    font-family: var(--mono);
}

// ---- status messages ----

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
