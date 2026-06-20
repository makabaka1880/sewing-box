<template>
    <div class="mem-view">
        <div class="mem-toolbar">
            <label class="jump-label">Addr:</label>
            <span class="jump-prefix">0x</span>
            <input class="jump-input" :value="jumpText" maxlength="4" spellcheck="false" @input="onJumpInput"
                @keydown.enter.prevent="doJump" @focus="onJumpFocus" placeholder="0000" />
            <button class="nav-btn" @click="doJump">Go</button>
            <button class="nav-btn" @click="scrollToAddress(pc)">Go to PC</button>
            <button class="nav-btn" @click="scrollToAddress(sp)">Go to SP</button>
        </div>
        <div class="mem-body">
            <div ref="hexContainer" class="hex-container" @scroll="onHexScroll">
                <div class="hex-spacer" :style="{ height: topSpacer + 'px' }" />
                <div v-for="row in visibleRows" :key="row.baseAddr" class="hex-row" :class="rowClass(row)">
                    <span class="hex-addr">{{ fmtHex(row.baseAddr, 4) }}</span>
                    <span class="hex-colon">:</span>
                    <template v-for="(b, i) in row.bytes" :key="i">
                        <span v-if="i === 8" class="hex-mid-gap" />
                        <template v-if="editingAddr === row.baseAddr + i">
                            <input ref="editInputs" class="hex-input" :value="editText" maxlength="2" spellcheck="false"
                                @input="onEditInput" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit"
                                @blur="commitEdit" />
                        </template>
                        <span v-else class="hex-byte" :class="byteClass(row.baseAddr + i, b)"
                            @click="beginEdit(row.baseAddr + i, b)">
                            {{ fmtHex(b, 2) }}
                        </span>
                    </template>
                    <span class="hex-ascii-sep">│</span>
                    <span class="hex-ascii">
                        <span v-for="(b, i) in row.bytes" :key="i" class="ascii-char"
                            :class="{ dim: shouldDimAscii(b) }">{{ toAscii(b)
                            }}</span>
                    </span>
                </div>
                <div class="hex-spacer" :style="{ height: bottomSpacer + 'px' }" />
            </div>
            <div ref="minimapWrap" class="minimap-wrap" @scroll="paintMinimap">
                <canvas ref="minimapCanvas" class="minimap-canvas" @click="onMinimapClick" />
            </div>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

const props = defineProps<{
    memory: Uint8Array;
    pc: number;
    sp: number;
}>();

const emit = defineEmits<{
    (e: 'edit-byte', addr: number, value: number): void;
}>();

const BYTES_PER_ROW = 16;
const TOTAL_ROWS = 65536 / BYTES_PER_ROW;
const OVERSCAN = 20;

// ── Row height measurement ──

const ROW_HEIGHT_PX = ref(19); // fallback default — measured on mount

function measureRowHeight() {
    const test = document.createElement('div');
    test.style.cssText = 'line-height:1.2rem;position:absolute;visibility:hidden;pointer-events:none;font-size:0.6rem;font-family:monospace';
    test.textContent = 'X';
    document.body.appendChild(test);
    const h = test.getBoundingClientRect().height;
    document.body.removeChild(test);
    if (h > 0) ROW_HEIGHT_PX.value = h;
}

// ── Viewport state ──

const hexContainer = ref<HTMLDivElement | null>(null);
const viewportTop = ref(0);
const viewportHeight = ref(0);
const minimapSize = ref(0); // square side = hex container height

let resizeObs: ResizeObserver | null = null;

function updateViewport() {
    const el = hexContainer.value;
    if (!el) return;
    viewportTop.value = el.scrollTop;
    viewportHeight.value = el.clientHeight;
    minimapSize.value = el.clientHeight;
}

function onHexScroll() {
    updateViewport();
}

// ── Visible row range ──

const startRow = computed(() => {
    if (ROW_HEIGHT_PX.value <= 0) return 0;
    return Math.floor(viewportTop.value / ROW_HEIGHT_PX.value);
});

const visibleCount = computed(() => {
    if (ROW_HEIGHT_PX.value <= 0) return 0;
    return Math.ceil(viewportHeight.value / ROW_HEIGHT_PX.value);
});

const firstRenderRow = computed(() => Math.max(0, startRow.value - OVERSCAN));
const lastRenderRow = computed(() => Math.min(TOTAL_ROWS - 1, startRow.value + visibleCount.value + OVERSCAN));

const topSpacer = computed(() => firstRenderRow.value * ROW_HEIGHT_PX.value);
const bottomSpacer = computed(() => (TOTAL_ROWS - 1 - lastRenderRow.value) * ROW_HEIGHT_PX.value);

// ── Visible rows ──

interface MemRow {
    baseAddr: number;
    bytes: number[];
    hasPC: boolean;
    hasSP: boolean;
    pcOffset: number;
    spOffset: number;
}

const visibleRows = computed<MemRow[]>(() => {
    const mem = props.memory;
    const pc = props.pc;
    const sp = props.sp;
    const first = firstRenderRow.value;
    const last = lastRenderRow.value;
    const result: MemRow[] = [];
    for (let i = first; i <= last; i++) {
        const baseAddr = i * BYTES_PER_ROW;
        const bytes: number[] = [];
        for (let j = 0; j < BYTES_PER_ROW; j++) {
            bytes.push(mem[baseAddr + j] ?? 0);
        }
        const hasPC = pc >= baseAddr && pc < baseAddr + BYTES_PER_ROW;
        const hasSP = sp >= baseAddr && sp < baseAddr + BYTES_PER_ROW;
        result.push({
            baseAddr,
            bytes,
            hasPC,
            hasSP,
            pcOffset: hasPC ? pc - baseAddr : -1,
            spOffset: hasSP ? sp - baseAddr : -1,
        });
    }
    return result;
});

function rowClass(row: MemRow) {
    return {
        'pc-row': row.hasPC,
        'sp-row': row.hasSP && !row.hasPC,
    };
}

function byteClass(addr: number, _value: number) {
    return {
        zero: _value === 0,
        'pc-byte': addr === props.pc,
        'sp-byte': addr === props.sp && addr !== props.pc,
    };
}

// ── Formatting helpers ──

function fmtHex(n: number, width: number): string {
    return n.toString(16).toUpperCase().padStart(width, '0');
}

function toAscii(n: number): string {
    if (n === 0) return '·';
    if (n < 0x20 || n === 0x7F) return '·';
    if (n > 0x7E) return '·';
    return String.fromCharCode(n);
}

function shouldDimAscii(n: number): boolean {
    return n === 0 || n < 0x20 || n === 0x7F || n > 0x7E;
}

// ── Jump toolbar ──

const jumpText = ref('');

function onJumpInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 4);
    jumpText.value = raw.toUpperCase();
}

function onJumpFocus() {
    jumpText.value = '';
}

function doJump() {
    const hex = jumpText.value || '0';
    const addr = parseInt(hex, 16);
    if (!isNaN(addr) && addr >= 0 && addr <= 0xFFFF) {
        scrollToAddress(addr);
    }
}

function scrollToAddress(addr: number) {
    const container = hexContainer.value;
    if (!container || ROW_HEIGHT_PX.value <= 0) return;
    const rowIdx = Math.floor(addr / BYTES_PER_ROW);
    const target = rowIdx * ROW_HEIGHT_PX.value - container.clientHeight / 2;
    container.scrollTo({ top: Math.max(0, target), behavior: 'smooth' });
    jumpText.value = '';
}

// ── Editing ──

const editingAddr = ref<number | null>(null);
const editText = ref('');
const editInputs = ref<HTMLInputElement | HTMLInputElement[] | null>(null);

function beginEdit(addr: number, currentValue: number) {
    editingAddr.value = addr;
    editText.value = fmtHex(currentValue, 2);
    nextTick(() => {
        const els = Array.isArray(editInputs.value) ? editInputs.value : [editInputs.value];
        const el = els[els.length - 1] as HTMLInputElement | undefined;
        el?.focus();
        el?.select();
    });
}

function onEditInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 2);
    editText.value = raw.toUpperCase();
}

function commitEdit() {
    if (editingAddr.value === null) return;
    const addr = editingAddr.value;
    const hex = editText.value || '0';
    const val = parseInt(hex, 16);
    if (!isNaN(val) && val >= 0 && val <= 0xFF) {
        emit('edit-byte', addr, val);
    }
    // Defer DOM cleanup so the <input> survives until Vue
    // finishes processing the parent's memory update.
    nextTick(() => {
        editingAddr.value = null;
        editText.value = '';
    });
}

function cancelEdit() {
    editingAddr.value = null;
    editText.value = '';
}

// ── Minimap ──

const HEATMAP_W = 64; // logical heatmap grid width
const HEATMAP_H = 1024; // logical heatmap grid height
const LABEL_W = 24; // px reserved for address labels on right

const minimapCanvas = ref<HTMLCanvasElement | null>(null);
const minimapWrap = ref<HTMLDivElement | null>(null);
let minimapDpr = 1;
let heapmapData: ImageData | null = null; // cached full heatmap

function byteToColor(v: number): [number, number, number] {
    if (v === 0) return [6, 6, 10];
    const t = v / 255;
    const r = Math.floor(10 + Math.max(0, (t - 0.35) / 0.65) * 200);
    const g = Math.floor(10 + Math.sin(t * Math.PI) * 160);
    const b = Math.floor(95 + Math.max(0, (0.55 - t) / 0.55) * 145);
    return [Math.min(255, r), Math.min(255, g), Math.min(255, b)];
}

function buildHeapmap() {
    // Build the full heatmap ImageData once — memory stable until changed
    const data = new Uint8ClampedArray(HEATMAP_W * HEATMAP_H * 4);
    const mem = props.memory;
    for (let i = 0; i < 65536; i++) {
        const val = mem[i] ?? 0;
        const [r, g, b] = byteToColor(val);
        const idx = i * 4;
        data[idx] = r;
        data[idx + 1] = g;
        data[idx + 2] = b;
        data[idx + 3] = 255;
    }
    heapmapData = new ImageData(data, HEATMAP_W, HEATMAP_H);
}

function paintMinimap() {
    const canvas = minimapCanvas.value;
    const wrap = minimapWrap.value;
    if (!canvas || !wrap) return;

    const cw = wrap.clientWidth; // content area of square wrapper
    if (cw <= 0) return;
    const hmW = cw - LABEL_W; // heatmap rendered width
    if (hmW <= 0) return;
    const hmH = hmW * (HEATMAP_H / HEATMAP_W); // proportional heatmap height

    const dpr = window.devicePixelRatio || 1;
    if (canvas.width !== cw * dpr || canvas.height !== hmH * dpr || minimapDpr !== dpr) {
        minimapDpr = dpr;
        canvas.width = cw * dpr;
        canvas.height = hmH * dpr;
        canvas.style.width = cw + 'px';
        canvas.style.height = hmH + 'px';
    }

    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    if (heapmapData) {
        ctx.clearRect(0, 0, cw, hmH);
        // Synchronous offscreen canvas for crisp pixel-perfect upscale
        const off = document.createElement('canvas');
        off.width = HEATMAP_W;
        off.height = HEATMAP_H;
        off.getContext('2d')!.putImageData(heapmapData, 0, 0);
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(off, 0, 0, HEATMAP_W, HEATMAP_H, 0, 0, hmW, hmH);
    }

    paintOverlays(ctx, hmW, hmH);
}

function paintOverlays(ctx: CanvasRenderingContext2D, hmW: number, hmH: number) {
    const bytesPerRow = HEATMAP_W;
    const sx = hmW / HEATMAP_W;
    const sy = hmH / HEATMAP_H;

    // ── Address labels (right side) ──
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--text-muted').trim() || '#888';
    ctx.font = `5.5px monospace`;
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    const LABEL_STEP = 0x0400; // every 1K
    for (let addr = 0; addr < 0x10000; addr += LABEL_STEP) {
        const y = (addr / bytesPerRow) * sy;
        if (y > hmH) break;
        const label = addr.toString(16).toUpperCase().padStart(4, '0');
        ctx.fillText(label, hmW + 2, y);
    }

    // ── Vertical divider ──
    ctx.strokeStyle = 'rgba(255,255,255,0.12)';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    ctx.moveTo(hmW, 0);
    ctx.lineTo(hmW, hmH);
    ctx.stroke();

    // Viewport highlight (rendered area)
    const vrY0 = (firstRenderRow.value * BYTES_PER_ROW) / bytesPerRow * sy;
    const vrY1 = ((lastRenderRow.value + 1) * BYTES_PER_ROW) / bytesPerRow * sy;
    ctx.fillStyle = 'rgba(255,255,255,0.08)';
    ctx.fillRect(0, vrY0, hmW, vrY1 - vrY0);

    // Viewport outline (visible area without overscan)
    const vpY0 = (startRow.value * BYTES_PER_ROW) / bytesPerRow * sy;
    const vpY1 = Math.min(((startRow.value + visibleCount.value) * BYTES_PER_ROW) / bytesPerRow * sy, hmH);
    ctx.strokeStyle = 'rgba(255,255,255,0.4)';
    ctx.lineWidth = 1;
    ctx.strokeRect(0.5, vpY0 + 0.5, hmW - 1, Math.max(1, vpY1 - vpY0 - 1));

    // ── PC highlight row + dot ──
    const pcAddr = props.pc & 0xFFFF;
    const pcX = (pcAddr % bytesPerRow) * sx;
    const pcY = Math.floor(pcAddr / bytesPerRow) * sy;
    const pcH = Math.max(1, sy);
    ctx.fillStyle = 'rgba(100,200,255,0.30)';
    ctx.fillRect(0, pcY, hmW, pcH);
    ctx.fillStyle = '#6cf';
    ctx.fillRect(pcX - 1.5, pcY + pcH / 2 - 1.5, 3, 3);

    // ── SP highlight row + dot ──
    const spAddr = props.sp & 0xFFFF;
    const spX = (spAddr % bytesPerRow) * sx;
    const spY = Math.floor(spAddr / bytesPerRow) * sy;
    const spH = Math.max(1, sy);
    ctx.fillStyle = 'rgba(255,153,0,0.25)';
    ctx.fillRect(0, spY, hmW, spH);
    ctx.fillStyle = '#f90';
    ctx.fillRect(spX - 1.5, spY + spH / 2 - 1.5, 3, 3);
}

function onMinimapClick(e: MouseEvent) {
    const canvas = minimapCanvas.value;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const hmW = rect.width - LABEL_W;
    if (hmW <= 0) return;
    const xCss = e.clientX - rect.left;
    const yCss = e.clientY - rect.top;
    if (xCss >= hmW) return;
    const x = Math.floor(xCss / hmW * HEATMAP_W);
    const y = Math.floor(yCss / rect.height * HEATMAP_H);
    if (x < 0 || x >= HEATMAP_W || y < 0 || y >= HEATMAP_H) return;
    scrollToAddress(y * HEATMAP_W + x);
}

// Re-draw viewport highlight on scroll; full heatmap on memory change
watch([startRow, visibleCount, firstRenderRow, lastRenderRow, () => props.pc, () => props.sp], () => {
    paintMinimap();
});

watch(() => props.memory, () => {
    buildHeapmap();
    paintMinimap();
});

watch(minimapSize, () => {
    paintMinimap();
});

// ── Lifecycle ──

onMounted(() => {
    measureRowHeight();
    const el = hexContainer.value;
    if (el) {
        updateViewport();
        resizeObs = new ResizeObserver(() => updateViewport());
        resizeObs.observe(el);
    }
    buildHeapmap();
    paintMinimap();
});

onUnmounted(() => {
    resizeObs?.disconnect();
});
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.mem-view {
    display: flex;
    flex-direction: column;
    user-select: none;
}

// ── Toolbar ──

.mem-toolbar {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    margin-bottom: 0.4rem;
    flex-wrap: wrap;
}

.jump-label {
    font-family: var(--mono);
    font-size: 0.55rem;
    color: var(--text-muted);
}

.jump-prefix {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-muted);
    opacity: 0.6;
}

.jump-input {
    width: 3.2rem;
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.6rem;
    text-align: left;
    padding: 0.1rem 0.3rem;
    outline: none;

    &:focus {
        border-color: var(--accent);
    }

    &::placeholder {
        color: var(--text-disabled);
        opacity: 0.5;
    }
}

.nav-btn {
    background: var(--surface-0);
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 0.1rem 0.4rem;
    font-family: var(--mono);
    font-size: 0.5rem;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;

    &:hover {
        color: var(--text);
        border-color: var(--text-muted);
    }
}

// ── Layout body ──

.mem-body {
    display: flex;
    gap: 0.25rem;
    min-height: 0;
    min-width: 0;
}

// ── Hex rows ──

.hex-container {
    flex-shrink: 0;
    max-width: 100%;
    min-width: 0;
    overflow: auto;
    min-height: 12rem;
    max-height: 40vh;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--mono);
    font-size: 0.6rem;
    line-height: 1.2rem;
}

.hex-row {
    display: flex;
    align-items: center;
    padding: 0 0.3rem;
    white-space: nowrap;
    line-height: 1.2rem;

    &.pc-row {
        background: color-mix(in srgb, var(--accent) 12%, transparent);
    }

    &.sp-row {
        background: color-mix(in srgb, oklch(0.7 0.15 25) 8%, transparent);
    }
}

.hex-addr {
    color: var(--text-muted);
    opacity: 0.55;
    min-width: 4.2ch;
    text-align: right;
}

.hex-colon {
    color: var(--text-muted);
    opacity: 0.3;
    margin-right: 0.35rem;
}

.hex-byte {
    display: inline-block;
    width: 2.2ch;
    text-align: center;
    cursor: pointer;
    border-radius: 2px;
    color: var(--text);

    &:hover {
        background: var(--surface-1);
        color: var(--accent);
    }

    &.zero {
        opacity: 0.35;
    }

    &.pc-byte {
        background: var(--accent);
        color: var(--surface-0);
        border-radius: 2px;
        font-weight: 700;

        &:hover {
            background: var(--accent-hover);
            color: var(--surface-0);
        }

        &.zero {
            opacity: 0.7;
        }
    }

    &.sp-byte {
        outline: 1px solid color-mix(in srgb, oklch(0.7 0.15 25) 60%, transparent);
        outline-offset: -1px;
        border-radius: 2px;

        &.zero {
            opacity: 1;
        }
    }
}

.hex-mid-gap {
    display: inline-block;
    width: 0.6ch;
}

.hex-ascii-sep {
    color: var(--text-muted);
    opacity: 0.25;
    margin: 0 0.35rem 0 0.5rem;
}

.hex-ascii {
    opacity: 0.5;
    color: var(--text-muted);
}

.ascii-char {
    display: inline-block;
    width: 2.2ch;
    text-align: center;

    &.dim {
        opacity: 0.35;
    }
}

.hex-input {
    width: 2.2ch;
    background: var(--surface-1);
    border: 1px solid var(--accent);
    border-radius: 2px;
    outline: none;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.6rem;
    text-align: center;
    padding: 0;
    line-height: 1rem;
}

// ── Minimap ──

.minimap-wrap {
    flex: 1;
    min-width: 0;
    align-self: stretch;
    aspect-ratio: 1;
    max-height: 40vh;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 3px;
    cursor: crosshair;

    @media (max-width: 720px) {
        display: none;
    }
}

.minimap-canvas {
    display: block;
    image-rendering: pixelated;
}
</style>
