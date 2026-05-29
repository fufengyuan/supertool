export interface Todo {
  id: string;
  text: string;
  completed: boolean;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
  priority: 'high' | 'medium' | 'low' | '';
  tag: string;
  projectId?: string;
  dueDate?: string;
  description?: string;
  repeatType?: string;
  repeatInterval?: number;
  orderNum?: number;
  editedBy?: string;
  assignedTo?: string;
  owner?: string;
  comments?: Comment[];
}

export interface Comment {
  id: string;
  author: string;
  content: string;
  timestamp: string;
}

export interface Subtask {
  id: string;
  todoId: string;
  text: string;
  completed: boolean;
  orderNum: number;
}

export interface Tag {
  name: string;
}

export interface Project {
  id: string;
  name: string;
  description?: string;
  color?: string;
  category?: string;
  archived?: boolean;
  gitUrl1?: string;
  gitUrl2?: string;
  repoPath?: string;
  branch?: string;
  repoPath2?: string;
  branch2?: string;
  createdAt: string;
  updatedAt: string;
  stats?: {
    total: number;
    completed: number;
    progress: number;
  };
}

export interface CicdConfig {
  id: string;
  projectId: string;
  sshHost: string;
  deployBranch: string;
  sshPort?: number;
  sshUser?: string;
  sshPassword?: string;
  sshKeyPath?: string;
  deployPath?: string;
  mavenProfile?: string;
  mavenSettings?: string;
  libSeparate?: boolean;
  restartScript?: string;
  healthCheckUrl?: string;
  servers?: string;  // JSON string of server IDs
  modules?: CicdModule[];
}

export interface CicdModule {
  id: string;
  name: string;
  path: string;
  artifactName?: string;
  order: number;
  enabled: boolean;
}

export type SortBy = 'priority' | 'dueDate' | 'createdAt' | 'default' | null;

export type Priority = 'high' | 'medium' | 'low' | '';

export type ViewMode =
  | 'todo'
  | 'weekly-report'
  | 'projects'
  | 'servers'
  | 'data-backup'
  | 'notifications'
  | 'database'
  | 'mfa'
  | 'notes'
  | 'cicd'
  | 'devtools'
  | 'accounting'
  | 'vpn'
  | 'log-aggregator'
  | 'git';

// Git Repository types
export interface GitRepo {
  id: string
  name: string
  path: string
  remote?: string
  branch?: string
  lastOpened?: string
  createdAt: string
  updatedAt: string
}

export type FilterMode = 'all' | 'active' | 'completed';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
  duration: number;
  progress: number;
  _timer?: ReturnType<typeof setInterval>;
}

export interface LogEntry {
  timestamp: string;
  level: number;
  levelLabel: string;
  message: string;
  context: string;
}

export interface TodoFilterOptions {
  filter?: FilterMode;
  tagFilter?: string;
  searchQuery?: string;
  priorityFilter?: string;
  statusFilter?: FilterMode;
  sortBy?: SortBy;
}

export interface DeployPreflightResult {
  name: string;
  passed: boolean;
  message: string;
}

export interface DeployPreflightReport {
  passed: boolean;
  results: DeployPreflightResult[];
}

export interface UserInfo {
  name: string;
}

export interface ExportOptions {
  scope?: string;
}

export interface ImportOptions {
  mode?: 'merge' | 'replace';
}

export interface PerformanceMetrics {
  componentRenders: Map<string, { count: number; total: number; max: number }>;
  operationTimings: Array<{ name: string; duration: number; timestamp: number }>;
  fps: number;
  memoryUsage: {
    usedJSHeapSize: number;
    totalJSHeapSize: number;
    jsHeapSizeLimit: number;
  } | null;
  virtualListEnabled: boolean;
}

export interface ErrorHandlerOptions {
  context?: string;
  showToast?: boolean;
  rethrow?: boolean;
}

export type ErrorCategory = 'network' | 'database' | 'validation' | 'unknown';

export interface MfaSecret {
  id: string;
  name: string;
  secret: string;
  digits: number;
  period: number;
  algorithm: string;
  account?: string;
  issuer?: string;
  createdAt: string;
  updatedAt: string;
}

export interface AccountingCategory {
  id: string;
  name: string;
  type: 'income' | 'expense';
  icon: string;
  sortOrder: number;
  createdAt: string;
}

export interface AccountingRecord {
  id: string;
  date: string;
  type: 'income' | 'expense';
  category: string;
  amount: number;
  description: string;
  status: string;
  attachmentPath: string | null;
  // Enterprise fields
  voucherNumber?: string;
  receiptType?: string;
  receiptPath?: string;
  entity?: string;
  project?: string;
  supplier?: string;
  invoiceNumber?: string;
  taxAmount?: number;
  paymentMethod?: string;
  approver?: string;
  attachmentsJson?: string;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

// ============ OpenVPN ============

export interface OpenVPNConfig {
  id: string;
  name: string;
  filePath: string;        // Full path to .ovpn file
  content: string;         // Raw .ovpn content
  createdAt: string;
  updatedAt: string;
}

export interface OpenVPNStatus {
  connected: boolean;
  configId: string | null;   // Currently connected config ID
  configName: string | null;
  state: 'disconnected' | 'connecting' | 'connected' | 'error' | 'disconnecting';
  log: string[];             // Recent connection log lines
  connectedSince?: string;
  remote?: string;
}

// ============ WireGuard ============

export interface WireGuardConfig {
  id: string;
  name: string;
  privateKey: string;
  publicKey: string;
  address: string;
  dns?: string;
  mtu?: number;
  peerPublicKey: string;
  peerEndpoint: string;
  peerAllowedIPs: string;
  peerPersistentKeepalive?: number;
  presharedKey?: string;
  createdAt: string;
  updatedAt: string;
}

export interface WireGuardStatus {
  connected: boolean;
  configId: string | null;
  configName: string | null;
  state: string;
  log: string[];
  connectedSince?: string;
  bytesSent: number;
  bytesReceived: number;
  latestHandshake?: string;
}

// ============ Log Aggregator ============

export interface LogPreset {
  id: string;
  name: string;
  serverIds: string[];
  logPath: string;
  logType: 'file' | 'journalctl' | 'docker' | 'custom';
  keywords: string[];
  maxLines: number;
  createdAt: string;
  updatedAt: string;
}

export interface LogStreamParams {
  serverIds: string[];
  command: string;
  maxLines?: number;
}

export interface LogLine {
  serverId: string;
  serverName: string;
  timestamp: number;
  content: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  raw: string;
}


// ============ Additional types for Tauri ============

export interface DbConnectionConfig {
  id?: string;
  name: string;
  type: 'mysql' | 'postgresql' | 'redis' | 'sqlite';
  host?: string;
  port?: number;
  username?: string;
  password?: string;
  database?: string;
  path?: string;
  dbIndex?: number;
  requiresApproval?: boolean;
}

export interface Server {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  sshKeyPath?: string;
  privateKey?: string;
  protocol?: string;
  tags?: string[];
  description?: string;
  groupId?: string | null;
  orderNum?: number;
  createdAt?: string;
  updatedAt?: string;
}

export interface ServerGroup {
  id: string;
  name: string;
  color?: string;
  orderNum?: number;
  createdAt?: string;
}

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface ProjectStats {
  total: number;
  completed: number;
  progress: number;
}

export interface NotificationSettings {
  enabled: boolean;
  reminderTime: number;
  soundEnabled: boolean;
  desktopEnabled: boolean;
}

export interface Note {
  id: string;
  title: string;
  content: string;
  groupId?: string;
  createdAt: string;
  updatedAt: string;
  orderNum?: number;
}

export interface NoteGroup {
  id: string;
  name: string;
  icon?: string;
  orderNum?: number;
  createdAt: string;
}

export interface WeeklyReport {
  id: string;
  title: string;
  content: string;
  startDate: string;
  endDate: string;
  createdAt: string;
  updatedAt: string;
}

export interface Budget {
  id: string;
  category: string;
  amount: number;
  period: 'monthly' | 'yearly';
  createdAt: string;
  updatedAt: string;
}

// ============ Hermes Tools ============

export interface ToolsetInfo {
  key: string;
  label: string;
  description: string;
  enabled: boolean;
}

export interface MCPServerInfo {
  name: string;
  type: 'http' | 'stdio' | 'other';
  detail: string;
}

// ============ Hermes Memory ============

export interface MemoryEntry {
  index: number;
  content: string;
}

export interface MemoryFileInfo {
  content: string;
  exists: boolean;
  lastModified: number | null;
  entries: MemoryEntry[];
  charCount: number;
  charLimit: number;
}

export interface SessionStats {
  totalSessions: number;
  totalMessages: number;
}

export interface MemoryInfo {
  memory: MemoryFileInfo;
  user: MemoryFileInfo;
  stats: SessionStats;
}

export interface MemoryProviderInfo {
  name: string;
  description: string;
  installed: boolean;
  active: boolean;
  envVars: string[];
}

export interface MemoryProviderResult {
  providers: MemoryProviderInfo[];
  activeProvider: string;
  memoryEnabled: boolean;
  userProfileEnabled: boolean;
  memoryCharLimit: number;
  userCharLimit: number;
}

export interface MemoryWriteResult {
  success: boolean;
  error?: string;
}

// ============ Provider Credential Management ============

export interface ProviderInfo {
  id: string
  name: string
  authType: string
  configured: boolean
  hasValidKey: boolean
  apiKeyPreview: string
}

export interface ProviderListResult {
  success: boolean
  providers: ProviderInfo[]
}

export interface ProviderSaveResult {
  success: boolean
  providerId: string
}

export interface OAuthFlowResult {
  success: boolean
  authorizationUrl: string
  deviceCode: string
  verificationUri: string
  providerId: string
  note?: string
}

export interface OAuthPollResult {
  success: boolean
  providerId: string
  configured: boolean
  hasToken: boolean
  completed: boolean
}

// ============ Hermes Cron Jobs ============

export interface CronJob {
  id: string
  name: string
  prompt: string
  schedule: string       // schedule_display (e.g. "once in 30m", "every 2h")
  state: 'active' | 'paused' | 'completed' | 'scheduled'
  enabled: boolean
  next_run_at: string | null
  last_run_at: string | null
  last_status: string | null
  last_error: string | null
  deliver: string
  skills: string[]
  script: string | null
}

// ============ Hermes Skills ============

export interface SkillInfo {
  name: string
  category: string
  description: string
  path: string
  source: 'installed' | 'bundled'
}

export interface SkillCliResult {
  success: boolean
  error?: string
}

// ============ Hermes Config (Agent Settings) ============

export interface HermesConfigInfo {
  hermesHome: string
  configExists: boolean
  installed: boolean
  version: string
}

export interface ConfigExportResult {
  success: boolean
  content?: string
  message?: string
}

export interface ConfigImportResult {
  success: boolean
  message?: string
}
