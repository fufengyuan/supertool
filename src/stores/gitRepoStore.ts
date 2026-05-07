import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GitRepo } from '../types'

export const useGitRepoStore = defineStore('gitRepo', () => {
  const currentRepo = ref<GitRepo | null>(null)
  const previousViewMode = ref<string>('todo')

  const selectRepo = (repo: GitRepo) => {
    currentRepo.value = repo
  }

  const clearCurrentRepo = () => {
    currentRepo.value = null
  }

  return {
    currentRepo,
    previousViewMode,
    selectRepo,
    clearCurrentRepo,
  }
})
