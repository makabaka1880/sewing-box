<template>
    <codemirror v-model="code" placeholder="Code goes here..." :style="{ height: '100%' }" :autofocus="true"
        :indent-with-tab="true" :tab-size="2" :extensions="extensions" @ready="handleReady"
        @change="log('change', $event)" @focus="log('focus', $event)" @blur="log('blur', $event)" />
</template>

<script lang="ts" setup>
import { shallowRef } from 'vue'
import { Codemirror } from 'vue-codemirror'

const code = defineModel<string>({ default: '' })

const props = withDefaults(defineProps<{
  defaultContent?: string
}>(), {
  defaultContent: '',
})

if (props.defaultContent) {
  code.value = props.defaultContent
}

const extensions: any[] = []

const view = shallowRef()
const handleReady = (payload: any) => {
    view.value = payload.view
}

const log = (..._: any[]) => { }
</script>