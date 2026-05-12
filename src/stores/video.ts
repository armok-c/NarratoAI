import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VideoMeta } from '@/types'

export const useVideoStore = defineStore('video', () => {
  const videos = ref<VideoMeta[]>([])

  function addVideo(v: VideoMeta) {
    videos.value.push(v)
  }

  function removeVideo(id: string) {
    videos.value = videos.value.filter(v => v.id !== id)
  }

  function clear() {
    videos.value = []
  }

  return {
    videos,
    addVideo,
    removeVideo,
    clear,
  }
})
