# Agent Trigger Configuration

本文档说明如何配置 Agent 的自动响应触发机制。

## 功能概述

Agent 现在支持灵活的触发配置，可以根据以下条件决定是否响应消息：

- **触发模式**: Always（总是响应）、Mention（仅提及）、Keyword（关键词）
- **提及模式**: 自定义提及前缀（如 `@agent`、`@bot`）
- **关键词触发**: 响应包含特定关键词的消息
- **频道过滤**: 白名单/黑名单控制响应的频道

## 触发模式

### 1. Always 模式

Agent 响应所有消息（除非在黑名单中）。

```rust
use gearclaw_core::{AgentTriggerConfig, TriggerMode};

let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Always,
    ..Default::default()
};
```

**使用场景**:
- 个人助手（直接消息）
- 测试环境
- 受控频道

### 2. Mention 模式（默认）

仅在消息包含特定提及模式时响应。

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,
    mention_patterns: vec!["@agent".to_string(), "@bot".to_string()],
    ..Default::default()
};
```

**示例**:
- `@agent 帮我写一段代码` → ✅ 触发
- `@bot 今天天气怎么样` → ✅ 触发
- `有人能帮忙吗？` → ❌ 不触发

### 3. Keyword 模式

仅在消息包含特定关键词时响应。

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Keyword,
    keywords: vec!["错误".to_string(), "bug".to_string(), "报错".to_string()],
    ..Default::default()
};
```

**示例**:
- `程序报错了` → ✅ 触发
- `发现一个 bug` → ✅ 触发
- `今天天气不错` → ❌ 不触发

## 频道过滤

### 白名单（enabled_channels）

仅在指定频道中响应。

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,
    enabled_channels: vec![
        "discord:123456789012345678".to_string(),  // 仅这个 Discord 频道
        "telegram:987654321".to_string(),          // 仅这个 Telegram 群组
    ],
    ..Default::default()
};
```

### 黑名单（disabled_channels）

在指定频道中禁用响应。

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,
    disabled_channels: vec![
        "discord:987654321098765432".to_string(),  // 忽略这个频道
    ],
    ..Default::default()
};
```

**优先级**: 黑名单 > 白名单 > 默认行为

## 配置文件

在 `~/.gearclaw/config.toml` 中配置:

```toml
[agent.triggers]
mode = "mention"  # "always", "mention", "keyword"

mention_patterns = ["@agent", "@bot", "!ai"]
keywords = ["错误", "bug", "help", "报错"]

enabled_channels = [
    "discord:123456789012345678",
    "telegram:987654321"
]

disabled_channels = [
    "discord:987654321098765432"
]
```

## 完整示例

```rust
use gearclaw_gateway::GatewayServer;
use gearclaw_channels::DiscordAdapter;
use gearclaw_core::{Agent, Config, AgentTriggerConfig, TriggerMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建服务器
    let server = GatewayServer::new(GatewayConfig::default());

    // 初始化 Agent
    if let Ok(config) = Config::load(&None) {
        if let Ok(agent) = Agent::new(config).await {
            let handlers = server.handlers();

            // 设置 Agent
            handlers.set_agent(Arc::new(agent)).await;

            // 配置触发器
            let trigger_config = AgentTriggerConfig {
                mode: TriggerMode::Mention,
                mention_patterns: vec!["@agent".to_string()],
                keywords: vec![],
                enabled_channels: vec!["discord:123456789".to_string()],
                disabled_channels: vec![],
            };
            handlers.set_trigger_config(trigger_config).await;

            println!("Agent 已配置为提及模式 (@agent)");
            println!("仅在频道 discord:123456789 中响应");
        }
    }

    // 启动 Discord 适配器
    if let Ok(mut discord) = DiscordAdapter::from_env() {
        discord.start().await?;
        server.register_channel(discord).await?;
    }

    server.start().await?;
}
```

## 工作流程

```
Discord 消息到达
    ↓
检查黑名单 → 如果在列表中，跳过
    ↓
检查白名单 → 如果配置了且不在列表中，跳过
    ↓
检查触发模式:
    ├─ Always → 触发
    ├─ Mention → 检查是否包含提及模式
    └─ Keyword → 检查是否包含关键词
    ↓
如果触发:
    ├─ 广播到所有 WebSocket 客户端
    └─ Agent 处理消息
        └─ 发送响应回 Discord
```

## 最佳实践

### 1. 生产环境推荐配置

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,  // 避免过度响应
    mention_patterns: vec!["@agent".to_string()],
    enabled_channels: vec![],     // 空白名单 = 所有频道
    disabled_channels: vec![
        "discord:announcements_channel".to_string(),  // 避免在公告频道响应
    ],
};
```

### 2. 开发/测试环境

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Always,  // 快速测试
    enabled_channels: vec!["discord:test_channel".to_string()],  // 仅测试频道
    ..Default::default()
};
```

### 3. 专用支持机器人

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Keyword,  // 仅响应问题
    keywords: vec![
        "错误".to_string(),
        "bug".to_string(),
        "帮助".to_string(),
        "help".to_string(),
    ],
    ..Default::default()
};
```

## 高级用法

### 动态更新触发配置

```rust
// 运行时更新配置
let handlers = server.handlers();

// 从 Always 切换到 Mention
let new_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,
    mention_patterns: vec!["@agent".to_string()],
    ..Default::default()
};
handlers.set_trigger_config(new_config).await;
```

### 平台特定配置

```rust
let trigger_config = AgentTriggerConfig {
    mode: TriggerMode::Mention,
    mention_patterns: vec![
        "@agent".to_string(),      // Discord
        "/agent".to_string(),       // Telegram bot 命令风格
    ],
    enabled_channels: vec![
        "discord:123456".to_string(),
        "telegram:789012".to_string(),
    ],
    ..Default::default()
};
```

## 故障排查

### Agent 不响应消息

**检查**:
1. 触发模式是否正确？
2. 提及模式/关键词是否匹配？
3. 频道是否在黑名单中？
4. 频道是否在白名单中（如果配置了白名单）？

**调试**:
```rust
use gearclaw_gateway::triggers::should_trigger_agent;

let would_trigger = should_trigger_agent(
    "discord",
    &source,
    "message content",
    &trigger_config,
);

println!("Would trigger: {}", would_trigger);
```

### Agent 响应太频繁

**解决方案**:
1. 切换从 `Always` 到 `Mention` 模式
2. 配置白名单限制响应频道
3. 使用更具体的提及模式或关键词

## API 参考

### `AgentTriggerConfig`

```rust
pub struct AgentTriggerConfig {
    pub mode: TriggerMode,              // 触发模式
    pub mention_patterns: Vec<String>,   // 提及模式列表
    pub keywords: Vec<String>,           // 关键词列表
    pub enabled_channels: Vec<String>,   // 白名单
    pub disabled_channels: Vec<String>,  // 黑名单
}
```

### `TriggerMode`

```rust
pub enum TriggerMode {
    Always,   // 响应所有消息
    Mention,  // 仅响应提及
    Keyword,  // 仅响应关键词
}
```

### 方法

```rust
// 设置触发配置
handlers.set_trigger_config(config).await;

// 获取当前配置
let config = handlers.get_trigger_config().await;

// 检查消息是否触发
use gearclaw_gateway::triggers::should_trigger_agent;
let triggered = should_trigger_agent(platform, source, content, &config);

// 提取提及前缀
use gearclaw_gateway::triggers::extract_mention_prefix;
let content_without_mention = extract_mention_prefix("@agent hello", &config);
// => Some("hello")
```

## 相关文档

- [CHANNELS.md](./CHANNELS.md) - 频道集成指南
- [examples/with_agent.rs](./examples/with_agent.rs) - 完整示例代码
