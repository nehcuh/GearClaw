# Discord 配置指南

GearClaw 已完整集成 Discord 支持！本指南将帮助你完成配置和部署。

## 📋 前置要求

1. **Discord 账号** - 需要有一个 Discord 账号
2. **Discord 应用** - 需要创建一个 Discord 应用并获取 Bot Token
3. **服务器权限** - 需要有管理 Discord 服务器的权限

## 🚀 快速开始

### 步骤 1: 创建 Discord 应用和 Bot

1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 点击 **"New Application"** 按钮
3. 输入应用名称（例如：GearClaw Bot）
4. 点击 **"Create"**

### 步骤 2: 创建 Bot 用户

1. 在左侧导航栏点击 **"Bot"**
2. 点击 **"Reset Token"** 或 **"Add Bot"**
3. **重要**: 复制 Bot Token（只显示一次！）
   ```bash
   # 格式类似: MTAwNjMw... (很长的字符串)
   ```
4. **保存好 Token** - 后续配置需要用到

### 步骤 3: 配置 Bot 权限

1. 在 Bot 设置页面，关闭 **"Public Bot"**（仅自己使用）
2. 开启以下权限：
   - ✅ **MESSAGE CONTENT INTENT** (必需) - 读取消息内容
   - ✅ **SERVER MEMBERS INTENT** (可选) - 访问成员列表
   - ✅ **PRESENCE INTENT** (可选) - 访问在线状态

### 步骤 4: 邀请 Bot 到服务器

1. 在左侧导航栏点击 **"OAuth2"** → **"URL Generator"**
2. 勾选以下权限：
   - `bot`
   - `applications.commands`
3. 在 Bot 权限中勾选：
   - `Send Messages`
   - `Read Messages/View Channels`
   - `Read Message History`
   - `Add Reactions`
4. 复制生成的 URL
5. 在浏览器中打开 URL，选择服务器并授权

### 步骤 5: 配置 GearClaw

编辑 `~/.gearclaw/config.toml`:

```toml
# Discord 配置
[agent]
# 启用频道白名单（可选）
enabled_channels = [
    "discord:123456789012345678",  # 允许的频道 ID
]

# 启用频道黑名单（可选）
disabled_channels = [
    "discord:987654321098765432",  # 禁止的频道 ID
]

# 触发模式
trigger_mode = "mention"  # 可选: "mention", "keyword", "auto"

# 提及模式下的触发词
mention_patterns = ["@agent", "@bot", "@gearclaw"]
```

设置环境变量（推荐）：

```bash
# 方法 1: 在 ~/.zshrc 或 ~/.bashrc 中添加
export DISCORD_BOT_TOKEN="你的_BOT_TOKEN_在这里"

# 方法 2: 直接在终端设置（临时）
export DISCORD_BOT_TOKEN="你的_BOT_TOKEN_在这里"
```

或者在 config.toml 中设置（不推荐，不安全）：

```toml
# ⚠️ 不推荐：敏感信息放在环境变量中更安全
[discord]
bot_token = "你的_BOT_TOKEN_在这里"
```

### 步骤 6: 启动 GearClaw Gateway

```bash
# 启动 Gateway 服务
cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway

# 或者使用配置文件
cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway --config ~/.gearclaw/config.toml
```

## 📝 获取频道 ID

### 方法 1: 启用开发者模式

1. Discord 设置 → **Advanced** → 开启 **Developer Mode**
2. 右键点击频道 → **Copy ID** → 获取频道 ID（如：123456789012345678）

### 方法 2: 使用命令

在 Discord 中输入：
```
@GearClaw channel_id
```

## 🎯 使用方式

### 方式 1: 提及触发（默认）

在 Discord 频道中提及 Bot：

```
@agent 帮我搜索 Rust 编程教程
@bot 今天天气怎么样
@gearclaw 解释一下闭包是什么
```

### 方式 2: 关键词触发

配置 `trigger_mode = "keyword"` 并设置关键词：

```toml
[agent]
trigger_mode = "keyword"
keywords = ["帮忙", "搜索", "解释"]
```

然后直接发送消息：

```
帮忙搜索 Rust 教程
搜索今天的新闻
解释闭包概念
```

### 方式 3: 自动回复

配置 `trigger_mode = "auto"`，Bot 会回复所有消息（谨慎使用！）

## ⚙️ 配置选项

### 频道白名单

只在指定频道中响应：

```toml
[agent]
enabled_channels = [
    "discord:123456789012345678",  # 通用频道
    "discord:987654321098765432",  # 私有频道
]
```

### 频道黑名单

在所有频道响应，除了指定频道：

```toml
[agent]
disabled_channels = [
    "discord:111111111111111111",  # 不响应此频道
]
```

### 自定义触发词

```toml
[agent]
mention_patterns = ["@gearclaw", "@助手", "@ai"]
```

## 🔧 高级配置

### 消息分块

Discord 消息限制为 2000 字符，GearClaw 会自动分块：

```toml
[discord]
message_limit = 2000  # 最大消息长度
```

### 嵌入式内容

Bot 可以发送富文本嵌入（未来支持）：

```toml
[discord]
enable_embeds = true  # 启用嵌入消息
```

### 前缀命令（未来）

计划支持类似命令的前缀：

```
!help
!search Rust
!status
```

## 🧪 测试连接

### 1. 检查 Bot 是否在线

在 Discord 服务器右侧列表中应该能看到 Bot 用户。

### 2. 发送测试消息

在配置好的频道中：

```
@agent 你好
```

Bot 应该会回复！

### 3. 测试工具调用

```
@agent 帮我在浏览器中搜索 Rust 编程
```

Bot 会：
1. 打开浏览器
2. 搜索 "Rust 编程"
3. 返回结果

## 🐛 故障排查

### 问题 1: Bot 不回复

**检查**：
1. ✅ Bot Token 是否正确设置？
   ```bash
   echo $DISCORD_BOT_TOKEN
   ```

2. ✅ Bot 是否在服务器中？
   - 检查服务器成员列表

3. ✅ 权限是否正确？
   - MESSAGE CONTENT INTENT 是否开启？
   - Bot 是否有 "Read Messages" 和 "Send Messages" 权限？

4. ✅ 频道是否在白名单中？
   - 检查 `enabled_channels` 配置

**解决**：
```bash
# 查看日志
RUST_LOG=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway
```

### 问题 2: 权限错误

**错误信息**：
```
Error: 403 Forbidden
```

**解决**：
1. 重新邀请 Bot 并勾选所有必需权限
2. 确保频道设置允许 Bot 发送消息

### 问题 3: MESSAGE CONTENT INTENT

**错误信息**：
```
Disallowed intent: MESSAGE_CONTENT is required
```

**解决**：
1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 选择你的应用 → Bot → **Privileged Gateway Intents**
3. 开启 **MESSAGE CONTENT INTENT**
4. 保存并重启 Bot

### 问题 4: Bot Token 无效

**错误信息**：
```
Error: 401 Unauthorized
```

**解决**：
1. Token 是否过期或被重置？
2. 重新生成 Token 并更新环境变量
3. 重启 Gateway 服务

## 📊 监控和日志

### 查看实时日志

```bash
# Debug 日志
RUST_LOG=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway

# 只显示 Discord 相关日志
RUST_LOG=gearclaw_channels=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway
```

### 常见日志级别

- `TRACE` - 最详细
- `DEBUG` - 调试信息
- `INFO` - 一般信息（默认）
- `WARN` - 警告
- `ERROR` - 错误

## 🔐 安全最佳实践

### ✅ 推荐做法

1. **使用环境变量**
   ```bash
   export DISCORD_BOT_TOKEN="token"
   ```

2. **限制 Bot 权限**
   - 只开启必需的 intents
   - 使用频道白名单

3. **定期轮换 Token**
   - 每几个月重置一次

4. **不要提交 Token 到 Git**
   ```gitignore
   # .gitignore
   .env
   config.toml
   ```

### ❌ 避免做法

1. ❌ 将 Token 写在代码中
2. ❌ 在公开频道分享 Token
3. ❌ 使用 "Public Bot" 模式（除非需要）
4. ❌ 开启不必要的权限

## 🎨 自定义 Bot 外观

### 更改 Bot 头像和名称

1. 访问 [Discord Developer Portal](https://discord.com/developers/applications)
2. 选择你的应用
3. 在 **General Information** 页面：
   - 上传头像图片
   - 修改 Bot 显示名称
4. 保存更改，Bot 会自动更新

## 📚 API 参考

### DiscordAdapter

```rust
use gearclaw_channels::DiscordAdapter;

// 从环境变量创建
let adapter = DiscordAdapter::from_env()?;

// 或手动配置
let adapter = DiscordAdapter::new(DiscordConfig {
    bot_token: "your_token".to_string(),
    message_limit: 2000,
});

// 启动适配器
adapter.start().await?;

// 发送消息
use gearclaw_channels::{MessageTarget, MessageContent};

adapter.send_message(
    MessageTarget::Channel("1234567890".to_string()),
    MessageContent {
        text: Some("Hello from GearClaw!".to_string()),
        ..Default::default()
    }
).await?;
```

## 🔗 相关链接

- [Discord Developer Portal](https://discord.com/developers/applications)
- [Discord Bot Documentation](https://discord.com/developers/docs/intro)
- [Twilight Library Docs](https://twilight.rs/index.html)
- [Discord API Documentation](https://discord.com/developers/docs/topics/oauth2)

## 💡 使用示例

### 示例 1: 基本对话

```
User: @agent 解释什么是闭包
Bot: 闭包（Closure）是 Rust 中的一个重要概念...
```

### 示例 2: 工具调用

```
User: @agent 帮我打开 Safari 并搜索 Rust
Bot:
✓ 已启动应用: Safari
✓ 已在浏览器中搜索: Rust
```

### 示例 3: Memory 系统

```
User: @agent 我的配置文件在哪里？
Bot: (自动搜索记忆) 配置文件位于 ~/.gearclaw/config.toml
```

### 示例 4: macOS 自动化

```
User: @agent 发送通知说"任务完成"
Bot: ✓ 已发送通知: 任务完成
```

## 🎉 下一步

现在你可以：

1. ✅ 部署 Discord Bot
2. ✅ 在频道中与 GearClaw 对话
3. ✅ 使用所有工具（浏览器搜索、macOS 控制、文件操作等）
4. ✅ 集成 Memory 系统自动检索上下文

享受与 AI Agent 在 Discord 中的互动吧！🚀
