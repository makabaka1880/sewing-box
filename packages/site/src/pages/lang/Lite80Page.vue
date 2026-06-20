<template>
    <PlaygroundLayout>
        <template #header>
            <h2>{{ getLang("Lite80")?.name }}</h2>
            <p>{{ getLang("Lite80")?.description }}</p>
        </template>

        <template #header-actions>
            <button class="secondary" @click="openSpec">Spec &#x2197;</button>
        </template>

        <template #editor-label>EDITOR</template>

        <template #editor-tools>
            <button class="secondary" title="Assemble" @click="doAssemble">ASSEMBLE &#9874; </button>
            <button class="primary" title="Step" @click="doStep" :disabled="!program || halted">STEP &#x2192;</button>
            <button class="secondary" title="Run" @click="doRun" :disabled="!program || halted">RUN &#x21D3;</button>
        </template>

        <template #editor>
            <CodeMirror v-model="code" />
        </template>

        <template #editor-below>
            <section class="config" :class="{ collapsed: !memOpen }">
                <button class="section-header config-toggle" @click="memOpen = !memOpen">
                    Memory
                    <span class="toggle-arrow" :class="{ open: memOpen }">&#x25B8;</span>
                </button>
                <div v-show="memOpen" class="config-body">
                    <MemView :memory="memory" :pc="pc" :sp="sp" @edit-byte="onMemEdit" />
                </div>
            </section>
        </template>

        <template #result-label>MACHINE</template>

        <template #result-header>
            <span v-if="steps > 0" class="step-count">{{ steps }} step{{ steps !== 1 ? 's' : '' }}</span>
        </template>

        <template #result>
            <div class="result-wrapper">
                <section class="result">
                    <RegsCard :values="regs" :pc="pc" :sp="sp" :int-enabled="intEnabled"
                        @update:pc="onPcChange" @update:sp="onSpChange" @update:reg="onRegChange"
                        @update:int-enabled="onIntEnabledChange" />
                    <div v-if="curAsm" class="cur-asm">{{ curAsm }}</div>
                    <div v-if="curBytes" class="cur-bytes"><span class="bytes-addr">{{ curBytesAddr }}</span> {{ curBytes }}</div>
                    <div v-if="halted" class="halted-msg">Program halted.</div>
                    <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
                </section>
                <section class="config" :class="{ collapsed: !configOpen }">
                    <button class="section-header config-toggle" @click="configOpen = !configOpen">
                        IO Bus
                        <span class="toggle-arrow" :class="{ open: configOpen }">&#x25B8;</span>
                    </button>
                    <div v-show="configOpen" class="config-body">
                        <IoGrid v-model="ioBus" :port-history="portHistory" />
                    </div>
                </section>
            </div>
            <!-- TODO: machine state visualisation -->
        </template>
    </PlaygroundLayout>
</template>

<script lang="ts" setup>
import { inject, shallowRef, ref, watch } from 'vue';
import { getLang, getGrammar, generateEBNF, getSample } from '@/cfg/langs';
import { useEditorStore } from '@/stores/editor';
import PlaygroundLayout from '@/components/PlaygroundLayout.vue';
import CodeMirror from '@/components/CodeMirror.vue';
import IoGrid, { type HistoryEntry } from '@/components/Lite80/IoGrid.vue';
import RegsCard from '@/components/Lite80/RegsCard.vue';
import { I8080Wasm } from "@sewing-box/wasm-i8080";
import MemView from '@/components/Lite80/MemView.vue';

const store = useEditorStore();
const grammarModal = inject<{ show: boolean; content: string }>('grammarModal')!;
const grammarEBNF = generateEBNF(getGrammar('Lite80') ?? []);
function openSpec() {
    grammarModal.content = `<pre>${grammarEBNF}</pre>`;
    grammarModal.show = true;
}
const sample = getSample('Lite80') ?? '';

const code = ref(store.getCode('Lite80', sample));
const configOpen = ref(true);
const ioBus = ref<number[]>(Array(256).fill(0));
const pc = ref(0x0100);
const sp = ref(0xF000);
const regs = ref<number[]>([0, 0, 0, 0, 0, 0, 0, 0]);
const intEnabled = ref(false);
const program = ref<I8080Wasm | null>(null);
const errorMsg = ref('');
const steps = ref(0);
const halted = ref(false);
const curAsm = ref('');
const curBytes = ref('');
const curBytesAddr = ref('');
const portHistory = ref<Record<number, HistoryEntry[]>>({});
const MAX_HISTORY = 40;
const memory = shallowRef<Uint8Array>(new Uint8Array(65536));
const memOpen = ref(true);

function pushPortHistory(addr: number, entry: HistoryEntry) {
    if (!portHistory.value[addr]) portHistory.value[addr] = [];
    portHistory.value[addr].push(entry);
    if (portHistory.value[addr].length > MAX_HISTORY) portHistory.value[addr].shift();
}

function syncAsm() {
    curAsm.value = program.value?.disasm() ?? '';
    const bytes = program.value?.instr_bytes();
    if (bytes && bytes.length > 0) {
        curBytesAddr.value = (pc.value & 0xFFFF).toString(16).toUpperCase().padStart(4, '0') + ':';
        curBytes.value = Array.from(bytes).map(b => b.toString(2).padStart(8, '0')).join(' ');
    } else {
        curBytes.value = '';
    }
}

watch(code, (v) => store.setCode('Lite80', v));

function syncState() {
    if (!program.value) return;
    regs.value = Array.from(program.value.regs());
    pc.value = program.value.pc();
    sp.value = program.value.sp();
    ioBus.value = Array.from(program.value.ports());
    intEnabled.value = program.value.int_enabled();
    syncAsm();
    syncMemory();
}

function syncMemory() {
    if (!program.value) return;
    memory.value = program.value.memory_slice(0, 0x10000);
}

function onMemEdit(addr: number, value: number) {
    if (!program.value) return;
    program.value.memory_write_byte(addr, value);
    memory.value = program.value.memory_slice(0, 0x10000);
}

function onPcChange(val: number) {
    pc.value = val;
    program.value?.set_pc(val);
    halted.value = false;
    errorMsg.value = '';
    syncAsm();
}
function onSpChange(val: number) {
    sp.value = val;
    program.value?.set_sp(val);
    syncAsm();
}
function onRegChange(idx: number, val: number) {
    const next = [...regs.value];
    next[idx] = val;
    regs.value = next;
    program.value?.set_regs(new Uint8Array(next));
    if (idx === 6) syncState(); // flags may affect int_enable visuals
}
function onIntEnabledChange(val: boolean) {
    intEnabled.value = val;
    program.value?.set_int_enabled(val);
}

function doAssemble() {
    errorMsg.value = '';
    halted.value = false;
    steps.value = 0;
    portHistory.value = {};
    try {
        program.value = new I8080Wasm(code.value, new Uint8Array(ioBus.value), pc.value, sp.value);
        syncState();
    } catch (e) {
        program.value = null;
        errorMsg.value = String(e);
    }
}

function doStep() {
    if (!program.value || halted.value) return;
    errorMsg.value = '';
    program.value.set_ports(new Uint8Array(ioBus.value));
    const prevPc = program.value.pc();
    const isOut = program.value.disasm().startsWith('(out');
    try {
        const done = program.value.step();
        steps.value++;
        if (isOut) {
            const curPorts = Array.from(program.value.ports());
            for (let addr = 0; addr < curPorts.length; addr++) {
                pushPortHistory(addr, { step: steps.value, pc: prevPc, val: curPorts[addr]! });
            }
        }
        syncState();
        if (done) halted.value = true;
    } catch (e) {
        errorMsg.value = String(e);
        syncState();
    }
}

const MAX_RUN = 40960;

function doRun() {
    if (!program.value || halted.value) return;
    errorMsg.value = '';
    for (let i = 0; i < MAX_RUN; i++) {
        try {
            const prevPc = program.value.pc();
            const isOut = program.value.disasm().startsWith('(out');
            const done = program.value.step();
            steps.value++;
            if (isOut) {
                const curPorts = Array.from(program.value.ports());
                for (let addr = 0; addr < curPorts.length; addr++) {
                    pushPortHistory(addr, { step: steps.value, pc: prevPc, val: curPorts[addr]! });
                }
            }
            if (done) {
                halted.value = true;
                break;
            }
        } catch (e) {
            errorMsg.value = String(e);
            break;
        }
    }
    syncState();
    if (!halted.value && !errorMsg.value) {
        errorMsg.value = `Terminated after ${MAX_RUN} steps (possible non-termination).`;
    }
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.result-wrapper {
    display: flex;
    flex-direction: column;
    min-height: 100%;
}

.config {
    flex: 0 0 auto;
    border-top: 1px var(--border) solid;
    padding-left: var(--padding-safe);
    padding-right: var(--padding-safe);
    padding-top: .5rem;

    &.collapsed {
        padding-top: .35rem;
        padding-bottom: .35rem;
    }
}

.config-toggle {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    width: 100%;
    background: none;
    border: none;
    padding: 0;
    font-size: 0.85rem;
    font-family: inherit;
    cursor: pointer;
    transition: color 0.15s;

}

.toggle-arrow {
    font-size: 0.65rem;
    transition: transform 0.15s;

    &.open {
        transform: rotate(90deg);
    }
}

.config-body {
    padding-top: 0.5rem;
}

.result {
    flex: 1 0 auto;
    padding: var(--padding-safe);
}

.section-header {
    color: var(--text-muted);
    font-size: 0.85rem;
}

.cur-asm {
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--accent);
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
}

.cur-bytes {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-muted);
    opacity: 0.6;
}

.bytes-addr {
    font-weight: 700;
    color: var(--accent);
}

.halted-msg {
    color: var(--accent);
    font-size: 0.65rem;
    font-family: var(--mono);
    margin-top: 0.35rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--accent);
    border-radius: 3px;
}

.step-count {
    font-size: 0.6rem;
    color: var(--text-muted);
}

.error-msg {
    color: var(--error);
    font-size: 0.65rem;
    font-family: var(--mono);
    margin-top: 0.35rem;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--error);
    border-radius: 3px;
    white-space: pre-wrap;
    word-break: break-all;
}
</style>
