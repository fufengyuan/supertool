/**
 * 模块级单例共享 CICD 数据层。
 *
 * 解决 CICD 页面 DeployPanel + CiCdConfig 各自重复加载相同数据集的问题。
 * 模块级 ref 是单例的（ESM 模块缓存），多次调用 useSharedCicdData() 返回同一份数据。
 * 首次调用 load() 触发实际加载，后续调用直接跳过。
 * refresh() 强制重新加载所有数据。
 */

import { ref, type Ref } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'
import type { Project, Server } from '../types'

export interface CicdConfigEntry {
  id: string
  name?: string
  gitRepoId?: string
  deployBranch?: string
  buildTool?: string
  updatedAt?: string
  createdAt?: string
  lastDeployedAt?: string
  groupName?: string
  requiresApproval?: boolean
  servers?: string
  [key: string]: unknown
}

export interface ServerGroupEntry {
  id: string
  name: string
  color: string
  parentId: string | null
}

// ─── Module-level singleton state (ESM 隐式单例) ───

const configs: Ref<CicdConfigEntry[]> = ref([])
const projects: Ref<Project[]> = ref([])
const servers: Ref<Server[]> = ref([])
const serverGroups: Ref<ServerGroupEntry[]> = ref([])
const gitRepos: Ref<any[]> = ref([])

let _loaded = false
let _loading = false
let _loadPromise: Promise<void> | null = null

export function useSharedCicdData() {
  async function load(refresh = false): Promise<void> {
    // 如果已加载且不需要刷新，立即跳过
    if (_loaded && !refresh) return
    // 如果正在加载中，等待完成，再根据 refresh 决定是否重载
    if (_loading) {
      await _loadPromise
      if (refresh) {
        // 刚完成的加载可能已设置了 _loaded，需要强制重载
        _loaded = false
      } else {
        return // 数据已就绪，无需额外操作
      }
    }

    _loading = true
    _loadPromise = (async () => {
      try {
        const [allConfigs, allProjects, allServers, allSGroups, allGitRepos] = await Promise.all([
          getTauriAPI().getCicdConfigs?.() as Promise<CicdConfigEntry[]> | undefined,
          getTauriAPI().getProjects?.() as Promise<Project[]> | undefined,
          getTauriAPI().getAllServers?.() as Promise<Server[]> | undefined,
          getTauriAPI().getAllServerGroups?.() as Promise<ServerGroupEntry[]> | undefined,
          getTauriAPI().getGitRepos?.() as Promise<any> | undefined,
        ])

        configs.value = (allConfigs as CicdConfigEntry[]) || []
        projects.value = (allProjects as Project[]) || []
        servers.value = (allServers as Server[]) || []
        serverGroups.value = (allSGroups as ServerGroupEntry[]) || []

        const repoResult = allGitRepos as any
        gitRepos.value = repoResult?.success && repoResult?.data ? repoResult.data : (Array.isArray(repoResult) ? repoResult : [])

        _loaded = true
      } finally {
        _loading = false
        _loadPromise = null
      }
    })()
    await _loadPromise
  }

  async function refresh(): Promise<void> {
    _loaded = false
    await load(true)
  }

  function isLoaded(): boolean {
    return _loaded
  }

  return {
    configs,
    projects,
    servers,
    serverGroups,
    gitRepos,
    load,
    refresh,
    isLoaded,
  }
}
