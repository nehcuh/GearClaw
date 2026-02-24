#!/bin/bash
# Discord Bot Quick Start Script

set -e

echo "🤖 GearClaw Discord Bot 快速启动"
echo "=================================="
echo ""

# Check if DISCORD_BOT_TOKEN is set
if [ -z "$DISCORD_BOT_TOKEN" ]; then
    echo "❌ 错误: DISCORD_BOT_TOKEN 环境变量未设置"
    echo ""
    echo "请按以下步骤设置："
    echo "1. 访问 https://discord.com/developers/applications"
    echo "2. 创建应用并获取 Bot Token"
    echo "3. 运行: export DISCORD_BOT_TOKEN='你的_token'"
    echo ""
    exit 1
fi

echo "✅ DISCORD_BOT_TOKEN 已设置"
echo ""

# Check if config file exists
CONFIG_FILE="$HOME/.gearclaw/config.toml"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "⚠️  配置文件不存在: $CONFIG_FILE"
    echo "正在创建默认配置..."
    mkdir -p "$HOME/.gearclaw"
    cargo run --package gearclaw_cli --bin gearclaw_cli -- config sample > "$CONFIG_FILE" 2>/dev/null || true
    echo "✓ 配置文件已创建"
fi

echo "📋 配置文件: $CONFIG_FILE"
echo ""

# Show current configuration
echo "🔧 当前 Discord 配置:"
echo "  - Bot Token: ${DISCORD_BOT_TOKEN:0:20}..."
echo "  - Config: $CONFIG_FILE"
echo ""

# Start the gateway
echo "🚀 启动 GearClaw Gateway (Discord 模式)..."
echo ""

# Build and run
cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway
