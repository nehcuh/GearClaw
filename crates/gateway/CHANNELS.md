# Gateway + Channels Integration

本页面说明如何使用 Gateway 服务器和频道适配器的集成功能。

## 功能概述

Gateway 服务器现在支持：

- ✅ **发送消息** - 通过 WebSocket API 向多个消息平台发送消息
- ✅ **接收消息** - 实时接收来自频道的消息并广播到 WebSocket 客户端
- ✅ **事件流** - 支持频道消息、presence、tick 等事件的实时推送

### 支持的平台

- ✅ **Discord** - 已实现，使用 `twilight-rs` 库
- 🚧 **Telegram** - 计划中，将使用 `teloxide` 库
- 🚧 **WhatsApp** - 计划中，将通过 Node.js Baileys 桥接

## 快速开始

### 1. 设置环境变量

```bash
export DISCORD_BOT_TOKEN="your_discord_bot_token_here"
```

### 2. 启动 Gateway 服务器

```rust
use gearclaw_gateway::GatewayServer;
use gearclaw_channels::{DiscordAdapter, ChannelAdapter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建服务器
    let server = GatewayServer::new(gearclaw_gateway::GatewayConfig::default());

    // 初始化并启动 Discord 适配器
    if let Ok(mut discord) = DiscordAdapter::from_env() {
        discord.start().await?;
        server.register_channel(discord).await?;
    }

    // 启动服务器
    server.start().await?;

    Ok(())
}
```

运行示例：
```bash
cargo run --example with_discord
```

### 3. 连接 WebSocket 并监听事件

连接到 `ws://127.0.0.1:18789/ws`，服务器将自动推送：

```json
{
  "type": "res",
  "id": "hello",
  "ok": true,
  "payload": {
    "protocol": { "min": 1, "max": 1 },
    "presence": [],
    "health": { "status": "ok" },
    "state_version": { "presence": 0, "health": 0 },
    "uptime_ms": 0,
    "policy": {
      "max_payload": 1048576,
      "max_buffered_bytes": 10485760,
      "tick_interval_ms": 30000
    }
  }
}
```

### 4. 接收频道消息

当有人在 Discord 发送消息时，所有连接的客户端将收到：

```json
{
  "type": "event",
  "event": "channel.message",
  "payload": {
    "platform": "discord",
    "source": {
      "type": "user",
      "id": "123456789012345678",
      "name": "Username"
    },
    "content": "Hello, world!",
    "metadata": {
      "channel_id": "987654321098765432",
      "guild_id": "123456789012345678",
      "message_id": "111222333444555666"
    },
    "ts": 1736992800
  }
}
```

### 5. 发送消息到频道

发送 JSON 到 WebSocket：

```json
{
  "type": "req",
  "id": "msg-1",
  "method": "send",
  "params": {
    "target": "discord:123456789012345678",
    "message": "Hello, Discord!"
  }
}
```

响应：

```json
{
  "type": "res",
  "id": "msg-1",
  "ok": true,
  "payload": {
    "success": true,
    "platform": "discord",
    "identifier": "123456789012345678",
    "sent_at": "2025-01-15T10:30:00+00:00"
  }
}
```

## 事件类型

### channel.message

频道消息事件 - 当接收到来自 Discord、Telegram、WhatsApp 的消息时触发。

**字段：**
- `platform` (string): 平台名称（discord, telegram, whatsapp）
- `source` (object): 消息来源信息
  - `type` (string): 来源类型（user, channel, group）
  - `id` (string): 来源 ID
  - `name` (string): 来源名称
- `content` (string): 消息内容
- `metadata` (object, optional): 额外元数据
- `ts` (number): Unix 时间戳

**示例：**

来自用户：
```json
{
  "type": "event",
  "event": "channel.message",
  "payload": {
    "platform": "discord",
    "source": {
      "type": "user",
      "id": "123456789012345678",
      "name": "Alice"
    },
    "content": "Hello!",
    "ts": 1736992800
  }
}
```

来自频道：
```json
{
  "type": "event",
  "event": "channel.message",
  "payload": {
    "platform": "discord",
    "source": {
      "type": "channel",
      "id": "987654321098765432",
      "name": "general"
    },
    "content": "Announcement: ...",
    "ts": 1736992800
  }
}
```

### tick

心跳事件 - 每 30 秒发送一次，用于保持连接活跃。

```json
{
  "type": "event",
  "event": "tick",
  "payload": {}
}
```

## 架构说明

```
┌─────────────┐      WebSocket      ┌──────────────┐
│   Client    │ ◄──────────────────► │   Gateway    │
└─────────────┘                      └──────────────┘
     │  ▲                                     │
     │  │ Events                              │
     │  └─────────────────────────────────────┘
     │              (broadcast)
     │
┌────▼─────┐
│  Discord │
│  Gateway │  (Discord message events)
└──────────┘
```

**数据流：**

1. Discord 用户发送消息
2. Discord Gateway 接收 MESSAGE_CREATE 事件
3. DiscordAdapter 转换为 `IncomingMessage`
4. Gateway 广播为 `GatewayEvent::ChannelMessage`
5. 所有 WebSocket 客户端接收事件

## 完整示例

### Python 客户端

```python
import asyncio
import json
import websockets
from datetime import datetime

async def handle_gateway():
    uri = "ws://127.0.0.1:18789/ws"

    async with websockets.connect(uri) as ws:
        # Receive hello-ok
        hello = await ws.recv()
        print(f"Connected: {hello}")

        # Task to send messages
        async def sender():
            await asyncio.sleep(2)

            # Send a message to Discord
            request = {
                "type": "req",
                "id": "send-1",
                "method": "send",
                "params": {
                    "target": "discord:123456789012345678",
                    "message": "Hello from Gateway!"
                }
            }
            await ws.send(json.dumps(request))
            print("Sent message request")

        # Task to receive events
        async def receiver():
            while True:
                try:
                    message = await ws.recv()
                    data = json.loads(message)

                    if data.get("type") == "event":
                        event = data.get("event")
                        payload = data.get("payload", {})

                        if event == "channel.message":
                            print(f"\n[Channel Message]")
                            print(f"  Platform: {payload.get('platform')}")
                            print(f"  Source: {payload.get('source')}")
                            print(f"  Content: {payload.get('content')}")
                            print(f"  Time: {datetime.fromtimestamp(payload.get('ts', 0))}")
                        elif event == "tick":
                            print(".", end="", flush=True)

                    elif data.get("type") == "res":
                        print(f"\nResponse: {json.dumps(data, indent=2)}")

                except websockets.exceptions.ConnectionClosed:
                    print("\nConnection closed")
                    break

        # Run both tasks
        await asyncio.gather(sender(), receiver())

asyncio.run(handle_gateway())
```

### JavaScript/Node.js 客户端

```javascript
const WebSocket = require('ws');

const ws = new WebSocket('ws://127.0.0.1:18789/ws');

ws.on('open', () => {
  console.log('Connected to Gateway');
});

ws.on('message', (data) => {
  const msg = JSON.parse(data);

  if (msg.type === 'event') {
    if (msg.event === 'channel.message') {
      console.log('\n[Channel Message]');
      console.log('  Platform:', msg.payload.platform);
      console.log('  Source:', msg.payload.source);
      console.log('  Content:', msg.payload.content);
      console.log('  Time:', new Date(msg.payload.ts * 1000));
    } else if (msg.event === 'tick') {
      process.stdout.write('.');
    }
  } else if (msg.type === 'res') {
    console.log('\nResponse:', JSON.stringify(msg, null, 2));
  }
});

// Send a message after 2 seconds
setTimeout(() => {
  const request = {
    type: 'req',
    id: 'send-1',
    method: 'send',
    params: {
      target: 'discord:123456789012345678',
      message: 'Hello from Gateway!'
    }
  };

  ws.send(JSON.stringify(request));
  console.log('Sent message request');
}, 2000);
```

## Discord Bot 设置

### 1. 创建 Discord 应用

访问 https://discord.com/developers/applications 并创建一个新的应用。

### 2. 创建 Bot 用户

- 在应用设置中，转到 "Bot" 部分
- 点击 "Add Bot"
- 复制 Bot Token（用于 `DISCORD_BOT_TOKEN` 环境变量）

### 3. 配置 Bot 权限

Bot 需要以下权限：
- **Send Messages** - 发送消息
- **Read Messages/View Channels** - 读取消息
- **Message Content** - 读取消息内容（用于接收消息）

### 4. 邀请 Bot 到服务器

使用 OAuth2 URL 生成器邀请 Bot 到你的 Discord 服务器：
```
https://discord.com/api/oauth2/authorize?client_id=YOUR_CLIENT_ID&permissions=68608&scope=bot
```

### 5. 配置 Intents

Discord 需要启用 **Message Content Intent** 才能接收消息内容：
1. 在 Discord Developer Portal
2. 选择你的应用 → Bot
3. 滚动到 "Privileged Gateway Intents"
4. 启用 "Message Content Intent"

## 故障排查

### 没有接收到频道消息

**检查：**
1. Discord Bot Token 是否正确设置？
```bash
echo $DISCORD_BOT_TOKEN
```

2. Bot 是否在服务器中？检查 Discord 开发者门户的连接数。

3. Message Content Intent 是否启用？

4. 是否有正确的权限（读取消息、发送消息）？

### 消息发送失败

**错误：** `Platform 'discord' not registered`

**解决：** 确保在启动服务器前调用了 `server.register_channel(discord).await`

**错误：** `Failed to resolve target`

**解决：** 检查目标格式，应该是 `discord:123456789012345678`

## 性能考虑

- **事件缓冲**：广播通道默认容量为 100 个事件
- **背压处理**：如果客户端处理速度慢，会收到 `Lagged` 警告并跳过事件
- **并发连接**：支持多个客户端同时接收事件

## 下一步

- [ ] 实现 Telegram 适配器
- [ ] 实现 WhatsApp 桥接
- [ ] 添加 Agent 自动响应功能
- [ ] 实现消息重试和错误恢复
- [ ] 添加消息队列和速率限制
- [ ] 实现持久化消息历史
