<template>
    <div class="io-grid-outer">
        <div class="io-nav">
            <button class="nav-btn" @click="pagePrev" :disabled="page <= 0">&#x2190; PREV</button>
            <span class="page-indicator">page {{ page + 1 }} / {{ PAGE_COUNT }}</span>
            <button class="nav-btn" @click="pageNext" :disabled="page >= PAGE_COUNT - 1">NEXT &#x2192;</button>
            <button class="nav-btn clear-btn" @click="clearPage">CLR &cross;</button>
            <span class="port-range">{{ fmtHex(base, 2) }} – {{ fmtHex(base + CELLS_PER_PAGE - 1, 2) }}</span>
        </div>
        <div class="io-grid">
            <div v-for="cell in cells" :key="cell.addr" class="io-cell" :class="{ editing: editingAddr === cell.addr }"
                :style="{ '--intensity': (cell.val / 255) }" @click="beginEdit(cell.addr)"
                @mouseenter="showHistory(cell.addr, $event)" @mouseleave="hideHistory">
                <span class="cell-addr">{{ fmtHex(cell.addr, 2) }}</span>
                <span v-if="editingAddr !== cell.addr" class="cell-val">{{ fmtHex(cell.val, 2) }} <span class="cell-chr">{{ toChar(cell.val) }}</span></span>
                <input v-else ref="editInput" class="cell-input" :value="editText" maxlength="2" spellcheck="false"
                    @input="onInput" @keydown.enter="commitEdit" @keydown.escape="cancelEdit" @blur="commitEdit" />
            </div>
        </div>
        <div v-if="hoveredAddr !== null" class="history-popover" :style="popoverStyle">
            <div class="history-title">{{ fmtHex(hoveredAddr, 2) }}</div>
            <div class="history-header-row">
                <span class="hdr-step">#</span>
                <span class="hdr-pc">PC</span>
                <span class="hdr-val">VAL</span>
                <span class="hdr-chr">C</span>
            </div>
            <div class="history-entries">
                <div v-for="(e, i) in historyForHovered" :key="i" class="history-entry">
                    <span class="hist-step">{{ e.step }}</span>
                    <span class="hist-pc">{{ fmtHex(e.pc, 4) }}</span>
                    <span class="hist-val">{{ fmtHex(e.val, 2) }}</span>
                    <span class="hist-chr">{{ toChar(e.val) }}</span>
                </div>
                <div v-if="historyForHovered.length === 0" class="history-empty">no history</div>
            </div>
        </div>
        <div class="io-dots">
            <button
                v-for="p in PAGE_COUNT" :key="p"
                class="io-dot"
                :class="{ active: page === p - 1 }"
                :title="`page ${p} — ${fmtHex((p - 1) * CELLS_PER_PAGE, 2)}–${fmtHex(p * CELLS_PER_PAGE - 1, 2)}`"
                @click="page = p - 1"
            >{{ p }}</button>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { computed, nextTick, ref } from 'vue';

export interface HistoryEntry {
    step: number;
    pc: number;
    val: number;
}

const CELLS_PER_PAGE = 32;
const PAGE_COUNT = 8;

const props = defineProps<{
    modelValue: number[];
    portHistory?: Record<number, HistoryEntry[]>;
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', val: number[]): void;
}>();

const page = ref(0);
const editingAddr = ref<number | null>(null);
const editText = ref('');
const editInput = ref<HTMLInputElement | null>(null);
const hoveredAddr = ref<number | null>(null);
const popoverPos = ref({ x: 0, y: 0 });

const base = computed(() => page.value * CELLS_PER_PAGE);

const cells = computed(() => {
    const start = base.value;
    const result: { addr: number; val: number }[] = [];
    for (let i = 0; i < CELLS_PER_PAGE; i++) {
        const addr = start + i;
        result.push({ addr, val: props.modelValue[addr] ?? 0 });
    }
    return result;
});

const popoverStyle = computed(() => ({
    left: popoverPos.value.x + 'px',
    top: (popoverPos.value.y - 10) + 'px',
}));

const historyForHovered = computed(() => {
    if (hoveredAddr.value === null || !props.portHistory) return [];
    return props.portHistory[hoveredAddr.value] ?? [];
});

function showHistory(addr: number, e: MouseEvent) {
    hoveredAddr.value = addr;
    popoverPos.value = { x: e.clientX, y: e.clientY };
}

function hideHistory() {
    hoveredAddr.value = null;
}

function toChar(n: number): string {
    if (n === 0) return '·';
    if (n < 0x20 || n === 0x7F) return '·';
    if (n > 0x7F) return '·';
    return String.fromCharCode(n);
}

function fmtHex(n: number, width: number): string {
    return n.toString(16).toUpperCase().padStart(width, '0');
}

function pagePrev() { if (page.value > 0) page.value--; }
function pageNext() { if (page.value < PAGE_COUNT - 1) page.value++; }

function onInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 2);
    editText.value = raw.toUpperCase();
}

function beginEdit(addr: number) {
    editingAddr.value = addr;
    editText.value = fmtHex(props.modelValue[addr] ?? 0, 2);
    nextTick(() => {
        const el = Array.isArray(editInput.value) ? editInput.value[0] : editInput.value;
        el?.focus();
        el?.select();
    });
}

function clearPage() {
    const next = [...props.modelValue];
    const start = base.value;
    for (let i = 0; i < CELLS_PER_PAGE; i++) {
        next[start + i] = 0;
    }
    emit('update:modelValue', next);
}

function cancelEdit() {
    editingAddr.value = null;
    editText.value = '';
}

function commitEdit() {
    if (editingAddr.value === null) return;
    const addr = editingAddr.value;
    const hex = editText.value || '0';
    const val = parseInt(hex, 16);
    if (!isNaN(val) && val >= 0 && val <= 0xFF) {
        const next = [...props.modelValue];
        next[addr] = val;
        emit('update:modelValue', next);
    }
    editingAddr.value = null;
    editText.value = '';
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.io-grid-outer {
    margin-bottom: 0.5rem;
    user-select: none;
}

.io-nav {
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

.clear-btn {
    margin-left: 0.5rem;

    &:hover:not(:disabled) {
        color: #e0556a;
        border-color: #e0556a;
    }
}

.page-indicator {
    font-family: var(--mono);
    font-size: 0.55rem;
    color: var(--text-muted);
}

.port-range {
    margin-left: auto;
    font-size: 0.55rem;
    color: var(--text-muted);
    font-family: var(--mono);
}

.io-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 2px;
}

.io-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.2rem 0;
    border-width: calc(1px + var(--intensity) * 0.5px);
    border-style: solid;
    border-color: color-mix(in srgb, var(--accent) calc(var(--intensity) * 100%), var(--border));
    border-radius: 3px;
    font-family: var(--mono);
    font-size: 0.55rem;
    color: color-mix(in srgb, var(--accent) calc(var(--intensity) * 100%), var(--text-muted));
    cursor: pointer;
    transition: background-color 0.15s, border-color 0.15s, color 0.15s;

    &:hover {
        border-color: color-mix(in srgb, var(--accent) calc(var(--intensity) * 100% + 20%), var(--text-muted));
    }

    &.editing {
        border-color: var(--accent);
        background-color: var(--surface-1);
    }
}

.cell-addr {
    font-size: 0.45rem;
    opacity: 0.7;
}

.cell-val {
    font-size: 0.7rem;
}

.cell-chr {
    font-size: 0.55rem;
    opacity: 0.55;
    margin-left: 0.15rem;
}

.cell-input {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.7rem;
    text-align: center;
    padding: 0;
}

.history-popover {
    position: fixed;
    z-index: 999;
    transform: translate(-50%, -100%);
    background: var(--surface-0);
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 0.3rem 0.4rem;
    min-width: 7rem;
    pointer-events: none;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    max-height: 12rem;
    overflow-y: auto;
}

.history-title {
    font-family: var(--mono);
    font-size: 0.5rem;
    color: var(--accent);
    margin-bottom: 0.2rem;
    text-align: center;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.15rem;
}

.history-header-row {
    display: flex;
    gap: 0.3rem;
    font-family: var(--mono);
    font-size: 0.42rem;
    color: var(--text-muted);
    opacity: 0.6;
    margin-bottom: 0.15rem;
    padding-bottom: 0.1rem;
    border-bottom: 1px solid var(--border);
}

.hdr-step { min-width: 1.6rem; text-align: right; }
.hdr-pc  { min-width: 2.0rem; text-align: right; }
.hdr-val { min-width: 1.2rem; text-align: right; }
.hdr-chr { min-width: 0.6rem; text-align: center; }

.history-entries {
    display: flex;
    flex-direction: column;
    gap: 1px;
}

.history-entry {
    display: flex;
    gap: 0.3rem;
    font-family: var(--mono);
    font-size: 0.48rem;
}

.hist-step { min-width: 1.6rem; text-align: right; color: var(--accent); }
.hist-pc  { min-width: 2.0rem; text-align: right; color: var(--text-muted); }
.hist-val { min-width: 1.2rem; text-align: right; color: var(--text); }
.hist-chr { min-width: 0.6rem; text-align: center; color: var(--text-muted); opacity: 0.6; }

.history-empty {
    font-family: var(--mono);
    font-size: 0.45rem;
    color: var(--text-muted);
    opacity: 0.5;
    text-align: center;
}

.io-dots {
    display: flex;
    justify-content: center;
    gap: 0.4rem;
    margin-top: 0.35rem;
}

.io-dot {
    width: 1.1rem;
    height: 1.1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--mono);
    font-size: 0.5rem;
    color: var(--text-muted);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s, background-color 0.15s;

    &:hover {
        border-color: var(--text-muted);
        color: var(--text);
    }

    &.active {
        border-color: var(--accent);
        color: var(--accent);
        background-color: var(--surface-1);
    }
}
</style>
