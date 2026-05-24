---
name: dispatch-to-kanban
description: 将软件开发任务调度到 Kanban 看板，分配给专门的 Worker Profile 异步执行
trigger: 用户请求涉及代码开发、测试、重构、文档、部署、调研等软件工程任务
version: 1.1.0
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

## 创建 Worker Profile

```bash
# 1. 创建 profile
hermes profile create <name>

# 2. 从主配置复制必要文件
cp ~/.hermes/config.yaml ~/.hermes/profiles/<name>/config.yaml
cp ~/.hermes/.env ~/.hermes/profiles/<name>/.env

# 3. 编辑 SOUL.md 设置角色定位
# ~/.hermes/profiles/<name>/SOUL.md
```

### 标准 Profile 列表

| Profile | 角色 | 专长 |
|---------|------|------|
| `coder` | 高级软件工程师 | 代码开发、重构、架构设计 |
| `reviewer` | 高级代码审查员 | 代码审计、安全检查、最佳实践 |
| `tester` | 高级测试工程师 | 单测、集成测试、覆盖率分析 |
| `researcher` | 高级技术研究员 | 技术选型、方案调研、可行性分析 |
| `writer` | 高级技术文档工程师 | API 文档、README、架构文档 |
| `devops` | 高级运维工程师 | CI/CD、Docker/K8s、监控告警 |
| `debugger` | 高级调试工程师 | 复杂 Bug 定位、性能调优、根因分析 |

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

## 任务状态流转

```
todo（待办） → ready（就绪，可被领取） → in_progress（执行中） → done（完成）
                          ↓
                      blocked（阻塞，需人工介入）
```

## Dispatcher 调度

Gateway 内置 Dispatcher 每 60s 自动调度：
1. **reclaim**: 回收超时任务（stale claims）
2. **promote**: 推进就绪任务（依赖已完成）
3. **spawn**: 启动 Worker Profile 执行任务

手动触发：
```bash
hermes kanban dispatch
```

## 结果查看

```bash
# 查看所有任务
hermes kanban list

# 查看任务详情
hermes kanban show <task_id>

# 查看执行历史
hermes kanban runs <task_id>

# 查看 Worker 日志
hermes kanban log <task_id>

# 添加评论（给 Worker 提供上下文）
hermes kanban comment <task_id> "你的消息"
```

## 注意事项（踩坑记录）

1. **Worker 必须有独立配置**：新创建的 profile 没有 config.yaml 和 .env，必须手动复制，否则 worker 启动时 401 崩溃
2. **Worker workspace 隔离**：每个 task 有独立 workspace（`~/.hermes/kanban/workspaces/t_<id>/`），依赖链上的文件传递需显式处理：
   - 要么让 parent task 把文件写到 child 能访问的共享路径
   - 要么通过 `kanban comment` 告知文件位置
3. **模型配置继承**：Worker 以 `HERMES_HOME=~/.hermes/profiles/<name>` 启动，从该目录读取 config.yaml。如果不存在则回退到默认配置但可能选错 provider
4. **Dispatcher 在 Gateway 中运行**：每 60s 一次调度循环。如果 gateway 未运行，需要 `hermes kanban dispatch` 手动触发
5. **任务超时**：Worker 默认 max_turns=90，超出后自动失败。长时间任务需调整 agent.max_turns 或拆分成子任务
