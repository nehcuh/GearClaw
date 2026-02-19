# 11 GUI 桌面客户端

## 1. 当前定位

GUI 提供图形化交互入口，主要覆盖：

1. 聊天视图  
2. 会话侧栏  
3. 设置视图  
4. 日志与状态面板

## 2. 环境要求

1. macOS 12.0+  
2. Xcode Command Line Tools（Metal 编译依赖）  
3. Rust 工具链

安装命令：

```bash
xcode-select --install
```

## 3. 运行方式

```bash
cargo run -p gearclaw_gui
```

## 4. 常见问题

1. `xcrun: unable to find utility "metal"`：通常是 Xcode CLT 未正确安装或未切换。  
2. `gpui` 相关依赖冲突：先尝试 `cargo update` 再重建。  
3. GUI 未纳入默认 workspace 构建：按需单独构建运行。

## 5. 导航

- 上一篇：[`10-macOS自动化工具.md`](./10-macOS自动化工具.md)  
- 下一篇：[`12-测试与验证.md`](./12-测试与验证.md)
