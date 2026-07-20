import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getTauriAPI } from '@/utils/tauri-api'

export const useLanStore = defineStore('lan', () => {
  const unreadCounts = ref<Record<string, number>>({})
  const peerNameMap = ref<Record<string, string>>({})
  const totalUnread = computed(() =>
    Object.values(unreadCounts.value).reduce((a, b) => a + b, 0)
  )

  let cleanupFn: (() => void) | null = null
  let initCalled = false

  async function init() {
    if (initCalled) return
    initCalled = true

    // Load initial unread counts from backend (requires userId, fail silently if not ready)
    try {
      const info = await getTauriAPI().lanGetUserInfo()
      if (info?.id) {
        const res = await getTauriAPI().lanGetAllUnreadCounts(info.id)
        if (res?.data) {
          unreadCounts.value = { ...res.data }
          for (const peerId of Object.keys(res.data)) {
            peerNameMap.value[peerId] = peerId
          }
        }
      }
    } catch { /* lan not started yet */ }

    // Listen for kanban task assignments and any other LAN events that may set text
    cleanupFn = await getTauriAPI().lanOnMessage((data: any) => {
      if (!data || !data.from) return
      const senderId = data.from
      const senderName = data.name || data.userName || senderId
      peerNameMap.value[senderId] = senderName

      if (!unreadCounts.value[senderId]) {
        unreadCounts.value[senderId] = 0
      }
      unreadCounts.value[senderId]++

      // Show a brief toast notification via the tray/notification system
      // The backend already fires notify_rust, but we also log to app
      console.log(`[LAN] Message from ${senderName}: ${data.text?.substring(0, 80)}`)
    })
  }

  function destroy() {
    if (cleanupFn) {
      cleanupFn()
      cleanupFn = null
    }
    initCalled = false
  }

  function markAsRead(peerId: string) {
    unreadCounts.value[peerId] = 0
  }

  function resetAllUnread() {
    unreadCounts.value = {}
  }

  function setPeerName(peerId: string, name: string) {
    peerNameMap.value[peerId] = name
  }

  return {
    unreadCounts,
    peerNameMap,
    totalUnread,
    init,
    destroy,
    markAsRead,
    resetAllUnread,
    setPeerName,
  }
})
