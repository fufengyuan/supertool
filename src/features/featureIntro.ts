// 核心功能新手引导注册表
//
// 仅覆盖「有使用门槛 / 依赖前置资源」的核心功能页（服务器、Git、CICD、数据库、
// 日志、Nginx、VPN、备份、告警）。每个功能首次进入时弹窗展示「功能介绍 /
// 使用方法 / 前置条件」，前置条件可带跳转链接直接去补齐资源（如去添加服务器）。
// devtools 等自带说明、无依赖的小工具不纳入，避免打扰。
//
// 记忆机制：sessionStorage 仅缓存本次会话（本次打开有效），重启应用后清空，
// 下次启动进入功能页会继续弹出；Close 即视为已看过。

export interface FeaturePrereq {
  /** 前置条件描述（如「已登记的服务器」） */
  label: string
  /** 点击跳转的路由（如 /servers），可空表示纯文字要求 */
  link?: string
  /** 链接按钮文案，默认「去添加」 */
  linkLabel?: string
}

export interface FeatureIntro {
  /** 路由 path，精确匹配（如 /cicd） */
  path: string
  /** 功能名 */
  title: string
  /** 功能介绍 */
  intro: string
  /** 使用方法（多步骤用换行分隔） */
  howto: string
  /** 前置条件 */
  prereqs: FeaturePrereq[]
}

export const FEATURE_INTROS: FeatureIntro[] = [
  {
    path: '/servers',
    title: '服务器管理',
    intro:
      '统一登记多台服务器的连接信息（SSH 账号、密码/私钥、分组），是日志搜索、Nginx、VPN、备份、告警等远程功能的基础，请先维护好这里的服务器。',
    howto: '① 点击「添加服务器」录入名称、IP/域名、SSH 端口与登录凭据，可分组建文件夹；\n② 保存后可「测试连接」验证是否可达；\n③ 后续在日志、CICD 等功能的服务器选择器里直接勾选使用。',
    prereqs: [],
  },
  {
    path: '/git',
    title: 'Git 仓库',
    intro:
      '登记本机或远程的 Git 仓库。仓库登记后即可被 CI/CD 部署、代码浏览等功能引用，是持续部署的前置。',
    howto:
      '① 点击「添加仓库」，选择本地目录或粘贴远程地址；\n② 自动读取分支列表，可指定默认部署分支；\n③ 登记后即可在 CI/CD 新建配置时选择该仓库。',
    prereqs: [],
  },
  {
    path: '/cicd',
    title: 'CI/CD 持续部署',
    intro:
      '一键完成「拉取代码 → 构建 → 上传 → 重启」的持续部署，支持单体/多模块、多环境部署与健康检查失败自动回滚。',
    howto:
      '① 选择已登记的 Git 仓库与部署分支；\n② 选择目标服务器与部署路径；\n③ 配置构建工具、启动/重启脚本（可再细配多环境与健康检查）；\n④ 点击「部署」实时查看构建与上传进度，失败会自动回滚到上一版本。',
    prereqs: [
      { label: '已登记的 Git 仓库', link: '/git', linkLabel: '去添加仓库' },
      { label: '已登记的目标服务器', link: '/servers', linkLabel: '去添加服务器' },
    ],
  },
  {
    path: '/database',
    title: '数据库管理',
    intro:
      '图形化连接 MySQL/PG/SQLite/Redis/ES 等数据库，提供连接树、表结构查看、SQL 执行、数据同步与 Redis 键管理。',
    howto:
      '① 「新增连接」填写类型、地址、端口与账号密码，测试通过后保存；\n② 左侧连接树展开库表，右键可建表/看结构；\n③ SQL 编辑器执行查询，结果可直接导出；Redis/ES 单独页签管理键值与索引。',
    prereqs: [{ label: '可访问的数据库实例（地址、端口、账号密码，需对本机开放）' }],
  },
  {
    path: '/logs',
    title: '日志搜索',
    intro:
      '在已登记的服务器上实时或历史检索日志，支持关键字、错误级别过滤，以及按日期范围搜索已轮转压缩的 gzip 日志。',
    howto:
      '① 选择一台已登记的服务器并填入日志路径与关键字；\n② 支持指定日期/天数搜索历史轮转日志（自动匹配文件名日期），实时日志即时 tail；\n③ 命中行带上下文展示，可一键复制或跳转错误溯源。',
    prereqs: [{ label: '已登记的服务器（日志所在主机）', link: '/servers', linkLabel: '去添加服务器' }],
  },
  {
    path: '/nginx',
    title: 'Nginx 管理',
    intro:
      '通过 SSH 管理远程 Nginx：结构化编辑 server/stream/upstream、证书与黑白名单，生成配置并一键重载，重载失败自动回滚。',
    howto:
      '① 选择服务器并导入其现有配置作为镜像；\n② 在结构树中增删改 server/upstream/location 等块（支持 SSL、IPv6）；\n③ 保存并「应用」，校验通过后重载；重载失败自动恢复原配置。',
    prereqs: [
      { label: '已登记的服务器（需可 SSH 且 Nginx 已安装）', link: '/servers', linkLabel: '去添加服务器' },
    ],
  },
  {
    path: '/vpn',
    title: 'VPN 组网',
    intro:
      '基于 WireGuard 快速组建内网：集中管理对端节点、内网网段与路由，一键生成并下发配置。',
    howto:
      '① 选择一台 Linux 服务器作为 VPN 节点；\n② 录入各对端的公钥与内网 IP，规划网段；\n③ 生成配置并下发，节点状态可在面板查看。',
    prereqs: [{ label: '已登记的 Linux 服务器（作为 VPN 节点）', link: '/servers', linkLabel: '去添加服务器' }],
  },
  {
    path: '/backup',
    title: '配置备份',
    intro:
      '将本机配置、数据库或关键文件打包备份到远端服务器，支持定时策略与从备份恢复，防止误操作丢数据。',
    howto:
      '① 选择备份来源（本机配置/数据库）与目标服务器+目录；\n② 配置保留份数、是否加密，可设定时任务；\n③ 恢复时选择备份点一键还原。',
    prereqs: [
      { label: '已登记的服务器（作为备份目标）', link: '/servers', linkLabel: '去添加服务器' },
    ],
  },
  {
    path: '/alert',
    title: '监控告警',
    intro:
      '汇聚服务器资源、服务异常等监控事件，集中查看与跟踪告警，第一时间发现线上问题。',
    howto:
      '① 选择监控目标服务器并配置采集；\n② 设置告警规则与通知渠道；\n③ 在告警列表跟进处理状态与历史。',
    prereqs: [
      { label: '已登记的服务器（监控目标）', link: '/servers', linkLabel: '去添加服务器' },
    ],
  },
]

// 记忆机制：仅缓存到本次会话（sessionStorage），关闭浏览器/重启应用后清空，
// 下次启动进入功能页会继续弹出引导；本次运行内每个功能首次进入只弹一次。
const SEEN_KEY = 'feature_intro_seen_v1'

/** 按路由 path 精确匹配引导配置（忽略 query/hash） */
export function getIntroForPath(path: string): FeatureIntro | null {
  const p = path.split('?')[0].split('#')[0]
  return FEATURE_INTROS.find(f => f.path === p) || null
}

function readSeen(): string[] {
  try {
    return JSON.parse(sessionStorage.getItem(SEEN_KEY) || '[]') as string[]
  } catch {
    return []
  }
}

function writeSeen(seen: string[]): void {
  try {
    sessionStorage.setItem(SEEN_KEY, JSON.stringify(seen))
  } catch {
    /* 忽略存储异常 */
  }
}

export function isIntroSeen(path: string): boolean {
  return readSeen().includes(path)
}

export function markIntroSeen(path: string): void {
  const seen = readSeen()
  if (!seen.includes(path)) {
    seen.push(path)
    writeSeen(seen)
  }
}