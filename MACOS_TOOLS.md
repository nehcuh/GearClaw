# macOS 自动化工具使用指南

GearClaw 现已支持完整的 macOS 应用程序控制和自动化功能！

## 功能概览

### 📱 应用程序管理
- `macos_launch_app` - 启动应用程序
- `macos_quit_app` - 退出应用程序
- `macos_bring_to_front` - 切换应用到前台
- `macos_is_running` - 检查应用是否运行

### 📜 脚本执行
- `macos_applescript` - 执行 AppleScript 代码
- `macos_jxa` - 执行 JavaScript for Automation (JXA)

### ⌨️ 输入模拟
- `macos_type_text` - 模拟键盘输入文本
- `macos_key_combo` - 模拟组合键 (如 Cmd+C, Cmd+V)

### 📋 剪贴板操作
- `macos_clipboard_read` - 读取剪贴板内容
- `macos_clipboard_write` - 写入剪贴板

### 🔔 通知系统
- `macos_notify` - 发送系统通知

### 🌐 系统操作
- `macos_open_url` - 在浏览器中打开 URL
- `macos_say` - 文字转语音 (TTS)

## 使用示例

### 对话模式中使用

#### 启动应用
```bash
cargo run -p gearclaw_cli

> 帮我打开 Safari
# GearClaw 会自动调用 macos_launch_app 工具
```

#### 检查应用状态
```bash
> Chrome 在运行吗？
# GearClaw 会调用 macos_is_running 工具
```

#### 发送通知
```bash
> 发个通知告诉我任务完成了
# GearClaw 会调用 macos_notify 工具
```

#### 剪贴板操作
```bash
> 读取剪贴板内容
# GearClaw 会调用 macos_clipboard_read 工具

> 把这句话复制到剪贴板：Hello World
# GearClaw 会调用 macos_clipboard_write 工具
```

#### 文字转语音
```bash
> 读出这段文字：任务已完成
# GearClaw 会调用 macos_say 工具
```

#### 打开网页
```bash
> 打开 GitHub
# GearClaw 会调用 macos_open_url 工具
```

#### 组合键操作
```bash
> 按下 Cmd+C
# GearClaw 会调用 macos_key_combo 工具
```

### 高级示例

#### AppleScript 自动化
```bash
> 用 AppleScript 创建一个新的提醒事项，内容是"下午3点开会"
# GearClaw 会生成并执行 AppleScript 代码
```

#### 多步骤工作流
```bash
> 帮我打开 Safari，访问 example.com，然后截个图
# GearClaw 会：
# 1. 启动 Safari (macos_launch_app)
# 2. 打开 URL (macos_open_url)
# 3. 执行截图命令 (exec)
```

## 安全权限

某些操作需要 macOS 系统权限：

### 辅助功能权限
- 键盘/鼠标模拟 (`macos_type_text`, `macos_key_combo`)
- 应用控制 (`macos_bring_to_front`, `macos_quit_app`)

**启用方法**：
1. 系统设置 → 隐私与安全性 → 辅助功能
2. 添加 Terminal 或你的应用

### 完整磁盘访问权限（可选）
- 读取所有应用信息
- 访问系统级文件

## API 参考

### 应用管理工具

#### macos_launch_app
```json
{
  "name": "macos_launch_app",
  "description": "启动 macOS 应用程序",
  "parameters": {
    "app_name": "Safari | Chrome | Terminal | ..."
  }
}
```

#### macos_quit_app
```json
{
  "name": "macos_quit_app",
  "description": "退出 macOS 应用程序",
  "parameters": {
    "app_name": "应用名称"
  }
}
```

#### macos_bring_to_front
```json
{
  "name": "macos_bring_to_front",
  "description": "将应用程序切换到前台",
  "parameters": {
    "app_name": "应用名称"
  }
}
```

#### macos_is_running
```json
{
  "name": "macos_is_running",
  "description": "检查应用是否正在运行",
  "parameters": {
    "app_name": "应用名称"
  }
}
```

### 脚本执行工具

#### macos_applescript
```json
{
  "name": "macos_applescript",
  "description": "执行 AppleScript 代码",
  "parameters": {
    "script": "tell application \"Finder\" to ..."
  }
}
```

#### macos_jxa
```json
{
  "name": "macos_jxa",
  "description": "执行 JavaScript for Automation",
  "parameters": {
    "script": "Application('Finder')...."
  }
}
```

### 输入模拟工具

#### macos_type_text
```json
{
  "name": "macos_type_text",
  "description": "模拟键盘输入文本",
  "parameters": {
    "text": "要输入的文本"
  }
}
```

#### macos_key_combo
```json
{
  "name": "macos_key_combo",
  "description": "模拟组合键",
  "parameters": {
    "keys": ["cmd", "c"] | ["cmd", "shift", "3"]
  }
}
```

### 剪贴板工具

#### macos_clipboard_read
```json
{
  "name": "macos_clipboard_read",
  "description": "读取剪贴板内容",
  "parameters": {}
}
```

#### macos_clipboard_write
```json
{
  "name": "macos_clipboard_write",
  "description": "写入剪贴板内容",
  "parameters": {
    "text": "要复制的文本"
  }
}
```

### 通知工具

#### macos_notify
```json
{
  "name": "macos_notify",
  "description": "发送系统通知",
  "parameters": {
    "title": "通知标题 (可选)",
    "message": "通知内容",
    "sound": false  // 是否播放提示音
  }
}
```

### 系统工具

#### macos_open_url
```json
{
  "name": "macos_open_url",
  "description": "在默认浏览器中打开 URL",
  "parameters": {
    "url": "https://..."
  }
}
```

#### macos_say
```json
{
  "name": "macos_say",
  "description": "文字转语音",
  "parameters": {
    "text": "要朗读的文本",
    "voice": "Ting-Ting (可选)",
    "rate": 175  // 语速
  }
}
```

## 常见应用名称

| 应用名称 | 说明 |
|---------|------|
| Safari | 浏览器 |
| Chrome | Chrome 浏览器 |
| Firefox | Firefox 浏览器 |
| Terminal | 终端 |
| iTerm | iTerm2 |
| Finder | 文件管理器 |
| System Events | 系统事件（AppleScript） |
| Music | 音乐 |
| Calendar | 日历 |
| Reminders | 提醒事项 |
| Notes | 备忘录 |
| Messages | 信息 |
| Mail | 邮件 |
| Photos | 照片 |

## 支持的按键

### 字母键
- `a` 到 `z`

### 功能键
- `tab` - Tab 键
- `return` / `enter` - 回车键
- `space` - 空格键
- `escape` / `esc` - Escape 键
- `delete` / `backspace` - 删除键

### 箭头键
- `up` - 上箭头
- `down` - 下箭头
- `left` - 左箭头
- `right` - 右箭头

### 修饰键
- `cmd` / `command` / `⌘` - Command 键
- `shift` - Shift 键
- `option` / `alt` - Option 键
- `control` / `ctrl` - Control 键

## 错误排查

### 辅助功能权限未授予
**错误**: `执行 AppleScript 失败: Not authorized`

**解决**: 系统设置 → 隐私与安全性 → 辅助功能 → 添加 Terminal

### 应用未找到
**错误**: `启动应用失败: Unable to find application`

**解决**: 检查应用名称拼写，确保应用已安装

### 组合键不支持
**错误**: `不支持的按键: xxx`

**解决**: 查看支持的按键列表，使用正确的按键名称

## 技术实现

这些工具通过以下方式实现：

- **应用管理**: `open` 命令 + AppleScript
- **脚本执行**: `osascript` 命令
- **输入模拟**: AppleScript (`System Events`)
- **剪贴板**: `pbpaste` / `pbcopy` 命令
- **通知**: AppleScript `display notification`
- **系统操作**: `open`, `say` 命令

所有工具都通过 `ToolExecutor` 集成，LLM 可以自动调用这些工具完成任务。

## 未来扩展

计划添加的功能：

- [ ] 鼠标点击和移动
- [ ] 窗口管理（大小、位置）
- [ ] 屏幕录制
- [ ] 更多系统传感器数据
- [ ] Voice Control 集成
- [ ] Shortcuts 应用集成
