#!/bin/bash
# Discord Bot 诊断脚本

echo "🔍 GearClaw Discord Bot 诊断工具"
echo "=================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ISSUES_FOUND=0

# Test 1: Check if DISCORD_BOT_TOKEN is set
echo -e "${BLUE}检查 1: Discord Bot Token${NC}"
echo "-------------------------------"

if [ -z "$DISCORD_BOT_TOKEN" ]; then
    echo -e "${RED}❌ DISCORD_BOT_TOKEN 环境变量未设置！${NC}"
    echo ""
    echo "这是最主要的问题！请设置："
    echo ""
    echo "  export DISCORD_BOT_TOKEN='你的_token'"
    echo ""
    echo "或添加到 ~/.zshrc："
    echo "  echo 'export DISCORD_BOT_TOKEN=\"你的_token\"' >> ~/.zshrc"
    echo "  source ~/.zshrc"
    echo ""
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    TOKEN_LEN=${#DISCORD_BOT_TOKEN}
    if [ "$TOKEN_LEN" -lt 50 ]; then
        echo -e "${RED}❌ Token 长度不正确（长度: $TOKEN_LEN）${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✅ Token 已设置（长度: $TOKEN_LEN）${NC}"
    fi
fi
echo ""

# Test 2: Check if gateway is running
echo -e "${BLUE}检查 2: Gateway 服务状态${NC}"
echo "-------------------------------"

GATEWAY_PROCESS=$(ps aux | grep -i "gearclaw.*gateway\|target/debug/gearclaw" | grep -v grep | head -1)

if [ -z "$GATEWAY_PROCESS" ]; then
    echo -e "${RED}❌ Gateway 服务未运行！${NC}"
    echo ""
    echo "请启动服务："
    echo ""
    echo "  cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway"
    echo ""
    echo "或使用启动脚本："
    echo "  ./start_discord.sh"
    echo ""
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}✅ Gateway 服务正在运行${NC}"
    echo "$GATEWAY_PROCESS"
fi
echo ""

# Test 3: Check Discord configuration
echo -e "${BLUE}检查 3: Discord 配置文件${NC}"
echo "-------------------------------"

CONFIG_FILE="$HOME/.gearclaw/config.toml"

if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}✅ 配置文件存在: $CONFIG_FILE${NC}"

    # Check for agent configuration
    if grep -q "\[agent\]" "$CONFIG_FILE"; then
        echo -e "${GREEN}✅ [agent] 配置存在${NC}"

        # Check for enabled_channels
        if grep -q "enabled_channels" "$CONFIG_FILE"; then
            echo -e "${YELLOW}⚠️  发现 enabled_channels 配置${NC}"
            echo ""
            echo "当前配置："
            grep -A 5 "enabled_channels" "$CONFIG_FILE" | head -6
            echo ""
            echo "注意：如果启用了 enabled_channels，"
            echo "请确保你的频道 ID 在列表中！"
        fi
    else
        echo -e "${YELLOW}⚠️  [agent] 配置不存在，使用默认配置${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  配置文件不存在，将使用默认配置${NC}"
fi
echo ""

# Test 4: Check required intents
echo -e "${BLUE}检查 4: Discord Developer Portal 设置${NC}"
echo "-------------------------------"

echo "请确认在 Discord Developer Portal 中："
echo ""
echo "1. 访问: https://discord.com/developers/applications"
echo "2. 选择你的应用 → Bot"
echo "3. 在 'Privileged Gateway Intents' 部分："
echo "   ${YELLOW}✅ MESSAGE CONTENT INTENT${NC} （必须！）"
echo "   ✅ SERVER MEMBERS INTENT（可选）"
echo "   ✅ PRESENCE INTENT（可选）"
echo ""
echo "如果 MESSAGE CONTENT INTENT 未启用，Bot 无法读取消息！"
echo ""

# Test 5: Check Bot permissions in server
echo -e "${BLUE}检查 5: Discord Bot 权限${NC}"
echo "-------------------------------"

echo "在 Discord 服务器中，请确认 Bot 有以下权限："
echo ""
echo "必需权限："
echo "  ✅ Send Messages（发送消息）"
echo "  ✅ Read Messages/View Channels（读取消息）"
echo "  ✅ Read Message History（读取历史）"
echo ""
echo "检查方法："
echo "1. 服务器设置 → 角色"
echo "2. 找到你的 Bot 角色"
echo "3. 查看权限列表"
echo ""

# Test 6: Try to show recent logs
echo -e "${BLUE}检查 6: 查看日志${NC}"
echo "-------------------------------"

if command -v journalctl &> /dev/null; then
    echo "最近的服务日志："
    journalctl -u gearclaw -n 20 --no-pager 2>/dev/null || echo "  （没有找到 systemd 日志）"
else
    echo "提示：启动服务时查看输出，寻找以下信息："
    echo "  ✅ 'Discord adapter starting'"
    echo "  ✅ 'Discord Gateway shard created'"
    echo "  ✅ 'Discord Gateway connected'"
    echo ""
    echo "错误信息可能包括："
    echo "  ❌ 'Disallowed intent: MESSAGE_CONTENT is required'"
    echo "  ❌ '401 Unauthorized' - Token 错误"
    echo "  ❌ '403 Forbidden' - 权限不足"
fi
echo ""

# Summary
echo "=========================================="
if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ 没有发现明显问题${NC}"
else
    echo -e "${RED}❌ 发现 $ISSUES_FOUND 个问题${NC}"
fi
echo "=========================================="
echo ""

# Solutions
echo "🔧 常见问题解决方案："
echo ""
echo "1. Token 未设置："
echo "   export DISCORD_BOT_TOKEN='你的_token'"
echo ""
echo "2. MESSAGE CONTENT INTENT 未启用："
echo "   Discord Developer Portal → Bot → Privileged Gateway Intents"
echo "   ✅ MESSAGE CONTENT INTENT → Save Changes"
echo ""
echo "3. 服务未运行："
echo "   RUST_LOG=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway"
echo ""
echo "4. Bot 权限不足："
echo "   重新邀请 Bot，确保勾选所有必需权限"
echo ""
echo "5. 频道白名单问题："
echo "   编辑 ~/.gearclaw/config.toml"
echo "   移除或更新 enabled_channels 配置"
echo ""

# Test command
echo "🧪 测试命令："
echo ""
echo "设置 Token 并启动服务（带调试日志）："
echo ""
echo -e "${GREEN}export DISCORD_BOT_TOKEN='你的_token'${NC}"
echo -e "${GREEN}RUST_LOG=gearclaw_channels=debug,gearclaw_gateway=debug cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway${NC}"
echo ""
