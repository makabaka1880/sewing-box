import { ref } from 'vue'
import { defineStore } from 'pinia'

const KEY = 'sewing-box:editor-codes'

function load(): Record<string, string> {
  try {
    const raw = localStorage.getItem(KEY)
    return raw ? JSON.parse(raw) : {}
  } catch {
    return {}
  }
}

function persist(codes: Record<string, string>) {
  try {
    localStorage.setItem(KEY, JSON.stringify(codes))
  } catch {
    // localStorage full or unavailable — silently drop
  }
}

export const useEditorStore = defineStore('editor', () => {
  const codes = ref<Record<string, string>>(load())

  function getCode(lang: string, fallback: string): string {
    return codes.value[lang] ?? fallback
  }

  function setCode(lang: string, code: string) {
    codes.value = { ...codes.value, [lang]: code }
    persist(codes.value)
  }

  function clear(lang: string) {
    const { [lang]: _, ...rest } = codes.value
    codes.value = rest
    persist(codes.value)
  }

  return { codes, getCode, setCode, clear }
})
