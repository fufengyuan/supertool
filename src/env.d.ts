/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// Missing types for migrated components
interface DBConfig {
  id: string
  name: string
  type: 'mysql' | 'postgresql' | 'redis' | 'sqlite'
  host: string
  port: number
  user: string
  password?: string
  database?: string
  dbIndex?: number
}

declare const __APP_VERSION__: string
