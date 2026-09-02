/**
 * 局域网内置头像（SVG）
 *
 * 约定：头像字段值以 `av:` 前缀表示内置 SVG 头像（如 `av:cat`）。
 * 每个头像是一段独立的 SVG 内部内容，viewBox 固定为 `0 0 64 64`。
 * 渐变 id 使用 `{{uid}}` 占位，渲染时由 LanAvatar 组件替换为唯一值，
 * 避免同一页面多处渲染时 id 冲突。
 */

export interface LanAvatarDef {
  /** 头像标识，如 cat */
  key: string
  /** 展示名，用于选择器 title */
  label: string
  /** 背景渐变起始色，用于选择器卡片底色 */
  from: string
  /** 背景渐变结束色 */
  to: string
  /** SVG 内部内容（不含 <svg> 包裹） */
  svg: string
}

function bg(from: string, to: string): string {
  return `<defs><linearGradient id="ag{{uid}}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><circle cx="32" cy="32" r="32" fill="url(#ag{{uid}})"/>`
}

export const LAN_AVATARS: LanAvatarDef[] = [
  {
    key: 'cat',
    label: '橘猫',
    from: '#FFB86B',
    to: '#FF8A4C',
    svg: bg('#FFB86B', '#FF8A4C') + `
      <path d="M13.5 27.5 L11 9.5 L26.5 17.5 Z" fill="#F2763A"/>
      <path d="M50.5 27.5 L53 9.5 L37.5 17.5 Z" fill="#F2763A"/>
      <path d="M16.5 24.5 L15 14.5 L23.5 19.5 Z" fill="#FFC9A0"/>
      <path d="M47.5 24.5 L49 14.5 L40.5 19.5 Z" fill="#FFC9A0"/>
      <ellipse cx="32" cy="37" rx="20" ry="17.5" fill="#FFE3C4"/>
      <ellipse cx="25" cy="34.5" rx="2.6" ry="3.4" fill="#3D2C1E"/>
      <ellipse cx="39" cy="34.5" rx="2.6" ry="3.4" fill="#3D2C1E"/>
      <circle cx="25.9" cy="33.3" r="0.9" fill="#fff"/>
      <circle cx="39.9" cy="33.3" r="0.9" fill="#fff"/>
      <path d="M32 40.2 m-2.6 0 a2.6 1.9 0 0 1 5.2 0 a2.6 1.9 0 0 1 -5.2 0" fill="#FF8FA3"/>
      <path d="M32 42 v1.6 M32 43.6 q-3 2.6 -5.4 0.4 M32 43.6 q3 2.6 5.4 0.4" stroke="#3D2C1E" stroke-width="1.4" fill="none" stroke-linecap="round"/>
      <path d="M12 37.5 h8 M12 42 h8 M52 37.5 h-8 M52 42 h-8" stroke="#FFD9B8" stroke-width="1.3" stroke-linecap="round" opacity="0.85"/>`,
  },
  {
    key: 'fox',
    label: '狐狸',
    from: '#FFA07A',
    to: '#F2683C',
    svg: bg('#FFA07A', '#F2683C') + `
      <path d="M12 26 L10 8 L28 18 Z" fill="#D9502B"/>
      <path d="M52 26 L54 8 L36 18 Z" fill="#D9502B"/>
      <path d="M15.5 23 L14 13.5 L23 18.5 Z" fill="#FFC7B0"/>
      <path d="M48.5 23 L50 13.5 L41 18.5 Z" fill="#FFC7B0"/>
      <path d="M32 20 C18 20 11 27 11 34 C11 44 20 53 32 53 C44 53 53 44 53 34 C53 27 46 20 32 20 Z" fill="#F9784B"/>
      <path d="M32 34 C24 34 18.5 40 18 46 C18 50.5 24 53 32 53 C40 53 46 50.5 46 46 C45.5 40 40 34 32 34 Z" fill="#FFF1E6"/>
      <ellipse cx="24.5" cy="33" rx="2.7" ry="3.3" fill="#3D2115"/>
      <ellipse cx="39.5" cy="33" rx="2.7" ry="3.3" fill="#3D2115"/>
      <circle cx="25.5" cy="31.8" r="0.9" fill="#fff"/>
      <circle cx="40.5" cy="31.8" r="0.9" fill="#fff"/>
      <path d="M32 41.5 m-3 0 a3 2.4 0 0 1 6 0 a3 2.4 0 0 1 -6 0" fill="#2E1A12"/>
      <path d="M32 44 v1.4" stroke="#2E1A12" stroke-width="1.4" stroke-linecap="round"/>
      <path d="M32 45.4 q-3.4 3 -6 0.6 M32 45.4 q3.4 3 6 0.6" stroke="#2E1A12" stroke-width="1.4" fill="none" stroke-linecap="round"/>`,
  },
  {
    key: 'panda',
    label: '熊猫',
    from: '#A8E6CF',
    to: '#4FBFA0',
    svg: bg('#A8E6CF', '#4FBFA0') + `
      <circle cx="15" cy="17" r="7.5" fill="#2F3B45"/>
      <circle cx="49" cy="17" r="7.5" fill="#2F3B45"/>
      <ellipse cx="32" cy="35" rx="21" ry="19" fill="#FFFFFF"/>
      <ellipse cx="23.5" cy="34" rx="6" ry="7" fill="#2F3B45"/>
      <ellipse cx="40.5" cy="34" rx="6" ry="7" fill="#2F3B45"/>
      <circle cx="24.5" cy="34.5" r="2.6" fill="#FFFFFF"/>
      <circle cx="39.5" cy="34.5" r="2.6" fill="#FFFFFF"/>
      <circle cx="24.5" cy="34.5" r="1.3" fill="#1B232A"/>
      <circle cx="39.5" cy="34.5" r="1.3" fill="#1B232A"/>
      <ellipse cx="32" cy="43" rx="3.2" ry="2.4" fill="#2F3B45"/>
      <path d="M32 45.4 v1.4 M32 46.8 q-3 2.6 -5.2 0.4 M32 46.8 q3 2.6 5.2 0.4" stroke="#2F3B45" stroke-width="1.4" fill="none" stroke-linecap="round"/>`,
  },
  {
    key: 'penguin',
    label: '企鹅',
    from: '#7FC8F8',
    to: '#3A8DDE',
    svg: bg('#7FC8F8', '#3A8DDE') + `
      <ellipse cx="32" cy="35" rx="18" ry="22" fill="#2F4858"/>
      <ellipse cx="32" cy="40" rx="12.5" ry="16" fill="#FFFFFF"/>
      <ellipse cx="24" cy="30" rx="3.4" ry="4" fill="#FFFFFF"/>
      <ellipse cx="40" cy="30" rx="3.4" ry="4" fill="#FFFFFF"/>
      <circle cx="24.3" cy="30.8" r="2" fill="#1E2E38"/>
      <circle cx="40.3" cy="30.8" r="2" fill="#1E2E38"/>
      <circle cx="25" cy="29.8" r="0.7" fill="#fff"/>
      <circle cx="41" cy="29.8" r="0.7" fill="#fff"/>
      <path d="M32 33 l-4 3.4 h8 Z" fill="#F5A623"/>
      <path d="M32 36.4 v1.6" stroke="#1E2E38" stroke-width="1.2" stroke-linecap="round"/>
      <path d="M14 38 c-3 1.5 -3 6 -1 8 c1.6 1.6 4 1 5 -1 l1.5 -5 Z" fill="#2F4858"/>
      <path d="M50 38 c3 1.5 3 6 1 8 c-1.6 1.6 -4 1 -5 -1 l-1.5 -5 Z" fill="#2F4858"/>
      <ellipse cx="27" cy="56.5" rx="5" ry="2.6" fill="#F5A623"/>
      <ellipse cx="37" cy="56.5" rx="5" ry="2.6" fill="#F5A623"/>`,
  },
  {
    key: 'owl',
    label: '猫头鹰',
    from: '#B8A5F2',
    to: '#7C5DD6',
    svg: bg('#B8A5F2', '#7C5DD6') + `
      <path d="M14 22 L12 8 L26 16 Z" fill="#6A4BC4"/>
      <path d="M50 22 L52 8 L38 16 Z" fill="#6A4BC4"/>
      <ellipse cx="32" cy="36" rx="20" ry="21" fill="#8E72E0"/>
      <ellipse cx="32" cy="42" rx="12" ry="12" fill="#C9B8F5"/>
      <circle cx="23.5" cy="31" r="7.5" fill="#FFFFFF"/>
      <circle cx="40.5" cy="31" r="7.5" fill="#FFFFFF"/>
      <circle cx="24" cy="31.5" r="3.6" fill="#2E2545"/>
      <circle cx="40" cy="31.5" r="3.6" fill="#2E2545"/>
      <circle cx="25.2" cy="30.2" r="1.3" fill="#fff"/>
      <circle cx="41.2" cy="30.2" r="1.3" fill="#fff"/>
      <path d="M32 35 l-3.2 3.6 h6.4 Z" fill="#F5A623"/>
      <path d="M22 48 q10 5 20 0" stroke="#6A4BC4" stroke-width="1.6" fill="none" stroke-linecap="round" opacity="0.5"/>`,
  },
  {
    key: 'rabbit',
    label: '兔子',
    from: '#FFB3D1',
    to: '#FF7BAD',
    svg: bg('#FFB3D1', '#FF7BAD') + `
      <ellipse cx="23" cy="16" rx="5" ry="11" fill="#FFE3EF" transform="rotate(-12 23 16)"/>
      <ellipse cx="41" cy="16" rx="5" ry="11" fill="#FFE3EF" transform="rotate(12 41 16)"/>
      <ellipse cx="23" cy="17" rx="2.6" ry="8" fill="#FF9CC4" transform="rotate(-12 23 17)"/>
      <ellipse cx="41" cy="17" rx="2.6" ry="8" fill="#FF9CC4" transform="rotate(12 41 17)"/>
      <ellipse cx="32" cy="39" rx="18" ry="16" fill="#FFF0F6"/>
      <ellipse cx="25.5" cy="37" rx="2.6" ry="3.2" fill="#4A2B3A"/>
      <ellipse cx="38.5" cy="37" rx="2.6" ry="3.2" fill="#4A2B3A"/>
      <circle cx="26.4" cy="35.8" r="0.9" fill="#fff"/>
      <circle cx="39.4" cy="35.8" r="0.9" fill="#fff"/>
      <path d="M32 43 m-2.4 0 a2.4 1.8 0 0 1 4.8 0 a2.4 1.8 0 0 1 -4.8 0" fill="#FF6F9C"/>
      <path d="M32 44.8 v1.6" stroke="#4A2B3A" stroke-width="1.4" stroke-linecap="round"/>
      <path d="M32 46.4 q-3.2 2.4 -5.4 0.2 M32 46.4 q3.2 2.4 5.4 0.2" stroke="#4A2B3A" stroke-width="1.4" fill="none" stroke-linecap="round"/>
      <ellipse cx="18" cy="41" rx="3.4" ry="2.2" fill="#FFB3D1" opacity="0.7"/>
      <ellipse cx="46" cy="41" rx="3.4" ry="2.2" fill="#FFB3D1" opacity="0.7"/>`,
  },
  {
    key: 'robot',
    label: '机器人',
    from: '#7EE8FA',
    to: '#2BB0E8',
    svg: bg('#7EE8FA', '#2BB0E8') + `
      <path d="M32 6 v6" stroke="#1B7FB8" stroke-width="2" stroke-linecap="round"/>
      <circle cx="32" cy="6" r="3" fill="#FFD166"/>
      <rect x="12" y="14" width="40" height="36" rx="10" fill="#F2FAFF"/>
      <rect x="12" y="14" width="40" height="36" rx="10" fill="none" stroke="#1B7FB8" stroke-width="1.5" opacity="0.35"/>
      <circle cx="24" cy="30" r="4.6" fill="#1B7FB8"/>
      <circle cx="40" cy="30" r="4.6" fill="#1B7FB8"/>
      <circle cx="25.4" cy="28.6" r="1.7" fill="#7EE8FA"/>
      <circle cx="41.4" cy="28.6" r="1.7" fill="#7EE8FA"/>
      <rect x="22" y="39" width="20" height="4" rx="2" fill="#1B7FB8" opacity="0.75"/>
      <path d="M8 26 v10 M56 26 v10" stroke="#1B7FB8" stroke-width="2.4" stroke-linecap="round"/>`,
  },
  {
    key: 'rocket',
    label: '火箭',
    from: '#8B9DFF',
    to: '#5B6EE1',
    svg: bg('#8B9DFF', '#5B6EE1') + `
      <path d="M32 6 C40 14 44 24 44 34 L44 42 L20 42 L20 34 C20 24 24 14 32 6 Z" fill="#FFFFFF"/>
      <path d="M32 6 C36 12 39 20 40 28 L32 30 Z" fill="#E8EDFF"/>
      <circle cx="32" cy="27" r="5.5" fill="#5B6EE1"/>
      <circle cx="32" cy="27" r="3.6" fill="#B9C6FF"/>
      <path d="M20 34 L10 46 L20 43 Z" fill="#FF7B7B"/>
      <path d="M44 34 L54 46 L44 43 Z" fill="#FF7B7B"/>
      <rect x="26" y="42" width="12" height="4" rx="1.5" fill="#FFB86B"/>
      <path d="M27 48 q5 8 3 12 q-2 -5 -6 -7 Z" fill="#FFD166"/>
      <path d="M37 48 q-5 8 -3 12 q2 -5 6 -7 Z" fill="#FFB86B"/>`,
  },
  {
    key: 'chip',
    label: '芯片',
    from: '#86EFAC',
    to: '#22C55E',
    svg: bg('#86EFAC', '#22C55E') + `
      <path d="M22 8 v6 M32 8 v6 M42 8 v6 M22 56 v-6 M32 56 v-6 M42 56 v-6 M8 22 h6 M8 32 h6 M8 42 h6 M56 22 h-6 M56 32 h-6 M56 42 h-6" stroke="#15803D" stroke-width="2.6" stroke-linecap="round"/>
      <rect x="14" y="14" width="36" height="36" rx="6" fill="#1F2937"/>
      <rect x="23" y="23" width="18" height="18" rx="3" fill="#4ADE80"/>
      <path d="M23 32 h4 M41 32 h-4 M32 23 v4 M32 41 v-4" stroke="#15803D" stroke-width="1.6" stroke-linecap="round"/>
      <circle cx="32" cy="32" r="2.2" fill="#1F2937"/>`,
  },
  {
    key: 'dev',
    label: '开发者',
    from: '#A5B4FC',
    to: '#6366F1',
    svg: bg('#A5B4FC', '#6366F1') + `
      <circle cx="32" cy="26" r="13" fill="#FFE0C2"/>
      <path d="M19 23 C19 14 45 14 45 23 C45 20 41 18 32 18 C23 18 19 20 19 23 Z" fill="#3F3A5A"/>
      <circle cx="26.5" cy="27" r="4.6" fill="#FFFFFF"/>
      <circle cx="37.5" cy="27" r="4.6" fill="#FFFFFF"/>
      <path d="M31 27 h2" stroke="#3F3A5A" stroke-width="1.6"/>
      <circle cx="26.8" cy="27.3" r="2.1" fill="#2E2A45"/>
      <circle cx="37.8" cy="27.3" r="2.1" fill="#2E2A45"/>
      <path d="M29 35 q3 2.6 6 0" stroke="#C77B5C" stroke-width="1.6" fill="none" stroke-linecap="round"/>
      <path d="M32 39 C22 39 15 46 15 56 L49 56 C49 46 42 39 32 39 Z" fill="#FFFFFF"/>
      <path d="M32 39 L26 47 L32 56 L38 47 Z" fill="#EEF2FF"/>
      <path d="M26 47 h12 M32 47 v9" stroke="#C7D2FE" stroke-width="1.4"/>`,
  },
  {
    key: 'ninja',
    label: '运维侠',
    from: '#94A3B8',
    to: '#475569',
    svg: bg('#94A3B8', '#475569') + `
      <path d="M32 10 C18 10 12 20 12 30 C12 40 20 52 32 52 C44 52 52 40 52 30 C52 20 46 10 32 10 Z" fill="#334155"/>
      <path d="M12 28 C12 20 20 14 32 14 C44 14 52 20 52 28 C52 26 44 22 32 22 C20 22 12 26 12 28 Z" fill="#1E293B"/>
      <path d="M15 26 C22 20 42 20 49 26 L49 30 C42 25 22 25 15 30 Z" fill="#64748B" opacity="0.55"/>
      <path d="M19 33 C23 30.5 28.5 30.5 31 33 C28.5 35.5 23 35.5 19 33 Z" fill="#FFFFFF"/>
      <path d="M33 33 C35.5 30.5 41 30.5 45 33 C41 35.5 35.5 35.5 33 33 Z" fill="#FFFFFF"/>
      <circle cx="24.5" cy="33" r="2.2" fill="#1E293B"/>
      <circle cx="39.5" cy="33" r="2.2" fill="#1E293B"/>
      <path d="M22 44 C26 47 38 47 42 44 C38 49 26 49 22 44 Z" fill="#1E293B" opacity="0.5"/>
      <path d="M48 12 L54 8 L52 15 Z" fill="#1E293B" opacity="0.4"/>`,
  },
  {
    key: 'ghost',
    label: '小幽灵',
    from: '#C4B5FD',
    to: '#8B5CF6',
    svg: bg('#C4B5FD', '#8B5CF6') + `
      <path d="M32 8 C19 8 11 18 11 31 L11 50 C11 52.5 12.5 54 14.5 54 C16.5 54 18 52.5 18 50 L18 46 L21.5 50 C23 51.8 25.6 51.8 27 50 L29 47.5 L32 51 C33.4 52.8 36 52.8 37.4 51 L40.5 47.5 L43 50 C44.4 51.8 47 51.8 48.4 50 L50 48 L50 54 C50 55.6 51.4 57 53 57 C54.6 57 56 55.6 56 54 L56 31 C56 18 45 8 32 8 Z" fill="#FFFFFF"/>
      <ellipse cx="25" cy="30" rx="4" ry="5" fill="#4C3A7A"/>
      <ellipse cx="39" cy="30" rx="4" ry="5" fill="#4C3A7A"/>
      <circle cx="26.2" cy="28.4" r="1.5" fill="#fff"/>
      <circle cx="40.2" cy="28.4" r="1.5" fill="#fff"/>
      <ellipse cx="32" cy="39" rx="3.4" ry="4.2" fill="#4C3A7A" opacity="0.85"/>
      <ellipse cx="16" cy="36" rx="2.6" ry="3.2" fill="#DDD6FE" opacity="0.7"/>
      <ellipse cx="48" cy="36" rx="2.6" ry="3.2" fill="#DDD6FE" opacity="0.7"/>`,
  },
]

/** 所有内置头像的 key 列表，即 `av:<key>` 的取值集合 */
export const LAN_AVATAR_KEYS = LAN_AVATARS.map((a) => a.key)

/** 默认头像（首个） */
export const DEFAULT_LAN_AVATAR = `av:${LAN_AVATARS[0].key}`

/** 从头像字段值解析出内置头像定义，非内置头像返回 null */
export function resolveLanAvatar(value: string | undefined | null): LanAvatarDef | null {
  if (!value || !value.startsWith('av:')) {return null}
  const key = value.slice(3)
  return LAN_AVATARS.find((a) => a.key === key) || null
}
