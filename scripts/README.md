# Scripts 目录

本目录包含 GearClaw 项目的所有脚本工具。

## 📂 目录结构

```
scripts/
├── setup/       安装和配置脚本
├── tests/       测试和验证脚本
├── utils/       实用工具和诊断脚本
└── README.md    本文件
```

## 🚀 使用方法

### 运行测试

```bash
# 测试 Discord 连接
./scripts/tests/test_discord.sh

# 测试 Discord 消息
./scripts/tests/test_discord_message.sh

# 测试 Memory 系统
./scripts/tests/test_memory.sh
```

### 设置环境

```bash
# 设置 Discord Bot
./scripts/setup/setup_discord.sh

# 启动 Gateway 服务
./scripts/setup/start_gateway.sh
```

### 诊断工具

```bash
# 诊断 Discord 问题
./scripts/utils/diagnose_discord.sh

# 诊断 Embedding 问题
./scripts/utils/diagnose_embedding.sh

# 修复 Discord 配置
./scripts/utils/fix_discord.sh

# Gmail 自动化演示
./scripts/utils/gmail_automation.sh
```

## 📝 脚本说明

### Setup 脚本 (`setup/`)

| 脚本 | 说明 |
|:---|:---|
| `setup_discord.sh` | 初始化 Discord Bot 配置 |
| `start_gateway.sh` | 启动 Gateway 服务 |

### Test 脚本 (`tests/`)

| 脚本 | 说明 |
|:---|:---|
| `test_discord.sh` | 测试 Discord Bot 连接 |
| `test_discord_message.sh` | 测试 Discord 消息收发 |
| `test_memory.sh` | 测试 Memory 记忆系统 |

### Utils 脚本 (`utils/`)

| 脚本 | 说明 |
|:---|:---|
| `diagnose_discord.sh` | Discord 诊断工具 |
| `diagnose_embedding.sh` | Embedding 诊断工具 |
| `fix_discord.sh` | Discord 配置修复工具 |
| `gmail_automation.sh` | Gmail 自动化演示 |

## 🔧 添加新脚本

### 分类标准

1. **Setup**: 首次安装、初始化、配置
2. **Tests**: 验证功能、回归测试、冒烟测试
3. **Utils**: 日常使用的工具、调试、诊断

### 命名规范

- Setup 脚本: `setup_<feature>.sh`
- Test 脚本: `test_<feature>.sh`
- Utility 脚本: `<action>_<feature>.sh`

### 脚本模板

```bash
#!/bin/bash
# <脚本说明>
#
# 使用方法:
#   ./scripts/<category>/<script_name>.sh
#
# 功能:
#   1. ...
#   2. ...

set -e  # 遇到错误立即退出

# ... 脚本内容 ...
```

## ⚠️ 注意事项

1. 所有脚本应该有执行权限：`chmod +x scripts/**/*.sh`
2. 脚本应该使用 `set -e` 以在错误时退出
3. 添加清晰的注释说明用途
4. 遵循现有命名规范

## 🔗 相关文档

- [测试与验证](../docs/12-测试与验证.md)
- [Discord 接入指南](../docs/09-Discord接入指南.md)
