/**
 * useServers — Tauri 版本
 * 与 Tauri 版接口一致
 */
import { ref } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import type { Ref } from 'vue'
import type { Server, ServerGroup } from '../types'

export function useServers() {
  const loading: Ref<boolean> = ref(false)
  const error: Ref<string | null> = ref(null)

  const fetchServers = async (): Promise<Server[]> => {
    loading.value = true
    error.value = null
    try {
      return await getTauriAPI().getAllServers()
    } catch (err: unknown) {
      error.value = (err as Error).message
      return []
    } finally {
      loading.value = false
    }
  }

  const addServer = async (server: Partial<Server>): Promise<Server> => {
    error.value = null
    try {
      return await getTauriAPI().addServer(server)
    } catch (err) {
      error.value = (err as Error).message
      throw err
    }
  }

  const updateServer = async (server: Server): Promise<Server> => {
    error.value = null
    try {
      return await getTauriAPI().updateServer(server)
    } catch (err) {
      error.value = (err as Error).message
      throw err
    }
  }

  const deleteServer = async (serverId: string): Promise<void> => {
    error.value = null
    try {
      return await getTauriAPI().deleteServer(serverId)
    } catch (err) {
      error.value = (err as Error).message
      throw err
    }
  }

  const testConnection = async (server: Partial<Server>): Promise<{ success: boolean; error?: string }> => {
    try {
      return await getTauriAPI().testServerConnection(server)
    } catch (err) {
      return { success: false, error: (err as Error).message }
    }
  }

  // ============ Groups ============

  const fetchGroups = async (): Promise<ServerGroup[]> => {
    try {
      return await getTauriAPI().getAllServerGroups()
    } catch {
      return []
    }
  }

  const addGroup = async (group: Partial<ServerGroup>): Promise<ServerGroup> => {
    return await getTauriAPI().addServerGroup(group)
  }

  const updateGroup = async (groupId: string, updates: { name?: string; description?: string; parentId?: string | null; color?: string }): Promise<ServerGroup> => {
    return await getTauriAPI().updateServerGroup(groupId, updates)
  }

  const deleteGroup = async (groupId: string): Promise<void> => {
    return await getTauriAPI().deleteServerGroup(groupId)
  }

  return {
    loading,
    error,
    fetchServers,
    addServer,
    updateServer,
    deleteServer,
    testConnection,
    fetchGroups,
    addGroup,
    updateGroup,
    deleteGroup,
  }
}
