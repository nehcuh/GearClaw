# 08 Memory 记忆系统

## 1. 功能说明

Memory 子系统提供文档向量化索引与相似度检索能力，支持将检索结果自动注入 Agent 上下文。

## 2. 工作流程

1. 扫描 workspace 下 markdown 文档（`**/*.md`）  
2. 文本分块（按段落）  
3. 调用 embedding 接口向量化  
4. 写入 SQLite（files/chunks/meta）  
5. 用户提问时执行向量检索并返回 Top-K

## 3. 配置项

```toml
[agent]
workspace = "~/.gearclaw/workspace"
memory_enabled = true

[memory]
enabled = true
db_path = "~/.gearclaw/memory/index.sqlite"
```

说明：

1. `memory.enabled` 控制索引流程  
2. `agent.memory_enabled` 控制对话检索注入流程

## 4. 常用命令

```bash
# 同步索引
cargo run -p gearclaw_cli -- memory sync

# 查询
cargo run -p gearclaw_cli -- memory search "认证方式"
```

## 5. 对话注入机制

当命中相关片段时，Agent 会在 system prompt 中追加 “Relevant Context” 区块，作为回答前置上下文。

## 6. 调试建议

1. 开启 `RUST_LOG=debug` 观察检索日志  
2. 确认 workspace 路径与文件格式  
3. 检查数据库路径是否可写  
4. 确认 embedding 接口可用

## 7. 常见问题

1. 无检索结果：未同步或关键词不在语料中  
2. 结果不相关：文档结构不清晰、关键词过泛  
3. 同步失败：API key/网络/模型不可用

## 8. 导航

- 上一篇：[`07-Agent触发机制.md`](./07-Agent触发机制.md)  
- 下一篇：[`09-Discord接入指南.md`](./09-Discord接入指南.md)
