/**
 * SFTP Driver for VueFinder
 * Implements Driver interface using Tauri IPC to communicate with Rust SFTP backend
 * Note: Some operations are not fully supported due to limited backend API
 */
import { BaseAdapter } from 'vuefinder'
import type {
  Driver,
  FsData,
  DirEntry,
  DeleteResult,
  FileOperationResult,
  FileContentResult,
  ListParams,
  DeleteParams,
  RenameParams,
  TransferParams,
  ArchiveParams,
  SaveParams,
  SearchParams
} from 'vuefinder'
import { getTauriAPI } from '@/utils/tauri-api'

export interface SftpDriverConfig {
  serverId: string
  serverName: string
  initialPath?: string
}

/**
 * SFTP Driver - connects to remote server via SSH/SFTP
 */
export class SftpDriver extends BaseAdapter {
  private serverId: string
  private serverName: string
  private currentPath: string

  constructor(config: SftpDriverConfig) {
    super()
    this.serverId = config.serverId
    this.serverName = config.serverName
    this.currentPath = config.initialPath || '/'
  }

  /**
   * Convert SFTP file info to VueFinder DirEntry
   */
  private toDirEntry(file: { name: string; type: string; size?: number; modifyTime?: string }, dir: string): DirEntry {
    const isDir = file.type === 'directory'
    const path = dir === '/' ? `/${file.name}` : `${dir}/${file.name}`
    const extension = isDir ? '' : this.getExtension(file.name)
    
    return {
      dir: dir,
      basename: file.name,
      extension: extension,
      path: path,
      storage: 'sftp',
      type: isDir ? 'dir' : 'file',
      file_size: file.size ?? null,
      last_modified: file.modifyTime ? new Date(file.modifyTime).getTime() : null,
      mime_type: isDir ? null : this.getMimeType(file.name),
      read_only: false,
      visibility: 'public'
    }
  }

  /**
   * Get file extension from filename
   */
  private getExtension(filename: string): string {
    const parts = filename.split('.')
    return parts.length > 1 ? parts.pop()!.toLowerCase() : ''
  }

  /**
   * Get MIME type from filename
   */
  private getMimeType(filename: string): string {
    const ext = this.getExtension(filename)
    const mimeTypes: Record<string, string> = {
      'txt': 'text/plain',
      'md': 'text/markdown',
      'json': 'application/json',
      'js': 'application/javascript',
      'ts': 'application/javascript',
      'html': 'text/html',
      'css': 'text/css',
      'xml': 'application/xml',
      'yaml': 'application/x-yaml',
      'yml': 'application/x-yaml',
      'pdf': 'application/pdf',
      'zip': 'application/zip',
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'png': 'image/png',
      'gif': 'image/gif',
      'svg': 'image/svg+xml',
      'sh': 'application/x-sh',
      'py': 'text/x-python',
      'java': 'text/x-java',
      'rs': 'text/x-rust',
      'go': 'text/x-go',
      'sql': 'application/x-sql',
      'log': 'text/plain',
      'conf': 'text/plain',
    }
    return mimeTypes[ext] || 'application/octet-stream'
  }

  /**
   * Normalize path - ensure it starts with /
   */
  private normalizePath(path: string): string {
    if (!path) return '/'
    if (!path.startsWith('/')) return '/' + path
    return path
  }

  /**
   * List files at a path
   */
  async list(params?: ListParams): Promise<FsData> {
    const rawPath = params?.path || this.currentPath
    const path = this.normalizePath(rawPath)
    this.currentPath = path

    try {
      const result = await getTauriAPI().listSftpDir(this.serverId, path)
      
      if (!result.success) {
        throw new Error(result.error || 'Failed to list directory')
      }

      // Sort: directories first, then alphabetically
      const files = (result.files || [])
        .sort((a: any, b: any) => {
          if (a.type === 'directory' && b.type !== 'directory') return -1
          if (a.type !== 'directory' && b.type === 'directory') return 1
          return a.name.localeCompare(b.name)
        })
        .map((f: any) => this.toDirEntry(f, path))

      return {
        storages: ['sftp'],
        dirname: path,
        files: files,
        read_only: false
      }
    } catch (error: any) {
      throw new Error(`List failed: ${error.message}`)
    }
  }

  /**
   * Delete files/folders
   */
  async delete(params: DeleteParams): Promise<DeleteResult> {
    const deleted: DirEntry[] = []
    
    for (const item of params.items) {
      try {
        const isDir = item.type === 'dir' || item.type === 'directory'
        const result = await getTauriAPI().deleteSftpFile(this.serverId, item.path, isDir)
        if (result.success) {
          deleted.push({
            dir: params.path,
            basename: item.path.split('/').pop() || '',
            extension: '',
            path: item.path,
            storage: 'sftp',
            type: isDir ? 'dir' : 'file',
            file_size: null,
            last_modified: null,
            mime_type: null,
            read_only: false,
            visibility: 'public'
          })
        } else {
          throw new Error(result.error || 'Delete failed')
        }
      } catch (error: any) {
        throw new Error(`Delete ${item.path} failed: ${error.message}`)
      }
    }

    const fsData = await this.list({ path: params.path })
    return {
      files: fsData.files,
      storages: fsData.storages as any,
      read_only: false,
      dirname: params.path,
      deleted: deleted
    }
  }

  /**
   * Rename a file or folder (not directly supported - use move workaround)
   */
  async rename(params: RenameParams): Promise<FileOperationResult> {
    // Note: SFTP rename is not directly available in backend
    // This would require adding a rename command in Rust
    throw new Error('Rename operation not supported. Please use SSH terminal to rename files.')
  }

  /**
   * Copy files/folders (not supported)
   */
  async copy(params: TransferParams): Promise<FileOperationResult> {
    throw new Error('Copy operation not supported over SFTP.')
  }

  /**
   * Move files/folders (not supported)
   */
  async move(params: TransferParams): Promise<FileOperationResult> {
    throw new Error('Move operation not supported. Please use SSH terminal.')
  }

  /**
   * Create archive (not supported)
   */
  async archive(params: ArchiveParams): Promise<FileOperationResult> {
    throw new Error('Archive operation not supported.')
  }

  /**
   * Extract archive (not supported)
   */
  async unarchive(params: { item: string; path: string }): Promise<FileOperationResult> {
    throw new Error('Unarchive operation not supported.')
  }

  /**
   * Create new file (open editor with empty file)
   */
  async createFile(params: { path: string; name: string }): Promise<FileOperationResult> {
    const filePath = `${params.path}/${params.name}`
    try {
      // Open the file editor - this will create the file if it doesn't exist
      const result = await getTauriAPI().openSftpFileEditor(this.serverId, filePath)
      if (!result?.success) {
        throw new Error(result?.error || 'Create file failed')
      }
      // Return updated list
      const fsData = await this.list({ path: params.path })
      return {
        files: fsData.files,
        storages: fsData.storages as any,
        read_only: false,
        dirname: params.path
      }
    } catch (error: any) {
      throw new Error(`Create file failed: ${error.message}`)
    }
  }

  /**
   * Create folder (not directly supported)
   */
  async createFolder(params: { path: string; name: string }): Promise<FileOperationResult> {
    throw new Error('Create folder not supported. Please use SSH terminal: mkdir <name>')
  }

  /**
   * Get file content (not directly supported - use editor)
   */
  async getContent(params: { path: string }): Promise<FileContentResult> {
    // Open the file in the editor for viewing/editing
    try {
      await getTauriAPI().openSftpFileEditor(this.serverId, params.path)
      return {
        content: '', // Content is shown in editor window
        mimeType: this.getMimeType(params.path)
      }
    } catch (error: any) {
      throw new Error(`Get content failed: ${error.message}`)
    }
  }

  /**
   * Save content to file (not directly supported)
   */
  async save(params: SaveParams): Promise<string> {
    throw new Error('Save operation handled by file editor.')
  }

  /**
   * Get preview URL (not applicable)
   */
  getPreviewUrl(params: { path: string }): string {
    return ''
  }

  /**
   * Get download URL (handled by download function)
   */
  getDownloadUrl(params: { path: string }): string {
    return ''
  }

  /**
   * Search files (not supported)
   */
  async search(params: SearchParams): Promise<DirEntry[]> {
    throw new Error('Search not supported over SFTP.')
  }
}