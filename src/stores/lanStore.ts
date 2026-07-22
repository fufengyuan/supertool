import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getTauriAPI } from '@/utils/tauri-api'

export const useLanStore = defineStore('lan', () => {
  const unreadCounts = ref<Record<string, number>>({})
  const peerNameMap = ref<Record<string, string>>({})
  const totalUnread = computed(() =>
    Object.values(unreadCounts.value).reduce((a, b) => a + b, 0)
  )

  let cleanupFns: Array<() => void> = []
  let initCalled = false

  async function init() {
    if (initCalled) return
    initCalled = true

    // Load initial unread counts from backend (requires userId, fail silently if not ready)
    try {
      const info = await getTauriAPI().lanGetUserInfo()
      if (info?.id) {
        const res = await getTauriAPI().lanGetAllUnreadCounts(info.id)
        if (res && typeof res === 'object') {
          unreadCounts.value = { ...res }
          for (const peerId of Object.keys(res)) {
            peerNameMap.value[peerId] = peerId
          }
        }
      }
    } catch { /* lan not started yet */ }

    // Bump unread count for incoming text messages
    cleanupFns.push(await getTauriAPI().lanOnMessage((data: any) => {
      if (!data || !data.from) return
      const senderId = data.from
      const senderName = data.fromName || data.name || data.userName || senderId
      peerNameMap.value[senderId] = senderName

      if (!unreadCounts.value[senderId]) {
        unreadCounts.value[senderId] = 0
      }
      unreadCounts.value[senderId]++

      console.log(`[LAN] Message from ${senderName}: ${data.content?.substring(0, 80)}`)
    }))

    // Bump unread count for incoming file messages (so user sees the badge even if chat is closed)
    cleanupFns.push(await getTauriAPI().lanOnFileReceived((data: any) => {
      if (!data || !data.fromUserId) return
      const senderId = data.fromUserId
      const senderName = data.fromUserName || senderId
      peerNameMap.value[senderId] = senderName

      if (!unreadCounts.value[senderId]) {
        unreadCounts.value[senderId] = 0
      }
      unreadCounts.value[senderId]++

      console.log(`[LAN] File from ${senderName}: ${data.fileName}`)
    }))
  }

  function destroy() {
    cleanupFns.forEach(fn => fn())
    cleanupFns = []
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
