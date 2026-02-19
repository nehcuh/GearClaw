# 07 Agent 触发机制

## 1. 触发判断优先级

判定顺序：

1. 黑名单 `disabled_channels`（命中即拒绝）  
2. 白名单 `enabled_channels`（配置后必须命中）  
3. 触发模式 `mode`（always/mention/keyword）

## 2. 触发模式

### 2.1 always

始终触发（通过白名单/黑名单后）。

### 2.2 mention（默认）

消息包含提及模式时触发，常见为 `@agent`、`@bot`。

### 2.3 keyword

消息包含关键词时触发，大小写不敏感。

## 3. 频道键格式

统一使用：

```text
platform:channel_id
```

例如：

```text
discord:123456789012345678
```

## 4. 示例配置

```toml
[agent.triggers]
mode = "mention"
mention_patterns = ["@agent", "@bot"]
keywords = []
enabled_channels = []
disabled_channels = ["discord:111111111111111111"]
```

关键词模式示例：

```toml
[agent.triggers]
mode = "keyword"
keywords = ["bug", "报错", "help"]
```

## 5. 设计建议

1. 生产建议优先 `mention`，避免噪声触发。  
2. 群聊建议配置黑名单（公告频道等）。  
3. 关键词模式建议使用高区分词，避免误触发。

## 6. 导航

- 上一篇：[`06-网关与频道集成.md`](./06-网关与频道集成.md)  
- 下一篇：[`08-Memory记忆系统.md`](./08-Memory记忆系统.md)
