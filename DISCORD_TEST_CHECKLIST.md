# Discord Bot 测试检查清单

## ✅ 已完成的部分

1. ✅ **MESSAGE CONTENT INTENT** 已启用
2. ✅ **Discord Gateway 连接成功**
3. ✅ **Bot 在线运行**
4. ✅ **心跳正常**（每20-40秒）

## 🔍 需要检查的部分

### 1. Bot 是否在正确的服务器？

**检查方法：**
```bash
# 查看 Gateway 日志中的 Ready 事件
# 应该显示 Bot 加入的服务器列表
```

### 2. Bot 是否在正确的频道？

**确认：**
- 您必须在 Bot **被添加的服务器**中
- 在有 **Bot 权限的频道**中测试

### 3. 消息格式是否正确？

**正确格式：**
```
@agent hello
@agent 今天天气怎么样
```

**注意：**
- 必须使用 **@mention** 格式
- `@agent` 是默认触发词
- 也可以尝试 `@bot`

### 4. Bot 权限检查

**必需权限：**
- ✅ **Send Messages**（发送消息）
- ✅ **Read Messages/View Channels**（读取消息）
- ✅ **Read Message History**（读取历史）

**检查方法：**
1. 服务器设置 → 角色
2. 找到 Bot 角色
3. 查看权限列表

### 5. Channel ID 日志查看

**启动调试模式：**
```bash
# 停止当前进程
pkill -f 'gearclaw.*gateway'

# 重新启动（带详细日志）
RUST_LOG=gearclaw_channels=debug,gearclaw_cli=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway
```

**查看日志：**
- 看到服务器 ID（guild_id）
- 看到频道列表
- 看到消息事件

## 🧪 测试步骤

### Step 1: 确认 Bot 在服务器中

在 Discord 中：
```
1. 打开您的服务器
2. 在右侧成员列表中查找 Bot
3. 确认 Bot 显示为"在线"（绿色圆点）
```

### Step 2: 确认 Bot 在频道中

```
1. 选择一个文本频道
2. 确认 Bot 有权访问该频道
3. 在频道输入框右侧看到 Bot 图标
```

### Step 3: 发送测试消息

**尝试这些命令：**
```
@agent hello
@agent 你好
@bot hello
```

### Step 4: 查看日志

**实时监控：**
```bash
# 在另一个终端窗口
tail -f /tmp/gateway_final.log
```

**预期看到：**
```
Received Discord event: MessageCreate
Received Discord message from <username>: <message>
Agent response: <response>
Successfully sent response to Discord channel <id>
```

## ❗ 常见问题

### 问题 1: Bot 不响应

**可能原因：**
- Bot 未添加到服务器
- MESSAGE CONTENT INTENT 未启用（✅ 已启用）
- Bot 缺少权限
- 消息格式不正确

**解决方法：**
1. 重新邀请 Bot 到服务器
2. 检查 Bot 权限
3. 使用 @mention 格式

### 问题 2: 收到消息但不响应

**检查：**
```bash
# 查看详细日志
grep -E "Received Discord message|Agent response" /tmp/gateway_final.log
```

### 问题 3: Bot 已读但未回复

**可能原因：**
- LLM 服务未运行（http://127.0.0.1:1234）
- 配置错误

**检查：**
```bash
# 测试 LLM 服务
curl http://127.0.0.1:1234/v1/models
```

## 📞 获取帮助

如果以上都检查过还是有问题，请提供：
1. Discord 服务器中的 Bot 状态截图
2. Gateway 日志（最后 50 行）
3. 发送的测试消息内容

---

**当前 Gateway 状态：**
- 进程 ID: 66814
- 日志文件: /tmp/gateway_final.log
- 运行时间: 约 2 分钟
- 事件数: 5 个（Ready + 心跳）
