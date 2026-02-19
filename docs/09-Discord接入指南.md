# 09 Discord 接入指南

## 1. 前置条件

1. Discord 账号与可管理的服务器  
2. 已创建 Discord 应用并添加 Bot  
3. 能配置环境变量 `DISCORD_BOT_TOKEN`

## 2. Bot 配置步骤

1. 在 Discord Developer Portal 创建应用  
2. 在 Bot 页面创建/重置 token 并妥善保存  
3. 开启 `MESSAGE CONTENT INTENT`  
4. 通过 OAuth2 URL 邀请 Bot 入服并授予消息相关权限

## 3. 本地配置

```bash
export DISCORD_BOT_TOKEN="your_bot_token"
```

触发相关配置示例：

```toml
[agent.triggers]
mode = "mention"
mention_patterns = ["@agent", "@bot"]
enabled_channels = ["discord:123456789012345678"]
```

## 4. 启动与验证

```bash
cargo run -p gearclaw_cli -- gateway
```

验证点：

1. 日志中看到 Discord 连接与事件  
2. 在目标频道发送提及消息  
3. 收到 Agent 回复

## 5. 故障排查

1. **401 Unauthorized**：token 无效或已过期  
2. **Disallowed intent**：未开启 MESSAGE CONTENT INTENT  
3. **无回复**：频道不在允许范围、触发词不匹配、权限不足  
4. **发送失败**：target 格式不正确，应为 `discord:<channel_id>`

## 6. 安全建议

1. token 仅放环境变量，不写入仓库  
2. 使用最小权限原则  
3. 生产建议配置频道白名单

## 7. 导航

- 上一篇：[`08-Memory记忆系统.md`](./08-Memory记忆系统.md)  
- 下一篇：[`10-macOS自动化工具.md`](./10-macOS自动化工具.md)
