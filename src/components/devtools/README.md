# DevTools 模块结构

src/components/devtools/
├── DevTools.vue          # 主入口：左侧工具导航 + 右侧内容区
├── tools/
│   ├── CryptoTool.vue        # 哈希 (MD5/SHA1/SHA256/SHA512/SM3)
│   ├── EncryptTool.vue       # 加密解密 (AES/DES/RC4/Rabbit/TripleDES/SM2/SM4/Bcrypt)
│   ├── Base64Tool.vue        # BASE64 编码/解码
│   ├── UrlTool.vue           # URL 编码/解码
│   ├── UnicodeTool.vue       # Unicode 双向转换 (emoji, html 实体, css 实体)
│   ├── HexTool.vue           # Hex/String 转换 + Hex/Base64 转换
│   ├── TimeTool.vue          # 时间戳双向转换 + 时区
│   ├── CrontabTool.vue       # Crontab 校验 + 规则 + 例子
│   ├── QrCodeTool.vue        # 二维码 生成/解析
│   ├── BarcodeTool.vue       # 条形码 生成
│   ├── PinyinTool.vue        # 汉字转拼音
│   ├── IpTool.vue            # IP地址查询
│   ├── IpCalcTool.vue        # IP网络计算器
│   ├── CodeFormatTool.vue    # 代码格式化 (js/ts/html/css/less/scss/vue/json/yaml/sql 等)
│   ├── JsonTool.vue          # JSON 工具 (格式化/校验/压缩/转义/jsonpath/Protobuf/转GET参数/转语言)
│   ├── SerialTool.vue        # 序列化转换 (json/xml/yaml/phpArray/phpSerialize/properties)
│   ├── DiffTool.vue          # 文本差异化对比
│   ├── RegexTool.vue         # 正则表达式
│   ├── RandomTool.vue        # 随机字符生成器
│   ├── TextTool.vue          # 文本处理 (大小写/标点/简繁/统计/去重/排序/过滤)
│   ├── HtmlEntityTool.vue    # HTML 编码
│   ├── BaseConvTool.vue      # 进制转换 (2-64进制)
│   ├── VariableTool.vue      # 变量名格式转换
│   ├── JwtTool.vue           # JWT 解码
│   ├── AsciiTool.vue         # ASCII 编码转换
│   ├── ComplementTool.vue    # 原码/反码/补码
│   ├── ArmHexTool.vue        # ARM/HEX 互转
│   ├── UnitTool.vue          # 单位换算
│   ├── TimeCalcTool.vue      # 时间计算器
│   ├── UuidTool.vue          # UUID 生成
│   └── WsTool.vue            # WebSocket 调试
└── DevToolRegistry.ts    # 工具注册表
