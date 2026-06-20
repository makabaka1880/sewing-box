<template>
    <div class="regs-card">
        <div class="regs-top">
            <div class="regs-grid">
                <div class="reg-cell acc">
                    <span class="reg-name">A</span>
                    <span class="reg-val">{{ fmtHex(values[7] ?? 0, 2) }}</span>
                </div>
                <div v-for="pair in pairs" :key="pair.hi" class="reg-pair">
                    <div class="reg-cell">
                        <span class="reg-name">{{ pair.hi }}</span>
                        <span class="reg-val">{{ fmtHex(values[pair.hiCode] ?? 0, 2) }}</span>
                    </div>
                    <div class="reg-cell">
                        <span class="reg-name">{{ pair.lo }}</span>
                        <span class="reg-val">{{ fmtHex(values[pair.loCode] ?? 0, 2) }}</span>
                    </div>
                </div>
            </div>
            <div class="flags">
                <span v-for="f in flagBits" :key="f.bit" class="flag-chip" :class="{ set: flagSet(f.bit) }">
                    {{ f.label }}
                </span>
                <span class="flag-chip int-chip" :class="{ set: intEnabled }">INT</span>
            </div>
        </div>
        <div class="ptrs">
            <div class="ptr-row">
                <button class="ptr-step" @click="emit('update:pc', (pc - 1) & 0xFFFF)">−</button>
                <span class="ptr-label">PC</span>
                <span v-if="editing !== 'pc'" class="ptr-val clickable" @click="beginEdit('pc', pc)">{{ fmtHex(pc, 4)
                }}</span>
                <input v-else ref="editInputPc" class="ptr-input" :value="editText" maxlength="4" spellcheck="false"
                    @input="onInput" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit" />
                <button class="ptr-step right" @click="emit('update:pc', (pc + 1) & 0xFFFF)">+</button>
            </div>
            <div class="ptr-row">
                <button class="ptr-step" @click="emit('update:sp', (sp - 1) & 0xFFFF)">−</button>
                <span class="ptr-label">SP</span>
                <span v-if="editing !== 'sp'" class="ptr-val clickable" @click="beginEdit('sp', sp)">{{ fmtHex(sp, 4)
                }}</span>
                <input v-else ref="editInputSp" class="ptr-input" :value="editText" maxlength="4" spellcheck="false"
                    @input="onInput" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit" />
                <button class="ptr-step right" @click="emit('update:sp', (sp + 1) & 0xFFFF)">+</button>
            </div>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { nextTick, ref } from 'vue';

const props = defineProps<{
    values: number[];
    pc: number;
    sp: number;
    intEnabled: boolean;
}>();

const emit = defineEmits<{
    (e: 'update:pc', val: number): void;
    (e: 'update:sp', val: number): void;
}>();

const pairs = [
    { hi: 'B', hiCode: 0, lo: 'C', loCode: 1 },
    { hi: 'D', hiCode: 2, lo: 'E', loCode: 3 },
    { hi: 'H', hiCode: 4, lo: 'L', loCode: 5 },
];

const flagBits = [
    { label: 'S', bit: 7 },
    { label: 'Z', bit: 6 },
    { label: 'AC', bit: 4 },
    { label: 'P', bit: 2 },
    { label: 'C', bit: 0 },
];

const editing = ref<string | null>(null);
const editText = ref('');
const editInputPc = ref<HTMLInputElement | null>(null);
const editInputSp = ref<HTMLInputElement | null>(null);

function fmtHex(n: number, w: number): string {
    return n.toString(16).toUpperCase().padStart(w, '0');
}

function flagSet(bit: number): boolean {
    const f = props.values[6] ?? 0;
    return ((f >> bit) & 1) === 1;
}

function onInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 4);
    editText.value = raw.toUpperCase();
}

function beginEdit(field: string, current: number) {
    editing.value = field;
    editText.value = fmtHex(current, 4);
    nextTick(() => {
        const inp = field === 'pc' ? editInputPc.value : editInputSp.value;
        inp?.focus();
        inp?.select();
    });
}

function cancelEdit() {
    editing.value = null;
    editText.value = '';
}

function commitEdit() {
    if (editing.value === null) return;
    const field = editing.value;
    const hex = editText.value || '0';
    const val = parseInt(hex, 16);
    if (!isNaN(val) && val >= 0 && val <= 0xFFFF) {
        if (field === 'pc') emit('update:pc', val);
        else emit('update:sp', val);
    }
    editing.value = null;
    editText.value = '';
}
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

.regs-card {
    margin-bottom: 0.5rem;
    user-select: none;
}

.regs-top {
    display: flex;
    gap: 5px;
}

.regs-grid {
    display: grid;
    grid-template-columns: 1fr repeat(3, 1fr);
    gap: 2px;
    flex: 1;
}

.reg-pair {
    display: flex;
    flex-direction: column;
    gap: 2px;
    grid-row: span 2;

    >.reg-cell {
        flex: 1;
        min-height: 0;
    }
}

.reg-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 0.2rem 0;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--mono);
    color: var(--text-muted);
    transition: background-color 0.15s, border-color 0.15s, color 0.15s;

    &.acc {
        grid-row: span 2;
        // border-color: var(--border-strong);
        color: var(--text);

        .reg-val {
            font-size: 1rem;
        }
    }
}

.reg-name {
    font-size: 0.45rem;
    opacity: 0.7;
}

.reg-val {
    font-size: 0.7rem;
}

.flags {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
}

.flag-chip {
    text-align: center;
    font-family: var(--mono);
    font-size: 0.5rem;
    padding: 0.2rem 0.35rem;
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--text-muted);
    opacity: 0.35;
    transition: opacity 0.15s, border-color 0.15s, color 0.15s;

    &.set {
        opacity: 1;
        border-color: var(--accent);
        color: var(--accent);
    }
}

.int-chip {
    margin-top: 2px;
    border-color: var(--border-strong);

    &.set {
        border-color: var(--accent);
        color: var(--accent);
    }
}

.ptrs {
    margin-top: 5px;
    display: flex;
    gap: 5px;
}

.ptr-row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--mono);
    font-size: 0.6rem;
}

.ptr-label {
    color: var(--text-muted);
    font-size: 0.5rem;
    opacity: 0.7;
}

.ptr-step {
    background: none;
    border: none;
    color: var(--text-muted);
    font-family: var(--mono);
    font-size: 0.5rem;
    padding: 0 0.25rem;
    line-height: 1;
    cursor: pointer;
    transition: color 0.15s;

    &:hover {
        color: var(--accent);
    }

    &.right {
        margin-left: auto;
    }
}

.ptr-val {
    color: var(--accent);
    font-size: 0.7rem;

    &.clickable {
        cursor: pointer;

        &:hover {
            color: var(--text);
        }
    }
}

.ptr-input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.7rem;
    width: 100%;
    padding: 0;
}
</style>