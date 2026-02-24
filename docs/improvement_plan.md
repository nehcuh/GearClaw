# GearClaw 架构改进计划 (Improvement Plan)

本计划基于深度评审报告，旨在解决 GearClaw 当前存在的上帝对象、逻辑重复和性能瓶颈问题。

## 1. 核心目标
- **解耦 (Decoupling)**: 将具体工具实现从 `Agent` 结构体中剥离。
- **统一 (Unification)**: 消除 Gateway 和 Agent 之间的逻辑重复。
- **优化 (Optimization)**: 解决 Memory 系统的大规模检索性能问题。
- **类型安全 (Type Safety)**: 引入强类型的工具参数校验。

---

## 2. 阶段性任务

### 第一阶段：工具链重构与动态分发 (核心瘦身)
**目标**: 将 `Agent` 的 `execute_tool_call` 从 500 行的 `match` 语句重构为基于注册表的分发模式。

1.  **定义 `Tool` Trait**:
    - 在 `gearclaw_core::tools` 中定义 `AsyncTool` trait。
    - 方法包括 `name()`, `description()`, `parameters()`, `execute(args: Value, context: ToolContext)`.
2.  **迁移原生工具**:
    - 将 `read_file`, `write_file`, `exec`, `list_files` 等迁移到 `crates/tools`。
3.  **迁移 MCP 管理工具**:
    - 将 `mcp_install_server`, `mcp_list_servers` 等迁移到 `crates/mcp/src/tools.rs`。
4.  **实现 `ToolRegistry`**:
    - `Agent` 持有一个 `ToolRegistry` 实例。
    - 启动时自动注册核心工具、技能工具和 MCP 工具。

### 第二阶段：消息处理链路统一 (消除重复)
**目标**: 让 Gateway 和 CLI 共享完全相同的 `Agent` 消息处理逻辑。

1.  **增强 `Agent::process_channel_message`**:
    - 确保该 API 能够处理所有必要的上下文（Session, Platform, Source）。
2.  **重构 `GatewayServer`**:
    - 移除 `process_agent_response` 中的手动会话管理。
    - 直接调用 `agent.process_channel_message`。
3.  **标准化 Session ID**:
    - 统一格式为 `platform:source_type:source_id`（例如 `discord:channel:12345`）。

### 第三阶段：Memory 系统升级 (性能优化)
**目标**: 解决搜索时全量加载向量的问题，实现 O(log N) 级别的检索。

1.  **引入 `sqlite-vss` 或 `faiss-rs`**:
    - 在 SQLite 中使用向量搜索插件，或者集成轻量级向量库。
2.  **重写 `MemoryManager::search`**:
    - 改为调用数据库底层的向量检索指令。
3.  **分块优化**:
    - 改进 Markdown 分块策略，支持带重叠的滑动窗口（Sliding Window）。

### 第四阶段：强类型与安全性 (鲁棒性)
**目标**: 提高工具调用和配置的安全性。

1.  **工具参数宏/结构体**:
    - 使用 `serde` 直接将 `tool_calls` 的 JSON 参数反序列化为具体的 `Args` 结构体。
2.  **改进安全策略**:
    - 细化 `allowlist` 规则，支持参数级的正则校验。

---

## 3. 优先级与时间表

| 任务 | 优先级 | 预计难度 |
| :--- | :--- | :--- |
| **第一阶段: 工具重构** | P0 (紧急) | 中 |
| **第二阶段: 逻辑统一** | P1 (重要) | 低 |
| **第三阶段: 搜索优化** | P1 (重要) | 高 |
| **第四阶段: 强类型支持** | P2 (优化) | 中 |

---

## 4. 验证标准
- **回归测试**: CLI 和 Discord 机器人功能必须保持正常。
- **性能基准**: 在拥有 1000 条记忆记录时，检索延迟应低于 50ms。
- **可读性**: `agent.rs` 的行数应减少 50% 以上。
