# Nginx 模块重构计划

## 目标
将 nginxWebUI 表单驱动的 nginx 图形化管理功能移植到 SuperTool，以预设为配置单元，用结构化数据生成 nginx 配置。

## 架构

```
NginxPreset (已有, 作用域单元)
  ├── NginxServer[]      (server 块: proxyType/listen/server_name/SSL/location[])
  │   └── NginxLocation[]  (一项 location: path/proxy_pass/root/upstream_ref/websocket/header)
  ├── NginxUpstream[]    (upstream 块: 名称/策略/server[])
  │   └── NginxUpstreamServer[] (address/weight/backup/down)
  ├── NginxHttpParam[]   (http 级指令 key-value, 可排序)
  ├── NginxStream[]      (stream {} 内的 server 块)
  ├── NginxCert[]        (SSL 证书: name/pem/key)
  ├── NginxTemplate[]    (可复用配置片段)
  └── NginxBasicSetting  (每个预设一行: worker_processes 等)

配置生成流程:
  表单数据 (SQLite) → 配置生成器 → nginx 配置文本 → 预览 → 部署到远程
```

## 进度追踪

### ✅ Phase 0 — 方案设计 (当前)
- [x] 数据结构设计
- [x] UI 布局设计
- [x] 配置生成器设计
- [ ] 编写此计划文件

### 🔄 Phase 1 — Rust DB 模型定义
- [ ] 数据库表迁移 (init_db 新增 8 张表)
- [ ] core/src/db/nginx.rs — 新增 NginxServer/Location/Upstream/UpstreamServer/HttpParam/Stream/Cert/Template/Basic 模型 + CRUD
- [ ] core/src/logic/nginx.rs — 接入新 CRUD 方法到 CoreService

### 🔄 Phase 2 — 配置生成器
- [ ] core/src/logic/nginx_generator.rs — 从结构化数据生成 nginx 配置文本
  - [ ] generate_http_config (http 级指令 + upstream + server)
  - [ ] generate_stream_config
  - [ ] generate_full_config (basic + http + stream)
  - [ ] cert pem/key 嵌入或引用处理

### 🔄 Phase 3 — Tauri Commands
- [ ] tauri/src/commands/nginx.rs — 新增 CRUD commands
  - [ ] get_servers_by_preset / add_server / update_server / delete_server / set_servers_order
  - [ ] get_locations_by_server / add_location / update_location / delete_location
  - [ ] get_upstreams_by_preset / add_upstream / update_upstream / delete_upstream
  - [ ] get_upstream_servers / add/update/delete upstream_server
  - [ ] get_http_params / add/update/delete http_param
  - [ ] get_streams_by_preset / add/update/delete stream
  - [ ] cert CRUD + upload_cert
  - [ ] template CRUD
  - [ ] get/update basic_settings
  - [ ] generate_nginx_config(presetId) — 核心: 读取所有数据 → 生成配置文本
- [ ] tauri/src/main.rs — generate_handler 注册新 commands

### 🔄 Phase 4 — tauri-api.ts 前端接口
- [ ] src/utils/tauri-api.ts — 新增 nginx 结构化 API 方法
  - [ ] NginxServer / Location / Upstream 等 CRUD
  - [ ] generateNginxConfig
  - [ ] cert management

### 🔄 Phase 5 — NginxManager.vue 重构
- [ ] 左侧: 预设列表 (已有)
- [ ] 右侧: 标签页布局 (Tab 组件)
  - [ ] Server — server 管理主页面
  - [ ] Upstream — upstream 管理
  - [ ] HTTP — http 级参数
  - [ ] Stream — stream 配置
  - [ ] Cert — 证书管理
  - [ ] 模板 — 配置模板
  - [ ] 基本设置 — 全局 nginx 参数
- [ ] 底部: 预览 + 测试 + 发布 + 版本历史 (已有)

### 🔄 Phase 6 — 前端子页面
- [ ] NginxServerPage.vue — server 列表 + 编辑弹窗
  - [ ] 表格查看: proxyType / listen / server_name / SSL / enabled
  - [ ] 编辑弹窗: proxyType, listen(ip+port+def+ipv6+proxy_protocol), server_name, ssl(pem/key/rewrite/http2/protocols), deny_allow
  - [ ] Location 子表: enable/path/type(proxy_pass/root/upstream/blank/return)/value/rootPath/header/websocket/cros/upstream_ref
- [ ] NginxUpstreamPage.vue — upstream 列表 + 编辑弹窗
  - [ ] 表格: proxyType/name/strategy/server_count
  - [ ] 编辑: name, proxy_type, strategy(polling/ip_hash/least_conn/random), server 子表(address/weight/max_fails/fail_timeout/backup/down), 其他指令
- [ ] NginxHttpParamPage.vue — http 参数 key-value 编辑器
  - [ ] 表格: sort/enable/name/value + up/down 排序
- [ ] NginxStreamPage.vue — stream 管理
  - [ ] 类似 server 但 proxyType 固定 TCP/UDP + upstream_ref
- [ ] NginxCertPage.vue — 证书管理
  - [ ] 列表: name/domain/pem_path/key_path
  - [ ] 上传: 上传 pem/key 文件到远程服务器
- [ ] NginxTemplatePage.vue — 配置模板
  - [ ] 列表 + 编辑弹窗 (文本编辑器)
- [ ] NginxBasicPage.vue — 基本设置
  - [ ] worker_processes, worker_connections, error_log, pid, events 等

### 🔄 Phase 7 — 集成与清理
- [ ] 保留 NginxConfigVersion (版本历史) 逻辑
- [ ] 保留 deploy_nginx_config / test_nginx_config / fetch_nginx_config 逻辑
- [ ] 新的 generate_config 替代手动编辑(可选, 与原 visual/raw 模式并存)
- [ ] 删除旧的 NginxStructuredEditor.vue / ServerBlockCard.vue (保留 parser 用于反向解析)
- [ ] cargo check --workspace + vue-tsc --noEmit

## 待确认事项
- 证书文件: 上传到远程服务器(像 nginxWebUI 一样)还是本地管理？ → 上传到远程
- 配置预览: 生成后直接展示在页面还是跳转到 raw 编辑器？ → 直接展示预览
- 是否需要反向解析（配置文本 → 表单）？ → 后续再说, 现有 parser 保留
- 静态网站管理(WWW) 是否需要？ → 功能类似 server 模板, 通过 server 表单解决

## 文件影响清单

### 新增文件
```
core/src/logic/nginx_generator.rs       — 配置生成器 (300-400 行)
src/views/nginx/components/
  NginxServerPage.vue                   — Server 管理 (600-800 行)
  NginxUpstreamPage.vue                 — Upstream 管理 (400-500 行)
  NginxHttpParamPage.vue                — HTTP 参数 (200-300 行)
  NginxStreamPage.vue                   — Stream 配置 (200-300 行)
  NginxCertPage.vue                     — 证书管理 (200-300 行)
  NginxTemplatePage.vue                 — 模板管理 (150-200 行)
  NginxBasicPage.vue                    — 基本设置 (100-150 行)
```

### 修改文件
```
core/src/db/mod.rs                      — init_db 新增表
core/src/db/nginx.rs                    — 新增 8 个模型的 CRUD
core/src/logic/mod.rs                   — 无需改 (nginx 已注册)
core/src/logic/nginx.rs                 — 新增 CoreService 方法
tauri/src/commands/mod.rs               — 无需改 (nginx 已注册)
tauri/src/commands/nginx.rs             — 新增 Tauri commands
tauri/src/main.rs                       — generate_handler 注册
src/utils/tauri-api.ts                  — 新增 API 接口 + 装配
src/views/nginx/NginxManager.vue        — 重构为标签页布局
src/views/nginx/components/
  NginxStructuredEditor.vue             — 保留(作为 raw 预览模式)
  ServerBlockCard.vue                   — 保留(以备后续反向解析使用)
```

### 删除文件
无 (保留所有已有组件, 但可能弃用)

## 关键数据模型

### NginxServer
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT FK | 所属预设 |
| proxyType | INTEGER | 0=http, 1=TCP, 2=UDP |
| listen | TEXT | 监听配置 |
| ip | TEXT | 绑定 IP |
| def | BOOL | default_server |
| ipv6 | BOOL | IPV6 模式 |
| proxyProtocol | BOOL | proxy_protocol |
| serverName | TEXT | server_name |
| ssl | BOOL | 是否启用 SSL |
| certId | TEXT | 关联证书 ID |
| rewrite | BOOL | HTTP→HTTPS 重写 |
| rewriteListen | TEXT | 重写监听端口 |
| http2 | INTEGER | 0=no, 1=old, 2=new |
| protocols | TEXT | TLSv1 TLSv1.1 TLSv1.2 TLSv1.3 |
| passwordId | TEXT | 访问密码 ID |
| denyAllow | TEXT | IP 黑白名单 |
| denyId | TEXT | 黑名单 ID |
| allowId | TEXT | 白名单 ID |
| proxyUpstreamId | TEXT | TCP/UDP 转发的 upstream |
| descr | TEXT | 描述 |
| enable | BOOL | 是否启用 |
| sort | INTEGER | 排序号 |
| paramJson | TEXT | 额外参数(JSON) |

### NginxLocation
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| serverId | TEXT FK | 所属 server |
| enable | BOOL | 启用 |
| path | TEXT | location 路径 |
| type | INTEGER | 0=proxy_pass, 1=root, 2=upstream, 3=blank, 4=return |
| value | TEXT | 目标值 |
| upstreamType | INTEGER | 0=自动, 1=手动 |
| upstreamId | TEXT | 关联 upstream |
| upstreamPath | TEXT | upstream 附加路径 |
| rootPath | TEXT | 根目录 |
| rootPage | TEXT | 默认页面 |
| rootType | TEXT | 根类型 |
| header | BOOL | 是否携带 Host 头 |
| websocket | BOOL | WebSocket 支持 |
| cros | BOOL | CORS 跨域 |
| headerHost | TEXT | Host 头值(off/default) |
| returnUrl | TEXT | 重定向 URL |
| returnPath | BOOL | 保留路径 |
| paramJson | TEXT | 额外参数 |
| sort | INTEGER | 排序 |

### NginxUpstream
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT FK | |
| name | TEXT | upstream 名称 |
| proxyType | INTEGER | 0=http, 1=TCP, 2=UDP |
| strategy | TEXT | 负载策略 |
| descr | TEXT | 描述 |
| paramJson | TEXT | 额外参数 |

### NginxUpstreamServer
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| upstreamId | TEXT FK | |
| address | TEXT | IP:PORT |
| port | INTEGER | 端口 |
| weight | INTEGER | 权重 |
| maxFails | INTEGER | |
| failTimeout | TEXT | |
| maxConns | INTEGER | |
| backup | BOOL | |
| down | BOOL | |
| sort | INTEGER | |

### NginxHttpParam
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT FK | |
| name | TEXT | 指令名 |
| value | TEXT | 值 |
| enable | BOOL | |
| sort | INTEGER | |

### NginxCert
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT FK | |
| name | TEXT | 证书名称 |
| pem | TEXT | 远程 PEM 路径 |
| key | TEXT | 远程 KEY 路径 |
| domain | TEXT | 关联域名 |

### NginxTemplate
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT FK | |
| name | TEXT | 模板名 |
| content | TEXT | 模板内容 |

### NginxBasicSetting
| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| presetId | TEXT UNIQUE | 每个预设一行 |
| workerProcesses | TEXT | |
| workerConnections | INTEGER | |
| errorLog | TEXT | |
| errorLogLevel | TEXT | |
| pid | TEXT | |
| events | TEXT | events 块内容 |

## 配置生成器输出示例

```
# 基本设置
worker_processes  auto;
error_log  /var/log/nginx/error.log warn;
pid        /var/run/nginx.pid;
events {
    worker_connections  1024;
}

# HTTP
http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;
    
    # HTTP 级参数
    sendfile        on;
    keepalive_timeout  65;
    
    # Upstream 块
    upstream backend {
        server 127.0.0.1:8080 weight=5;
        server 127.0.0.1:8081 weight=3 backup;
    }
    
    # Server 块
    server {
        listen       80;
        server_name  example.com;
        root         /var/www/html;
        
        location / {
            proxy_pass http://backend;
        }
        
        location /api {
            proxy_pass http://127.0.0.1:3000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
    
    # SSL Server
    server {
        listen       443 ssl;
        server_name  example.com;
        ssl_certificate      /etc/nginx/ssl/example.pem;
        ssl_certificate_key  /etc/nginx/ssl/example.key;
        
        location / {
            root /var/www/html;
        }
    }
}

# Stream (TCP/UDP)
stream {
    server {
        listen 3306;
        proxy_pass backend_mysql;
    }
}
```
