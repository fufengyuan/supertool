export interface DevTool {
  id: string
  name: string
  icon: string
  category: 'crypto' | 'encode' | 'time' | 'code' | 'text' | 'network' | 'convert' | 'format' | 'misc'
  description: string
  offline: boolean
  /** 搜索关键词：英文缩写、拼音首字母、常见别名 */
  keywords: string
}

export const DEV_TOOL_REGISTRY: DevTool[] = [
  // === 加密/哈希 ===
  { id: 'crypto', name: '哈希计算', icon: 'lock', category: 'crypto', description: 'MD5, SHA1, SHA256, SHA512, SM3, 批量处理, 文件哈希', offline: true, keywords: 'hash md5 sha sm3 sj hh' },
  { id: 'encrypt', name: '加密/解密', icon: 'key', category: 'crypto', description: 'AES, DES, RC4, Rabbit, TripleDES, SM2, SM4, Bcrypt', offline: true, keywords: 'encrypt decrypt aes des rc4 sm2 sm4 bcrypt jiami jiemi' },
  { id: 'navicat', name: 'Navicat密码', icon: 'database', category: 'crypto', description: 'Navicat 12+ 保存的数据库密码加解密（.ncx）', offline: true, keywords: 'navicat password ncx 数据库 密码 jiemi jiami' },

  // === 编码/转换 ===
  { id: 'base64', name: 'BASE64编码', icon: 'file', category: 'encode', description: '编码, 解码, 支持文件', offline: true, keywords: 'base64 encode decode bm' },
  { id: 'url', name: 'URL编码', icon: 'link', category: 'encode', description: '编码, 解码', offline: true, keywords: 'url encode decode uri bm' },
  { id: 'unicode', name: 'Unicode转换', icon: 'globe', category: 'encode', description: '双向转换, emoji, HTML实体, CSS实体', offline: true, keywords: 'unicode emoji html entity zh' },
  { id: 'hex', name: 'Hex转换', icon: 'hash', category: 'encode', description: 'Hex/String, Hex/Base64 互转', offline: true, keywords: 'hex hex16 zh' },
  { id: 'html', name: 'HTML编码', icon: 'tag', category: 'encode', description: 'HTML 实体编码/解码', offline: true, keywords: 'html entity encode bm' },
  { id: 'jwt', name: 'JWT解码', icon: 'unlock', category: 'encode', description: 'Header, Payload 解码', offline: true, keywords: 'jwt token decode jm' },

  // === 时间/日期 ===
  { id: 'time', name: '时间戳', icon: 'clock', category: 'time', description: '时间戳双向转换, 毫秒, 时区', offline: true, keywords: 'timestamp unix time sjc sj' },
  { id: 'timecalc', name: '时间计算器', icon: 'calendar', category: 'time', description: '日期加减, 间隔计算', offline: true, keywords: 'time calc date sjjsq' },
  { id: 'crontab', name: 'Crontab', icon: 'timer', category: 'time', description: '规则, 校验, 例子', offline: true, keywords: 'crontab cron schedule dingshi' },

  // === 代码/JSON ===
  { id: 'codeformat', name: '代码格式化', icon: 'code', category: 'code', description: 'JS, TS, HTML, CSS, LESS, SCSS, Vue, JSON, YAML, SQL 等', offline: true, keywords: 'format prettier code ddmgs' },
  { id: 'json', name: 'JSON工具', icon: 'clipboard', category: 'code', description: '格式化, 校验, 压缩, 转义, jsonpath, 转GET参数, 转Java/C#/Go/Dart', offline: true, keywords: 'json format validate minify jsonpath jsongj' },
  { id: 'serial', name: '序列化转换', icon: 'refresh', category: 'code', description: 'JSON, XML, YAML, PHP Array, PHP Serialize, Properties', offline: true, keywords: 'serialize xml yaml php xlh' },

  // === 文本处理 ===
  { id: 'regex', name: '正则表达式', icon: 'search', category: 'text', description: '匹配, 查找, 替换', offline: true, keywords: 'regex regexp regular expression zz zzbb' },
  { id: 'text', name: '文本处理', icon: 'notebook', category: 'text', description: '大小写, 标点, 简繁, 统计, 去重, 排序, 过滤', offline: true, keywords: 'text case sort dedup wbencl' },
  { id: 'diff', name: '文本对比', icon: 'book', category: 'text', description: '行, 单词, CSS 差异对比', offline: true, keywords: 'diff compare contrast wbdb' },
  { id: 'variable', name: '变量名转换', icon: 'gitBranch', category: 'text', description: '驼峰, 蛇形, 横线, 常量 等格式互转', offline: true, keywords: 'variable camel snake kebab case blmzh' },
  { id: 'pinyin', name: '汉字转拼音', icon: 'keyboard', category: 'text', description: '声调, 首字母, 分隔符', offline: true, keywords: 'pinyin hzpz' },
  { id: 'ascii', name: 'ASCII编码', icon: 'terminal', category: 'text', description: '十进制, 十六进制, 八进制, 二进制, 字符串', offline: true, keywords: 'ascii ascii' },

  // === 网络工具 ===
  { id: 'ip', name: 'IP地址查询', icon: 'mapPin', category: 'network', description: '运营商, 城市', offline: false, keywords: 'ip address location ipcz' },
  { id: 'ipcalc', name: 'IP网络计算器', icon: 'server', category: 'network', description: '子网掩码, IP进制换算', offline: true, keywords: 'ip calc subnet mask ipwljsq' },
  { id: 'armhex', name: 'ARM/HEX', icon: 'tool', category: 'network', description: 'ARM指令与HEX互转', offline: false, keywords: 'arm hex instruction armhex' },
  { id: 'ws', name: 'WebSocket调试', icon: 'plug', category: 'network', description: 'WebSocket 在线调试', offline: false, keywords: 'websocket ws debug wsdc' },
  { id: 'apidebugger', name: '接口调试', icon: 'zap', category: 'network', description: 'HTTP 接口调试，仿 Postman', offline: false, keywords: 'api http debug postman jkts' },

  // === 进制/单位 ===
  { id: 'baseconv', name: '进制转换', icon: 'pencil', category: 'convert', description: '2-64进制互转', offline: true, keywords: 'base convert radix jzzh' },
  { id: 'unit', name: '单位换算', icon: 'layers', category: 'convert', description: '长度, 面积, 体积, 质量, 温度, 速度, 数据存储等', offline: true, keywords: 'unit convert dwhs' },
  { id: 'complement', name: '原码/反码/补码', icon: 'ban', category: 'convert', description: '二进制补码运算', offline: true, keywords: 'complement binary ymfm bm' },

  // === 其他工具 ===
  { id: 'qrcode', name: '二维码', icon: 'camera', category: 'misc', description: '生成, 解析', offline: true, keywords: 'qrcode ewm' },
  { id: 'barcode', name: '条形码', icon: 'grid', category: 'misc', description: '生成多种格式条形码', offline: true, keywords: 'barcode txm' },
  { id: 'calculator', name: '计算器', icon: 'calculator', category: 'misc', description: '基本运算, 科学计算, 历史记录', offline: true, keywords: 'calculator calc jsq' },
  { id: 'random', name: '随机生成器', icon: 'sparkles', category: 'misc', description: '批量生成, 特殊字符', offline: true, keywords: 'random generate sjscq' },
  { id: 'uuid', name: 'UUID生成', icon: 'plus', category: 'misc', description: '在线生成 UUID v4', offline: true, keywords: 'uuid guid uuidsc' },
  { id: 'html2md', name: 'HTML转Markdown', icon: 'download', category: 'misc', description: '输入网址或粘贴HTML，转换为 Markdown', offline: false, keywords: 'html markdown html2md hzzmd' },
  { id: 'image', name: '图像处理', icon: 'image', category: 'misc', description: '压缩, 转格式, 尺寸调整等', offline: true, keywords: 'image photo compress resize tuxs tuxiangcl' },
]

export function getToolById(id: string): DevTool | undefined {
  return DEV_TOOL_REGISTRY.find(t => t.id === id)
}

export function getToolsByCategory(category: string): DevTool[] {
  return DEV_TOOL_REGISTRY.filter(t => t.category === category)
}
