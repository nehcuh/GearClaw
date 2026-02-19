# GearClaw

GearClaw 是一个基于 Rust 的多 crate AI Agent 工作区，包含 CLI、Gateway、Channels、Memory、Skills 与 GUI（可选）能力。

## 1. 文档入口

- **统一文档索引**：[`docs/00-文档索引.md`](docs/00-文档索引.md)
- 架构说明：[`docs/01-项目架构说明.md`](docs/01-项目架构说明.md)
- 设计说明：[`docs/02-系统设计说明.md`](docs/02-系统设计说明.md)
- 扩展说明：[`docs/03-扩展说明.md`](docs/03-扩展说明.md)

### 1.1 按角色快捷入口

- 架构/设计负责人：[`docs/01-项目架构说明.md`](docs/01-项目架构说明.md)、[`docs/02-系统设计说明.md`](docs/02-系统设计说明.md)
- 功能开发者：[`docs/03-扩展说明.md`](docs/03-扩展说明.md)、[`docs/13-开发指南.md`](docs/13-开发指南.md)
- 接入/运维：[`docs/06-网关与频道集成.md`](docs/06-网关与频道集成.md)、[`docs/09-Discord接入指南.md`](docs/09-Discord接入指南.md)
- 排障人员：[`docs/12-测试与验证.md`](docs/12-测试与验证.md)、[`docs/08-Memory记忆系统.md`](docs/08-Memory记忆系统.md)

### 1.2 按任务快捷入口

- 首次启动：[`docs/04-快速开始与运行.md`](docs/04-快速开始与运行.md)
- 配置参数：[`docs/05-配置说明.md`](docs/05-配置说明.md)
- 调整触发策略：[`docs/07-Agent触发机制.md`](docs/07-Agent触发机制.md)
- macOS 自动化：[`docs/10-macOS自动化工具.md`](docs/10-macOS自动化工具.md)

## 2. 快速启动

```bash
# 1) 构建工作区
cargo build

# 2) 初始化配置（首次）
cargo run -p gearclaw_cli -- init

# 3) 交互模式
cargo run -p gearclaw_cli
```

## 3. 常用命令

```bash
# 单次执行
cargo run -p gearclaw_cli -- run "解释当前仓库结构"

# 会话管理
cargo run -p gearclaw_cli -- list-sessions
cargo run -p gearclaw_cli -- delete-session <session-id>

# 记忆系统
cargo run -p gearclaw_cli -- memory sync
cargo run -p gearclaw_cli -- memory search "query"

# Gateway
cargo run -p gearclaw_cli -- gateway
```

## 4. 当前状态说明

1. **MCP 能力默认禁用**（构建能力为 disabled）。
2. 工作区默认成员为 CLI/核心子系统/Gateway/Channels；GUI crate 在 workspace 中暂时注释，需要单独按需构建。
3. 文档已统一到 `docs/` 目录（根目录仅保留本 README）。

## 5. 许可证

MIT
