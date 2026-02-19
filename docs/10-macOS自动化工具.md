# 10 macOS 自动化工具

## 1. 能力概览

macOS 平台下，Agent 可通过 `macos_*` 工具执行自动化操作，包括：

1. 应用管理（启动/退出/前台/运行状态）
2. 脚本执行（AppleScript / JXA）
3. 输入模拟（文本输入、组合键）
4. 剪贴板读写
5. 通知与系统操作（open url / say / browser search）
6. **应用内容读取**（前台应用检测、辅助功能权限、选中文本、焦点字段、文档内容）

## 2. 常见工具

### 2.1 应用管理

1. `macos_launch_app`
2. `macos_quit_app`
3. `macos_bring_to_front`
4. `macos_is_running`

### 2.2 脚本执行

5. `macos_applescript`
6. `macos_jxa`

### 2.3 输入模拟

7. `macos_type_text`
8. `macos_key_combo`

### 2.4 剪贴板

9. `macos_clipboard_read` / `macos_clipboard_write`

### 2.5 系统操作

10. `macos_notify`
11. `macos_open_url` / `macos_search_in_browser`
12. `macos_say`

### 2.6 内容读取（新）

13. `macos_get_frontmost_app` — 获取当前前台应用名称
14. `macos_check_accessibility` — 检测辅助功能权限状态
15. `macos_read_selected_text` — 读取当前选中文本（剪贴板中转）
16. `macos_read_focused_field` — 通过 AX API 读取焦点元素文本
17. `macos_read_document` — 读取应用当前文档内容

## 3. 内容读取工具详解

### 推荐调用顺序

```
1. macos_check_accessibility   ← 确认权限已授予
2. macos_get_frontmost_app     ← 确认目标应用已在前台
3. macos_read_selected_text    ← 优先尝试（最广泛兼容）
   或 macos_read_focused_field ← 读取输入框内容
   或 macos_read_document      ← 读取全文档（TextEdit/Notes/Safari/Terminal）
```

### macos_read_selected_text

通过模拟 Cmd+C 并监控剪贴板变化读取选中文本。

| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `preserve_clipboard` | bool | `true` | 读取后是否还原剪贴板 |
| `timeout_ms` | int | 300 | 等待剪贴板更新毫秒数 |
| `max_chars` | int | 0 | 最大返回字符数（0 = 不限制） |

成功返回格式：`[N chars, clipboard restored] <文本内容>`

错误码：`ERROR:CLIPBOARD_UNCHANGED`、`ERROR:TIMEOUT`、`ERROR:PERMISSION_DENIED`、`ERROR:SCRIPT_ERROR`

### macos_read_focused_field

通过 Accessibility API 读取当前焦点 UI 元素的文本内容。依次尝试 `value` → `name` → `description` 属性。

| 参数 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `app_name` | string | （可选） | 限制查询范围到指定进程 |
| `max_chars` | int | 0 | 最大返回字符数 |

成功返回格式：`[field: value, N chars] <文本内容>`

错误码：`ERROR:EMPTY_FIELD`、`ERROR:PERMISSION_DENIED`

### macos_read_document

内置支持应用：**TextEdit**、**Notes**、**Safari**、**Terminal**。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `app_name` | string | 是 | 应用名称，不区分大小写 |
| `max_chars` | int | 否 | 最大返回字符数 |

错误码：`ERROR:UNSUPPORTED_APP`、`ERROR:SCRIPT_ERROR`、`ERROR:EMPTY_FIELD`、`ERROR:PERMISSION_DENIED`、`ERROR:TIMEOUT`

## 4. 使用示例（自然语言）

1. “帮我打开 Safari 并搜索 Rust ownership”
2. “把剪贴板内容读出来”
3. “发送系统通知：任务完成”
4. “读取 TextEdit 当前文档的内容”
5. “我选中了一段文字，帮我读出来”

## 5. 权限要求

建议在系统设置中为终端或应用授予：

1. **辅助功能权限**（输入模拟、前台切换、选中文本读取等）
2. **自动化权限**（AppleScript 交互、文档读取等）

可调用 `macos_check_accessibility` 检测权限状态。

## 6. 常见问题

1. `Not authorized` / `ERROR:PERMISSION_DENIED`：在 系统设置 → 隐私与安全性 → 辅助功能 / 自动化 中授权
2. `ERROR:CLIPBOARD_UNCHANGED`：目标应用没有选中内容，或不支持复制操作
3. `ERROR:UNSUPPORTED_APP`：使用 `macos_read_selected_text` 作为通用替代
4. 应用找不到：确认 app 名称与安装状态
5. 组合键无效：检查按键名称是否在支持集合中

## 7. 实现提示

工具底层通过 `open`、`osascript`、`pbcopy/pbpaste`、`say` 等系统能力封装实现。所有 osascript 调用均有 5 秒超时保护。

## 8. 导航

- 上一篇：[`09-Discord接入指南.md`](./09-Discord接入指南.md)
- 下一篇：[`11-GUI桌面客户端.md`](./11-GUI桌面客户端.md)
