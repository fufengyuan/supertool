---
name: dispatch-to-kanban
description: 将软件开发任务调度到 Kanban 看板，分配给专门的 Worker Profile 异步执行
trigger: 用户请求涉及代码开发、测试、重构、文档、部署、调研等软件工程任务
version: 1.0.0
author: SuperTool
---

# 软件开发 Kanban 任务调度 Skill

当用户请求涉及以下软件工程场景时，自动将任务调度到 Kanban 看板：

## 适用场景

| 场景 | 说明 | 推荐处理方式 |
|------|------|-------------|
| **代码开发** | 新功能、新模块开发 | Kanban 异步 |
| **代码重构** | 大规模重构、模块拆分 | Kanban 异步 |
| **技术调研** | 技术选型、方案调研 | Kanban 异步 |
| **文档撰写** | 技术文档、API 文档 | Kanban 异步 |
| **测试编写** | 单测、集成测试 | Kanban 异步 |
| **Bug 修复** | 复杂 Bug、多模块 Bug | Kanban 异步 |
| **部署发布** | CI/CD、环境部署 | Kanban 异步 |
| **依赖升级** | 大版本升级、依赖更新 | Kanban 异步 |

**不适用场景**（使用 delegate_task 同步处理）：
- 快速问答、代码片段生成
- 小范围修改（<50行）
- 紧急任务需立即反馈
- 子任务间有强依赖需实时协调

## 软件开发专用 Profile

### 预定义 Profile 类型

| Profile | 专长 | 典型任务 |
|---------|------|---------|
| `coder` | 代码开发 | 实现新功能、编写模块、重构代码 |
| `reviewer` | 代码审查 | PR review、代码审计、安全检查 |
| `tester` | 测试工程 | 单测编写、集成测试、E2E 测试 |
| `researcher` | 技术调研 | 技术选型、文档搜索、方案分析 |
| `writer` | 文档撰写 | API 文档、技术博客、README |
| `devops` | 运维部署 | CI/CD、Docker、K8s、环境配置 |
| `debugger` | Bug 修复 | 复杂 Bug 定位、性能调优 |

### Profile 创建模板

```bash
# 创建 coder profile（代码开发）
hermes profile create coder
hermes profile describe coder --set "专注代码开发，擅长实现新功能、重构代码、解决复杂编程问题。工作目录: ~/WebstormProjects/supertool"

# 创建 reviewer profile（代码审查）
hermes profile create reviewer
hermes profile describe reviewer --set "专注代码审查，擅长 PR review、代码审计、安全检查、最佳实践建议"

# 创建 tester profile（测试工程）
hermes profile create tester
hermes profile describe tester --set "专注测试工程，擅长编写单元测试、集成测试、E2E测试，确保代码质量"

# 创建 researcher profile（技术调研）
hermes profile create researcher
hermes profile describe researcher --set "专注技术调研，擅长搜索文档、分析技术方案、对比技术选型"

# 创建 writer profile（文档撰写）
hermes profile create writer
hermes profile describe writer --set "专注文档撰写，擅长编写 API 文档、技术博客、README、设计文档"

# 创建 devops profile（运维部署）
hermes profile create devops
hermes profile describe devops --set "专注运维部署，擅长 CI/CD、Docker、Kubernetes、服务器配置"

# 创建 debugger profile（Bug 修复）
hermes profile create debugger
hermes profile describe debugger --set "专注 Bug 修复，擅长定位复杂问题、性能调优、异常处理"
```

## 调度命令

### 单任务调度

```bash
hermes kanban create "<任务标题>" \
  --assignee <profile> \
  --body "<详细描述，包含：
    - 任务目标
    - 技术要求
    - 期望产出
    - 相关文件/模块
  >"
```

### 多任务并行（Swarm）

适用于可并行处理的软件开发流程：

```bash
hermes kanban swarm "<最终目标>" \
  --worker "coder:<开发任务标题>" \
  --worker "tester:<测试任务标题>" \
  --verifier "reviewer" \
  --synthesizer "writer"
```

### 依赖任务链

任务有先后依赖关系时：

```bash
# 先创建父任务
hermes kanban create "调研技术方案" --assignee researcher

# 再创建子任务（依赖父任务）
hermes kanban create "实现功能" --assignee coder --parent <父任务ID>

hermes kanban create "编写测试" --assignee tester --parent <实现任务ID>

hermes kanban create "更新文档" --assignee writer --parent <测试任务ID>
```

## 软件开发典型场景示例

### 场景 1: 新功能开发完整流程

**用户**: "帮我开发一个用户认证模块，包含登录、注册、密码重置"

**Agent 执行**:
```bash
# 创建 Swarm 并行任务图
hermes kanban swarm "完成用户认证模块开发" \
  --worker "coder:实现用户认证核心逻辑（登录/注册/密码重置）" \
  --worker "tester:编写认证模块单元测试" \
  --worker "writer:编写认证模块 API 文档" \
  --verifier "reviewer" \
  --synthesizer "writer"
```

**回复用户**:
```
已将认证模块开发调度到 Kanban 看板：

📋 任务分解：
  ├─ coder → 实现认证核心逻辑
  ├─ tester → 编写单元测试（并行）
  ├─ writer → 编写 API 文档（并行）
  ├─ reviewer → 代码审查（验证）
  └─ writer → 整合文档（合成）

🔄 执行方式：异步并行
📍 进度查看：Agent → 看板
```

### 场景 2: 技术调研 + 方案文档

**用户**: "调研一下 ORM 框架选型，对比 Prisma、Drizzle、TypeORM，输出选型报告"

**Agent 执行**:
```bash
hermes kanban create "ORM 框架选型调研" \
  --assignee researcher \
  --body "调研 Prisma、Drizzle、TypeORM 三个 ORM 框架：
    1. 性能对比（查询效率、批量操作）
    2. 类型安全（TypeScript 支持）
    3. 生态成熟度（社区、文档、插件）
    4. 与现有项目技术栈兼容性
    5. 迁移成本评估
    
    输出：选型报告 + 推荐方案 + 迁移建议"
```

### 场景 3: 复杂 Bug 修复

**用户**: "数据库连接池在高并发下会偶发连接泄漏，帮我定位修复"

**Agent 执行**:
```bash
hermes kanban create "修复数据库连接池泄漏问题" \
  --assignee debugger \
  --body "问题：高并发场景下数据库连接池偶发连接泄漏

    定位步骤：
    1. 分析连接池配置（max/poolSize/idleTimeout）
    2. 检查连接获取/释放代码路径
    3. 添加连接生命周期日志追踪
    4. 复现问题并定位泄漏点
    
    修复要求：
    - 确保所有连接路径都有 release
    - 添加超时自动释放保护
    - 补充连接池监控指标
    
    相关文件：
    - ~/WebstormProjects/supertool/core/db/connection_pool.ts
    - ~/WebstormProjects/supertool/core/db/pool_manager.ts"
```

### 场景 4: 模块重构

**用户**: "把 electron/main.ts 拆分成多个模块，太大了不好维护"

**Agent 执行**:
```bash
# 创建任务链（有依赖）
hermes kanban create "分析 main.ts 模块拆分方案" --assignee reviewer

hermes kanban create "执行 main.ts 模块拆分" \
  --assignee coder \
  --parent <分析任务ID> \
  --body "将 electron/main.ts 拆分为：
    - ipc-handlers.ts（IPC 处理器）
    - window-manager.ts（窗口管理）
    - app-lifecycle.ts（应用生命周期）
    - db-init.ts（数据库初始化）
    
    要求：
    - 保持原有功能不变
    - 每个模块 <200 行
    - 添加模块文档注释"

hermes kanban create "补充拆分后模块测试" \
  --assignee tester \
  --parent <拆分任务ID>
```

### 场景 5: CI/CD 配置

**用户**: "配置 GitHub Actions 自动发布 npm 包"

**Agent 执行**:
```bash
hermes kanban create "配置 GitHub Actions npm 发布流程" \
  --assignee devops \
  --body "配置 .github/workflows/publish.yml：
    1. 触发条件：release tag 或 main 分支 push
    2. 构建：pnpm build + pnpm tauri build
    3. 测试：pnpm test
    4. 发布：npm publish（自动版本号）
    5. 通知：成功/失败通知
    
    要求：
    - 使用 pnpm
    - 支持语义化版本
    - 添加发布日志"
```

## Profile 不存在时的处理

如果目标 profile 不存在，先创建：

```bash
# 检查 profile 是否存在
hermes profile list | grep -q "<profile>" || {
  hermes profile create <profile>
  hermes profile describe <profile> --set "<能力描述>"
}
```

## 任务状态流转

```
todo（待办） → ready（就绪，可被领取） → in_progress（执行中） → done（完成）
                          ↓
                      blocked（阻塞，需人工介入）
                          ↓
                      scheduled（等待定时执行）
```

## Dispatcher 调度

Gateway 内置 Dispatcher 会自动：
1. **reclaim**: 回收超时任务（stale claims）
2. **promote**: 推进就绪任务（依赖已完成）
3. **spawn**: 启动 Worker Profile 执行任务

手动触发调度：
```bash
hermes kanban dispatch
```

## 结果查看

```bash
# 查看任务详情
hermes kanban show <task_id>

# 查看执行历史
hermes kanban runs <task_id>

# 查看事件日志
hermes kanban log <task_id>
```

或通过 SuperTool 界面：
- Agent → 看板 → 查看任务详情

## 与 delegate_task 协同

复杂场景可结合使用：

```
对话 → Kanban 创建任务 → Worker 执行时 → delegate_task 分解子任务
```

例如：
1. Kanban 任务: "重构 authentication 模块"
2. Worker 执行时发现需要：
   - 调研现有代码结构 → delegate_task 给 researcher
   - 拆分重构步骤 → delegate_task 给 planner
   - 执行重构 → 直接处理

## 注意事项

1. **任务描述要详细**：包含目标、技术要求、期望产出、相关文件
2. **正确选择 Profile**：根据任务性质匹配专长
3. **合理设置依赖**：有顺序关系的任务用 `--parent`
4. **定期查看进度**：Dispatcher 异步执行，需主动查看结果
5. **处理阻塞任务**：blocked 任务需人工介入后 `unblock`