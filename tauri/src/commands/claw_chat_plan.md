# Claw 模块对齐 CLI 实现 — 改动计划

## P0 改动（功能阻塞）

### 1. 添加 build_runtime_state()
功能: 替代原版 CLI 的 build_runtime_plugin_state()，但只做 GUI 需要的最小版本
- 加载 ConfigLoader → RuntimeConfig → feature_config
- 构建 tool_registry (无 MCP)
- 返回 (feature_config, tool_registry)

### 2. claw_chat_send: new_with_features 替代 new
- 调用 build_runtime_state() 获取 feature_config
- 用 PermissionPolicy::new(mode).with_permission_rules(feature_config.permission_rules())
- ConversationRuntime::new_with_features(..., &feature_config)

### 3. TauriToolExecutor 增强
- 添加 allowed_tools 过滤
- 保留现有 execution (不要求 MCP 支持在这轮搞定)
