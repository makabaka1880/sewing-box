<template>
    <div class="tree-node" :class="{ branch: isBranch }">
        <template v-if="isAtom">
            <span class="tree-atom">{{ label }}</span>
        </template>
        <template v-else>
            <div class="tree-branch-label">{{ label }}</div>
            <div class="tree-children">
                <TreeCard v-for="(child, i) in children" :key="i" :node="child" />
            </div>
        </template>
    </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue';

type TreeNode =
    | { Atom: string }
    | { Branch: [string, TreeNode[]] };

const props = defineProps<{ node: TreeNode }>();

const isAtom = computed(() => 'Atom' in props.node);
const isBranch = computed(() => 'Branch' in props.node);
const label = computed(() =>
    isAtom.value
        ? (props.node as { Atom: string }).Atom
        : (props.node as { Branch: [string, TreeNode[]] }).Branch[0]
);
const children = computed((): TreeNode[] =>
    isBranch.value ? (props.node as { Branch: [string, TreeNode[]] }).Branch[1] : []
);
</script>

<style lang="scss" scoped>
.tree-node {
    font-family: var(--mono);
    font-size: 0.7rem;
}

.tree-atom {
    display: inline-block;
    padding: 0.1rem 0.35rem;
    border: 1px solid var(--border);
    border-radius: 3px;
    margin: 0.1rem 0;
}

.tree-branch-label {
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 0.15rem;
}

.tree-children {
    border-left: 1px solid var(--border);
    padding-left: 0.6rem;
    margin-left: 0.2rem;
}
</style>
