<template>
    <div class="regs-card">
        <div class="regs-top">
            <div class="regs-grid">
                <div class="reg-cell acc" :class="{ editing: editing === 7 }" @click="beginEdit(7)">
                    <span class="reg-name">A</span>
                    <span v-if="editing !== 7" class="reg-val">{{ fmtHex(values[7] ?? 0, 2) }}</span>
                    <input v-else ref="editInputs" class="reg-input" :value="editText" maxlength="2" spellcheck="false"
                        @input="onInput" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit"
                        @blur="commitEdit" />
                </div>
                <div v-for="pair in pairs" :key="pair.hi" class="reg-pair">
                    <div class="reg-cell" :class="{ editing: editing === pair.hiCode }" @click="beginEdit(pair.hiCode)">
                        <span class="reg-name">{{ pair.hi }}</span>
                        <span v-if="editing !== pair.hiCode" class="reg-val">{{ fmtHex(values[pair.hiCode] ?? 0, 2)
                            }}</span>
                        <input v-else ref="editInputs" class="reg-input" :value="editText" maxlength="2"
                            spellcheck="false" @input="onInput" @keydown.enter.prevent="commitEdit"
                            @keydown.escape="cancelEdit" @blur="commitEdit" />
                    </div>
                    <div class="reg-cell" :class="{ editing: editing === pair.loCode }" @click="beginEdit(pair.loCode)">
                        <span class="reg-name">{{ pair.lo }}</span>
                        <span v-if="editing !== pair.loCode" class="reg-val">{{ fmtHex(values[pair.loCode] ?? 0, 2)
                            }}</span>
                        <input v-else ref="editInputs" class="reg-input" :value="editText" maxlength="2"
                            spellcheck="false" @input="onInput" @keydown.enter.prevent="commitEdit"
                            @keydown.escape="cancelEdit" @blur="commitEdit" />
                    </div>
                </div>
            </div>
            <div class="flags">
                <button v-for="f in flagBits" :key="f.bit" class="flag-chip" :class="{ set: flagSet(f.bit) }"
                    @click.stop="toggleFlag(f.bit)">
                    {{ f.label }}
                </button>
                <button class="flag-chip int-chip" :class="{ set: intEnabled }"
                    @click.stop="emit('update:int-enabled', !intEnabled)">
                    INT
                </button>
                <span class="flag-hex-label" :class="{ editing: editing === 6 }" @click.stop="beginEdit(6)">
                    <span v-if="editing !== 6">F:{{ fmtHex(values[6] ?? 0, 2) }}</span>
                    <input v-else class="flag-input" :value="editText" maxlength="2" spellcheck="false" @input="onInput"
                        @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit" @blur="commitEdit" />
                </span>
            </div>
        </div>
        <div class="ptrs">
            <div class="ptr-row">
                <button class="ptr-step" @click="emit('update:pc', (pc - 1) & 0xFFFF)">−</button>
                <span class="ptr-label">PC</span>
                <span v-if="editing !== 'pc'" class="ptr-val clickable" @click="beginEdit('pc', pc)">{{ fmtHex(pc, 4)
                    }}</span>
                <input v-else ref="editInputPc" class="ptr-input" :value="editText" maxlength="4" spellcheck="false"
                    @input="onInputWide" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit" />
                <button class="ptr-step right" @click="emit('update:pc', (pc + 1) & 0xFFFF)">+</button>
            </div>
            <div class="ptr-row">
                <button class="ptr-step" @click="emit('update:sp', (sp - 1) & 0xFFFF)">−</button>
                <span class="ptr-label">SP</span>
                <span v-if="editing !== 'sp'" class="ptr-val clickable" @click="beginEdit('sp', sp)">{{ fmtHex(sp, 4)
                    }}</span>
                <input v-else ref="editInputSp" class="ptr-input" :value="editText" maxlength="4" spellcheck="false"
                    @input="onInputWide" @keydown.enter.prevent="commitEdit" @keydown.escape="cancelEdit" />
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
    (e: 'update:reg', idx: number, val: number): void;
    (e: 'update:int-enabled', val: boolean): void;
}>();

function toggleFlag(bit: number) {
    const f = props.values[6] ?? 0;
    emit('update:reg', 6, f ^ (1 << bit));
}

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

const editing = ref<number | string | null>(null);
const editText = ref('');
const editInputs = ref<HTMLInputElement | HTMLInputElement[] | null>(null);
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
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 2);
    editText.value = raw.toUpperCase();
}

function onInputWide(e: Event) {
    const raw = (e.target as HTMLInputElement).value.replace(/[^0-9a-fA-F]/g, '').slice(0, 4);
    editText.value = raw.toUpperCase();
}

function beginEdit(field: number | string, current?: number) {
    editing.value = field;
    if (typeof field === 'number') {
        editText.value = fmtHex(props.values[field] ?? 0, 2);
    } else {
        editText.value = fmtHex(current ?? 0, 4);
    }
    nextTick(() => {
        const els = Array.isArray(editInputs.value) ? editInputs.value : [editInputs.value];
        const el = els[els.length - 1] as HTMLInputElement | undefined;
        const pcEl = editInputPc.value;
        const spEl = editInputSp.value;
        const target = typeof field === 'number' ? el : field === 'pc' ? pcEl : spEl;
        target?.focus();
        target?.select();
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
    if (typeof field === 'number') {
        const val = parseInt(hex, 16);
        if (!isNaN(val) && val >= 0 && val <= 0xFF) {
            emit('update:reg', field, val);
        }
    } else {
        const val = parseInt(hex, 16);
        if (!isNaN(val) && val >= 0 && val <= 0xFFFF) {
            if (field === 'pc') emit('update:pc', val);
            else emit('update:sp', val);
        }
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
    gap: 3px;
    flex: 1;
}

.reg-pair {
    display: flex;
    flex-direction: column;
    gap: 3px;
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
    cursor: pointer;
    transition: background-color 0.15s, border-color 0.15s, color 0.15s;

    &:hover {
        border-color: var(--text-muted);
    }

    &.acc {
        grid-row: span 2;
        color: var(--text);

        .reg-val {
            font-size: 1rem;
        }
    }

    &.editing {
        border-color: var(--accent);
        background-color: var(--surface-1);
    }
}

.reg-name {
    font-size: 0.45rem;
    opacity: 0.7;
}

.reg-val {
    font-size: 0.7rem;
}

.reg-input {
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
    background: none;
    color: var(--text-muted);
    opacity: 0.35;
    cursor: pointer;
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

.flag-hex-label {
    margin-top: 2px;
    font-family: var(--mono);
    font-size: 0.45rem;
    color: var(--text-muted);
    opacity: 0.6;
    text-align: center;
    cursor: pointer;

    &.editing {
        opacity: 1;
    }
}

.flag-input {
    width: 2.5rem;
    background: transparent;
    border: none;
    outline: none;
    color: var(--accent);
    font-family: var(--mono);
    font-size: 0.65rem;
    text-align: center;
    padding: 0;
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
    width: 100%;
    padding: 0;
}
</style>
