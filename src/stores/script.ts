import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useScriptStore = defineStore('script', () => {
  const content = ref<string | null>(null)

  function load(id: string) {
    // TODO Phase 17: full implementation
  }

  function save() {
    // TODO Phase 17: full implementation
  }

  function validate() {
    // TODO Phase 17: full implementation
  }

  function update(text: string) {
    // TODO Phase 17: full implementation
  }

  return {
    content,
    load,
    save,
    validate,
    update,
  }
})
