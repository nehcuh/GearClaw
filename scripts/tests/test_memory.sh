#!/bin/bash
# Memory Integration Test Script
#
# This script demonstrates and tests the Memory system integration

set -e

echo "🧠 GearClaw Memory Integration Test"
echo "===================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Use cargo run for CLI commands
GEARCLAW="cargo run --package gearclaw_cli --bin gearclaw_cli --"

# Check if workspace exists
WORKSPACE="$HOME/.gearclaw/workspace"
if [ ! -d "$WORKSPACE" ]; then
    echo "Creating workspace directory..."
    mkdir -p "$WORKSPACE"
fi

# Create test documents
echo "📝 Creating test documents..."
cat > "$WORKSPACE/api-docs.md" <<'EOF'
# API 文档

## 用户认证

使用 Bearer token 进行认证：

```
Authorization: Bearer YOUR_TOKEN
```

## 创建用户

POST /api/users

```json
{
  "name": "John Doe",
  "email": "john@example.com"
}
```

## 错误处理

常见错误码：
- 400: Bad Request
- 401: Unauthorized
- 404: Not Found
EOF

cat > "$WORKSPACE/setup-guide.md" <<'EOF'
# 设置指南

## 安装

运行以下命令安装 GearClaw：

```bash
cargo install --path .
```

## 配置

配置文件位于 `~/.gearclaw/config.toml`

### API Key

设置 API key：

```bash
export ANTHROPIC_API_KEY="your-key-here"
```

### Workspace

默认 workspace 位置：`~/.gearclaw/workspace`

## 首次使用

1. 创建配置文件
2. 设置 API key
3. 运行 `gearclaw chat`
EOF

echo -e "${GREEN}✅ Created test documents${NC}"
echo ""

# Test 1: Memory Sync
echo "🔄 Test 1: Memory Sync"
echo "--------------------"
$GEARCLAW memory sync
echo ""

# Test 2: Memory Search
echo "🔍 Test 2: Memory Search"
echo "-----------------------"
echo "Searching for 'API authentication'..."
$GEARCLAW memory search "API authentication"
echo ""

echo "Searching for 'configuration'..."
$GEARCLAW memory search "configuration"
echo ""

echo "Searching for 'error handling'..."
$GEARCLAW memory search "error handling"
echo ""

# Test 3: Chat with Memory
echo "💬 Test 3: Chat with Memory Integration"
echo "----------------------------------------"
echo "Starting interactive chat (type 'exit' to quit)..."
echo ""
echo "Try asking:"
echo "  - 如何使用 API 进行认证？"
echo "  - 配置文件在哪里？"
echo "  - 有哪些错误码？"
echo ""

# Note: We can't automate the interactive chat, so we provide instructions
echo -e "${BLUE}To test memory integration in chat, run:${NC}"
echo "  cargo run --package gearclaw_cli --bin gearclaw_cli -- chat"
echo ""
echo "Then ask questions based on the documents we created."
echo ""

# Summary
echo "=========================================="
echo -e "${GREEN}✅ Memory Integration Test Summary${NC}"
echo "=========================================="
echo ""
echo "✅ Test documents created in: $WORKSPACE"
echo "✅ Memory sync completed"
echo "✅ Memory search tested"
echo ""
echo "Next steps:"
echo "  1. Run: cargo run --package gearclaw_cli --bin gearclaw_cli -- chat"
echo "  2. Ask questions about the test documents"
echo "  3. Verify that Agent uses memory context"
echo ""
echo "Example questions:"
echo "  - 'API 认证是怎么工作的？'"
echo "  - '配置文件放在哪里？'"
echo "  - '有哪些常见的错误码？'"
echo ""
