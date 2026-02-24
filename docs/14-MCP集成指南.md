# 14 MCP 集成指南

## 1. 概述

GearClaw 实现了完整的 MCP（Model Context Protocol）支持，包含：

1. 真实 Stdio 传输层（子进程通信）
2. JSON-RPC 2.0 协议握手与工具调用
3. 多服务器并行管理
4. 内置注册表（13 个官方服务器）
5. Agent 自主扩展能力（搜索 → 安装 → 启用 → 重载）
6. CLI 管理命令 `gearclaw mcp`

---

## 2. 架构概览

```
gearclaw_mcp (crate)
├── protocol.rs   JSON-RPC 2.0 消息类型（Request/Response/Notification）
├── transport.rs  StdioTransport — 子进程 stdin/stdout 读写
├── client.rs     McpClient — 单服务器连接（initialize 握手 + tool 调用）
├── registry.rs   RegistryEntry + 内置 12 个服务器目录
├── error.rs      McpError 枚举
└── lib.rs        McpManager（多服务器管理）、ServerStatusEntry、re-exports

gearclaw_core
├── config.rs     McpConfig / McpServerConfig（含 enabled 字段）
└── mcp.rs        线程安全包装 Arc<McpManager> + Mutex

Agent (execute_tool_call)
└── 6 个自扩展工具路由（见第 5 节）
```

---

## 3. 配置

在 `~/.gearclaw/config.toml` 的 `[mcp]` 段：

```toml
[mcp.servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
enabled = true

[mcp.servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
enabled = true
env = { GITHUB_PERSONAL_ACCESS_TOKEN = "ghp_xxx" }

[mcp.servers.fetch]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-fetch"]
enabled = false
```

字段说明：

- `command`：启动服务器的可执行程序
- `args`：命令参数列表
- `env`：注入到服务器进程的环境变量
- `enabled`：`true` = 启动时自动连接，`false` = 跳过（默认 `true`）

工具名称约定：调用格式为 `{server_name}__{tool_name}`，例如 `filesystem__list_directory`。

---

## 4. CLI 管理命令

### 4.1 列出服务器状态

```bash
gearclaw mcp list
```

输出每个已配置服务器的名称、命令、连接状态和工具数量。

### 4.2 搜索内置注册表

```bash
# 列出所有可用服务器
gearclaw mcp search

# 按关键词过滤
gearclaw mcp search database
gearclaw mcp search github
```

### 4.3 从注册表安装服务器

```bash
gearclaw mcp install filesystem
gearclaw mcp install github
gearclaw mcp install fetch
```

此命令会：
1. 从注册表查找服务器信息
2. 运行包安装命令（如 `npm install -g @modelcontextprotocol/server-filesystem`）
3. 将服务器配置写入 `config.toml`
4. 立即尝试连接

### 4.4 启用 / 禁用服务器

```bash
gearclaw mcp enable filesystem
gearclaw mcp disable github
```

修改 `config.toml` 中的 `enabled` 字段，并更新当前会话的连接状态。

### 4.5 重载所有连接

```bash
gearclaw mcp reload
```

断开所有已连接的服务器并重新连接（读取当前内存中的配置）。

---

## 5. Agent 自主扩展工具

当 Agent 运行时，它可以使用以下 6 个内置工具自主管理 MCP：

| 工具名 | 说明 |
|--------|------|
| `mcp_list_servers` | 列出所有已配置服务器的连接状态 |
| `mcp_search_registry` | 在内置注册表中搜索可用的 MCP 服务器 |
| `mcp_install_server` | 安装指定 ID 的 MCP 服务器并写入配置 |
| `mcp_enable_server` | 按名称启用服务器（修改配置 + 重连） |
| `mcp_disable_server` | 按名称禁用服务器（修改配置 + 断连） |
| `mcp_reload_servers` | 重载所有服务器连接 |

**示例对话：**

> 用户：帮我安装 filesystem MCP 服务器，并允许它访问我的工作目录
>
> Agent：（调用 `mcp_search_registry` → `mcp_install_server` → 修改 args → `mcp_reload_servers`）

---

## 6. 内置注册表

内置 13 个官方 MCP 服务器：

| ID | 名称 | 安装方式 | 必要环境变量 |
|----|------|----------|-------------|
| `filesystem` | Filesystem | npx | — |
| `github` | GitHub | npx | `GITHUB_PERSONAL_ACCESS_TOKEN` |
| `fetch` | Fetch | npx | — |
| `memory` | Memory (Knowledge Graph) | npx | — |
| `postgres` | PostgreSQL | npx | — |
| `sqlite` | SQLite | uvx | — |
| `brave-search` | Brave Search | npx | `BRAVE_API_KEY` |
| `puppeteer` | Puppeteer | npx | — |
| `slack` | Slack | npx | `SLACK_BOT_TOKEN`, `SLACK_TEAM_ID` |
| `aws-kb-retrieval` | AWS Knowledge Base | npx | `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION` |
| `google-maps` | Google Maps | npx | `GOOGLE_MAPS_API_KEY` |
| `sequential-thinking` | Sequential Thinking | npx | — |
| `context7` | Context7 | npx | — |

---

## 7. 连接生命周期

```
Agent::new()
  └── McpManager::init_clients()
        └── 对每个 enabled=true 的服务器：
              McpClient::connect()
                ├── StdioTransport::spawn() — 启动子进程
                ├── send initialize request
                ├── receive initialize response
                └── send initialized notification
                    → 服务器就绪，status = Connected
```

连接失败时（服务器未安装、命令不存在等）为非致命错误，Agent 仍然正常启动，只是该服务器的工具不可用。

---

## 8. 故障排查

**工具调用失败：`ToolNotFound`**

1. 运行 `gearclaw mcp list` 检查服务器是否 connected
2. 确认 `enabled = true`
3. 确认服务器已安装（`which npx` 或 `npm list -g | grep @modelcontextprotocol`）

**服务器无法启动：Spawn error**

1. 确认 Node.js 已安装（npx 方式）
2. 检查 env 中环境变量是否正确
3. 手动测试：`npx -y @modelcontextprotocol/server-filesystem /tmp`

**工具名冲突**

同名工具来自不同服务器时，第一个注册的优先（取决于 HashMap 迭代顺序）。建议为服务器设置唯一名称。

---

## 9. 导航

- 上一篇：[`13-开发指南.md`](./13-开发指南.md)
- 关联文档：[`05-配置说明.md`](./05-配置说明.md)、[`03-扩展说明.md`](./03-扩展说明.md)
