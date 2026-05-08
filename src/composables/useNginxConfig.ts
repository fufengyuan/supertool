import { ref } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import { useErrorHandler } from './useErrorHandler'
import { useToast } from './useToast'

export function useNginxConfig() {
  const { handleError } = useErrorHandler()
  const toast = useToast()

  const loading = ref(false)
  const presets = ref<any[]>([])
  const currentPreset = ref<any>(null)
  const configContent = ref('')
  const versions = ref<any[]>([])
  const testResult = ref<any>(null)
  const servers = ref<any[]>([])
  const serverGroups = ref<any[]>([])

  // Load all presets
  const loadPresets = async () => {
    try {
      loading.value = true
      const result = await getTauriAPI().getNginxPresets()
      presets.value = result?.data || result || []
    } catch (err) {
      handleError(err, { context: 'loadNginxPresets' })
    } finally {
      loading.value = false
    }
  }

  // Load servers for selector
  const loadServers = async () => {
    try {
      servers.value = await getTauriAPI().getAllServers() || []
      serverGroups.value = await getTauriAPI().getServerGroups() || []
    } catch (err) {
      handleError(err, { context: 'loadServers' })
    }
  }

  // Save preset (create or update)
  const savePreset = async (preset: any) => {
    try {
      if (preset.id) {
        await getTauriAPI().updateNginxPreset(preset)
        toast.success('预设已更新')
      } else {
        preset.id = crypto.randomUUID()
        preset.createdAt = new Date().toISOString()
        await getTauriAPI().addNginxPreset(preset)
        toast.success('预设已创建')
      }
      preset.updatedAt = new Date().toISOString()
      await loadPresets()
      return preset
    } catch (err) {
      handleError(err, { context: 'saveNginxPreset' })
    }
  }

  // Delete preset
  const deletePreset = async (id: string) => {
    try {
      await getTauriAPI().deleteNginxPreset(id)
      toast.success('预设已删除')
      if (currentPreset.value?.id === id) {
        currentPreset.value = null
        configContent.value = ''
      }
      await loadPresets()
    } catch (err) {
      handleError(err, { context: 'deleteNginxPreset' })
    }
  }

  // Fetch remote nginx config
  const fetchConfig = async (preset: any) => {
    try {
      loading.value = true
      currentPreset.value = preset
      const result = await getTauriAPI().fetchNginxConfig(preset.serverId, preset.configPath)
      configContent.value = result?.data || result || ''
      // Load version history
      const verResult = await getTauriAPI().getNginxConfigVersions(preset.id)
      versions.value = verResult?.data || verResult || []
      toast.success('配置已获取')
    } catch (err) {
      handleError(err, { context: 'fetchNginxConfig' })
    } finally {
      loading.value = false
    }
  }

  // Test nginx config
  const testConfig = async (serverId: string) => {
    try {
      loading.value = true
      const result = await getTauriAPI().testNginxConfig(serverId)
      testResult.value = result?.data || result
      return result?.data || result
    } catch (err) {
      handleError(err, { context: 'testNginxConfig' })
    } finally {
      loading.value = false
    }
  }

  // Deploy config to remote
  const deployConfig = async (comment: string) => {
    if (!currentPreset.value || !configContent.value) return
    try {
      loading.value = true
      const p = currentPreset.value
      const result = await getTauriAPI().deployNginxConfig(p.serverId, p.configPath, configContent.value, comment)
      if (result?.success) {
        // Save version to local DB
        await getTauriAPI().saveNginxConfigVersion({
          id: crypto.randomUUID(),
          presetId: p.id,
          content: configContent.value,
          checksum: await computeChecksum(configContent.value),
          comment,
          isCurrent: true,
          createdAt: new Date().toISOString(),
        })
        // Reload versions
        const verResult = await getTauriAPI().getNginxConfigVersions(p.id)
        versions.value = verResult?.data || verResult || []
        toast.success('配置已发布')
      } else {
        toast.error(result?.error || '发布失败')
      }
      return result
    } catch (err) {
      handleError(err, { context: 'deployNginxConfig' })
    } finally {
      loading.value = false
    }
  }

  // Rollback to a specific version
  const rollbackToVersion = async (versionId: string) => {
    if (!currentPreset.value) return
    try {
      loading.value = true
      const version = versions.value.find(v => v.id === versionId)
      if (!version) return

      // Deploy the old version's content
      const p = currentPreset.value
      const result = await getTauriAPI().deployNginxConfig(p.serverId, p.configPath, version.content, `回滚到版本: ${version.comment || versionId}`)
      if (result?.success) {
        configContent.value = version.content
        await getTauriAPI().setActiveNginxVersion(p.id, versionId)
        const verResult = await getTauriAPI().getNginxConfigVersions(p.id)
        versions.value = verResult?.data || verResult || []
        toast.success('已回滚')
      } else {
        toast.error(result?.error || '回滚失败')
      }
    } catch (err) {
      handleError(err, { context: 'rollbackNginxVersion' })
    } finally {
      loading.value = false
    }
  }

  // Simple checksum
  const computeChecksum = async (content: string): Promise<string> => {
    const encoder = new TextEncoder()
    const data = encoder.encode(content)
    const hashBuffer = await crypto.subtle.digest('SHA-256', data)
    const hashArray = Array.from(new Uint8Array(hashBuffer))
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').substring(0, 16)
  }

  return {
    loading, presets, currentPreset, configContent, versions, testResult,
    servers, serverGroups,
    loadPresets, loadServers, savePreset, deletePreset,
    fetchConfig, testConfig, deployConfig, rollbackToVersion,
  }
}
