<template>
    <div class="frame-card" :class="{ nf, error }">
        <div v-if="nf" class="frame-badge badge-nf">NF</div>
        <div v-if="error" class="frame-badge badge-error">ERR</div>
        <div class="frame-section">
            <div class="frame-label">SNAPSHOT</div>
            <span v-html="termHtml"></span>
        </div>
        <div class="frame-section">
            <div class="frame-label">Env</div>
            <span v-html="envHtml"></span>
        </div>
        <div class="frame-section">
            <div class="frame-label">Stack</div>
            <span v-html="stackHtml"></span>
        </div>
    </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue';
import katex from 'katex';
import 'katex/dist/katex.min.css';

type Term = {
    Var: string
} | {
    Lam: (string | Term)[]
} | {
    App: Term[]
}

type Closure = {
    term: Term;
    env: Environment;
}

type Environment =
    | 'Empty'
    | { Cons: [string, Closure, Environment] }

type Stack =
    | 'Empty'
    | { Cons: [Closure, Stack] }

function termToTex(term: Term): string {
    if ('Var' in term) {
        return term.Var;
    }
    if ('Lam' in term) {
        const [param, body] = term.Lam;
        return `\\lambda ${param}. ${termToTex(body as Term)}`;
    }
    const [M, N] = term.App;
    return `(${termToTex(M!)})\\;( ${termToTex(N!)})`;
}

function envToTex(env: Environment): string {
    if (env === 'Empty') return '\\varnothing';
    const [name, closure, rest] = env.Cons;
    const head = `\\textit{${name}} \\mapsto ${termToTex(closure.term)}`;
    if (rest === 'Empty') return `\\{ ${head} \\}`;
    return `\\{ ${head},\\; ${envToTex(rest).slice(2, -2)} \\}`;
}

function stackToTex(stack: Stack): string {
    if (stack === 'Empty') return '\\varnothing';
    const [closure, rest] = stack.Cons;
    const head = `\\langle ${termToTex(closure.term)} \\rangle`;
    return rest === 'Empty' ? head : `${head} \\mathrel{::} ${stackToTex(rest)}`;
}

const props = defineProps<{ config: { term: Term; env: Environment; stack: Stack }; nf?: boolean; error?: boolean }>()
const state = props.config;
const nf = props.nf ?? false;
const error = props.error ?? false;

const termHtml = computed(() =>
    katex.renderToString(termToTex(state.term), {
        displayMode: true,
        throwOnError: false,
    })
);

const envHtml = computed(() =>
    katex.renderToString(envToTex(state.env), {
        displayMode: false,
        throwOnError: false,
    })
);

const stackHtml = computed(() =>
    katex.renderToString(stackToTex(state.stack), {
        displayMode: false,
        throwOnError: false,
    })
);
</script>

<style lang="scss" scoped>
.frame-card {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.5rem;
    font-size: 0.75rem;
    position: relative;

    &.nf {
        border-color: var(--accent);
    }

    &.error {
        border-color: var(--error);
    }
}

.frame-badge {
    position: absolute;
    top: 0.3rem;
    right: 0.4rem;
    font-size: 0.55rem;
    font-weight: 700;
    border-radius: 3px;
    padding: 0.05rem 0.25rem;

    &.badge-nf {
        color: var(--accent);
        border: 1px solid var(--accent);
    }

    &.badge-error {
        color: var(--error);
        border: 1px solid var(--error);
    }
}

.frame-section {
    margin-bottom: 0.35rem;

    &:last-child {
        margin-bottom: 0;
    }
}

.frame-label {
    font-size: 0.6rem;
    text-transform: uppercase;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    margin-bottom: 0.15rem;
}

</style>