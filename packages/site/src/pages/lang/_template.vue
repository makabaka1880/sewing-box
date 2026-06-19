<!--
Copy this file to create a new language page.

Replace every occurrence of LANG with the language name (PascalCase for
components/classes, quoted string for lookups).

e.g. for "Brainfuck": rename LANGProgram -> BFProgramWasm,
                        "LANG" -> "Brainfuck",
                        LANGResultCard -> BrainfuckTapeView, etc.

Then write your own execution logic (step/run/clear) and result
visualisation. The scaffold handles loading the WASM program, wiring
up the editor, and the grammar modal (via PlaygroundLayout).
-->

<template>
    <PlaygroundLayout v-model:show-grammar="showGrammar">
        <template #header>
            <h2>{{ getLang("LANG")?.name }}</h2>
            <p>{{ getLang("LANG")?.description }}</p>
        </template>

        <template #header-actions>
            <button class="secondary" @click="showGrammar = true">Spec &#x2197;</button>
        </template>

        <template #editor-label>EDITOR</template>

        <template #editor-tools>
            <!-- TODO: add your step/run/clear buttons here -->
        </template>

        <template #editor>
            <CodeMirror v-model="code" />
        </template>

        <template #result-label>RESULT</template>

        <template #result-header>
            <!-- TODO: step count, clear button, etc. -->
        </template>

        <template #result>
            <div class="result-inner">
                <template v-if="program">
                    <!-- TODO: render your result visualisation here -->
                </template>
                <div v-else class="placeholder">Enter a program in the editor</div>
            </div>
        </template>

        <template #grammar-body>
            <pre>{{ grammarEBNF }}</pre>
        </template>
    </PlaygroundLayout>
</template>

<script lang="ts" setup>
import { ref, watch } from 'vue';
import { getLang, getGrammar, generateEBNF, getSample } from '@/cfg/langs';
import { useEditorStore } from '@/stores/editor';
import PlaygroundLayout from '@/components/PlaygroundLayout.vue';
import CodeMirror from '@/components/CodeMirror.vue';
// TODO: import your WASM class and any result-card components
// import { LANGProgram } from "@sewing-box/wasm-XXXX";

const store = useEditorStore();
const showGrammar = ref(false);
const grammarEBNF = generateEBNF(getGrammar('LANG') ?? []);
const sample = getSample('LANG') ?? '';

const code = ref(store.getCode('LANG', sample));
const program = ref<any | null>(null); // TODO: tighten type

watch(code, (src) => {
    try {
        // TODO: replace with your WASM constructor
        // program.value = new LANGProgram(src);
    } catch (e) {
        program.value = null;
    }
}, { immediate: true });

watch(code, (v) => store.setCode('LANG', v));

// TODO: write your own execution logic
// - step / run / clear functions
// - result derived from program.value.state()
// - halt detection, error handling, step counting
// - whatever execution model fits this language
</script>

<style lang="scss" scoped>
@use "@/style/main.scss" as *;

// ---- result panel ----

.result-inner {
    padding-top: var(--padding-safe);
    padding-bottom: var(--padding-safe);
}

.placeholder {
    color: var(--text-muted);
    font-size: 0.75rem;
}

// TODO: add your own styles for result visualisation
</style>
