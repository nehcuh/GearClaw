# 🦞 GearClaw - OpenClaw Rust Prototype

GearClaw 是一个用 Rust 实现的 OpenClaw AI 助手原型框架。它提供了完整的 LLM 交互、工具执行、会话管理和 CLI 命令行界面。

## ✨ 特性

- 🧠 **LLM 集成** - 支持 OpenAI 兼容 API（可配置不同提供商）
- 🔧 **工具系统** - 安全执行 shell 命令、文件读写、Git 操作、Docker 管理
- 📝 **会话管理** - 持久化会话历史，支持多会话
- 🛡️ **安全控制** - 可配置的安全级别（deny/allowlist/full）
- 🚀 **异步高性能** - 基于 Tokio 的异步架构
- 🎯 **CLI 界面** - 交互式聊天、单条命令执行、配置管理

## 📋 功能

### LLM 功能
- 支持多模型配置（主模型 + 备用模型）
- 工具调用（Function Calling）支持
- 流式响应（预留接口）
- 上下文窗口管理

### 工具功能
- `exec` - 执行 shell 命令
- `read_file` - 读取文件内容
- `write_file` - 写入文件
- `git_status` - 查看 Git 状态
- `docker_ps` - 列出容器
- `web_search` - 网页搜索（通过 DuckDuckGo API）

### 会话功能
- 多会话管理
- 会话持久化（JSON 格式）
- 对话历史保存和恢复
- 清除历史功能

## 🚀 快速开始

### 安装

```bash
# 克隆或下载项目
cd gearclaw

# 构建（开发模式）
cargo build

# 运行
cargo run
```

### 配置

创建配置文件 `~/.openclaw/gearclaw.toml`：

```toml
[llm]
primary = "zai/glm-4.7"
fallbacks = ["openai/gpt-4"]
endpoint = "https://api.openai.com/v1"
api_key = "your-api-key-here"

[tools]
security = "full"          # deny | allowlist | full
host = "gateway"              # gateway | sandbox | node
elevated_enabled = true
profile = "full"              # minimal | coding | messaging | full

[session]
session_dir = "/path/to/sessions"
save_interval = 60
max_tokens = 200000

[agent]
name = "GearClaw"
system_prompt = "你是一个智能 AI 助手..."
workspace = "/path/to/workspace"
memory_enabled = true
```

生成示例配置：
```bash
cargo run -- config-sample
```

### 使用

#### 交互式聊天
```bash
cargo run
# 或
cargo run -- chat
```

#### 单条命令
```bash
cargo run -- run "帮我写一个 Hello World 程序"
```

#### 管理会话
```bash
# 列出所有会话
cargo run -- list-sessions

# 删除会话
cargo run -- delete-session session-id
```

#### 交互式命令
```
> help          # 显示帮助
> clear         # 清除对话历史
> exit/quit     # 退出
> 列出当前目录的文件
> 查看 git 状态
> 写一个 Rust 函数
```

## 🏗️ 项目结构

```
gearclaw/
├── src/
│   ├── main.rs          # 入口，CLI 解析和命令分发
│   ├── cli.rs          # CLI 定义
│   ├── cli/
│   │   └── mod.rs     # CLI 模块
│   ├── config.rs       # 配置管理
│   ├── error.rs        # 错误类型定义
│   ├── agent.rs        # 主 Agent 逻辑
│   ├── agent/
│   │   └── mod.rs     # Agent 模块
│   ├── llm.rs          # LLM API 客户端
│   ├── tools.rs        # 工具执行器
│   ├── session.rs       # 会话管理
│   │   └── mod.rs     # Session 模块
├── Cargo.toml          # 依赖配置
└── README.md           # 本文件
```

## 🔧 依赖

- `tokio` - 异步运行时
- `serde` - 序列化/反序列化
- `serde_yml` - YAML 配置支持
- `reqwest` - HTTP 客户端
- `clap` - CLI 解析
- `tracing` - 日志和追踪
- `uuid` - 唯一 ID 生成
- `chrono` - 时间处理

## 🔒 安全特性

- 三级安全模式：`deny`（禁止）、`allowlist`（白名单）、`full`（完全开放）
- 白名单模式仅允许安全命令（ls, cat, git, docker 等）
- 工具执行错误捕获和报告
- 配置验证和错误处理

## 📝 开发

```bash
# 构建
cargo build

# 运行测试（需要先实现）
cargo test

# 检查
cargo clippy

# 格式化
cargo fmt
```

## 🎯 路线图

- [x] 基础项目结构
- [x] 配置系统
- [x] 错误处理
- [x] LLM API 集成
- [x] 工具执行系统
- [x] 会话管理
- [x] CLI 界面
- [ ] 更多工具集成（文件系统、数据库等）
- [ ] 流式响应
- [ ] 记忆/向量搜索
- [ ] WebSocket 支持
- [ ] Web UI

## 🤝 贡献

GearClaw 是一个学习项目，欢迎提交 Issue 和 Pull Request。

## 📄 许可证

MIT

## 🦞 为什么要叫 GearClaw？

"Gear" 代表 Rust 的齿轮/工具特性，"Claw" 致敬 OpenClaw 的龙虾标识。组合起来就是 "GearClaw" - 一个坚固、高效的 Rust AI 助手！

---

Made with 🦞 and Rust
