<template>
    <div class="row">
        <span class="bytes-label"><span class="addr-label">{{ instr.addr }}</span>{{ instr.bytes }}</span>
        <span class="disasm-label">{{ instr.disasm }}</span>
    </div>
</template>

<script setup lang="ts">
import { type InstrHistoryEntry } from '@/pages/lang/Lite80Page.vue';
const props = defineProps<{ instr: InstrHistoryEntry }>()
</script>

<style lang="scss" scoped>
.disasm-label {
    margin-left: .4rem;
    font-family: var(--mono);
    font-size: 0.65rem;
    color: var(--accent);
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
}

.addr-label {
    font-weight: 700;
    color: var(--accent);
    margin-right: .4rem;
}

.bytes-label {
    font-family: var(--mono);
    font-size: 0.6rem;
    color: var(--text-muted);
    opacity: 0.6;
}
</style>