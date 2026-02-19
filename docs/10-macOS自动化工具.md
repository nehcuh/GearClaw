# 10 macOS 自动化工具

## 1. 能力概览

macOS 平台下，Agent 可通过 `macos_*` 工具执行自动化操作，包括：

1. 应用管理（启动/退出/前台/运行状态）  
2. 脚本执行（AppleScript / JXA）  
3. 输入模拟（文本输入、组合键）  
4. 剪贴板读写  
5. 通知与系统操作（open url / say / browser search）

## 2. 常见工具

1. `macos_launch_app`  
2. `macos_quit_app`  
3. `macos_bring_to_front`  
4. `macos_applescript`  
5. `macos_jxa`  
6. `macos_type_text`  
7. `macos_key_combo`  
8. `macos_clipboard_read` / `macos_clipboard_write`  
9. `macos_notify`  
10. `macos_open_url` / `macos_search_in_browser`  
11. `macos_say`

## 3. 使用示例（自然语言）

1. “帮我打开 Safari 并搜索 Rust ownership”  
2. “把剪贴板内容读出来”  
3. “发送系统通知：任务完成”

## 4. 权限要求

建议在系统设置中为终端或应用授予：

1. 辅助功能权限（输入模拟、前台切换等）  
2. 必要的自动化权限（AppleScript 交互）

## 5. 常见问题

1. `Not authorized`：通常是辅助功能或自动化权限未授权  
2. 应用找不到：确认 app 名称与安装状态  
3. 组合键无效：检查按键名称是否在支持集合中

## 6. 实现提示

工具底层通过 `open`、`osascript`、`pbcopy/pbpaste`、`say` 等系统能力封装实现。

## 7. 导航

- 上一篇：[`09-Discord接入指南.md`](./09-Discord接入指南.md)  
- 下一篇：[`11-GUI桌面客户端.md`](./11-GUI桌面客户端.md)
