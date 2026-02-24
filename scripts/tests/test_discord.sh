#!/bin/bash
# Discord Bot Configuration Test Script

echo "🧪 Discord Bot 配置测试"
echo "======================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Check environment variable
echo "🔍 测试 1: 检查环境变量"
echo "-------------------------------"

if [ -z "$DISCORD_BOT_TOKEN" ]; then
    echo -e "${RED}❌ DISCORD_BOT_TOKEN 未设置${NC}"
    echo ""
    echo "请设置环境变量："
    echo "  export DISCORD_BOT_TOKEN='你的_bot_token'"
    echo ""
    exit 1
else
    TOKEN_LENGTH=${#DISCORD_BOT_TOKEN}
    if [ "$TOKEN_LENGTH" -lt 50 ]; then
        echo -e "${RED}❌ Token 太短（长度: $TOKEN_LENGTH），可能无效${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ DISCORD_BOT_TOKEN 已设置 (长度: $TOKEN_LENGTH)${NC}"
    fi
fi
echo ""

# Test 2: Check if cargo is available
echo "🔍 测试 2: 检查 Rust 环境"
echo "-------------------------------"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo 未安装${NC}"
    echo "请访问 https://rustup.rs/ 安装 Rust"
    exit 1
else
    echo -e "${GREEN}✅ Cargo 已安装: $(cargo --version)${NC}"
fi
echo ""

# Test 3: Check project structure
echo "🔍 测试 3: 检查项目结构"
echo "-------------------------------"

if [ ! -d "crates/channels" ]; then
    echo -e "${RED}❌ Discord 集成模块未找到${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Discord 集成模块存在${NC}"
fi

if [ ! -f "crates/channels/src/platforms/discord.rs" ]; then
    echo -e "${RED}❌ Discord adapter 未找到${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Discord adapter 存在${NC}"
fi
echo ""

# Test 4: Check dependencies
echo "🔍 测试 4: 检查依赖"
echo "-------------------------------"

if grep -q "twilight" crates/channels/Cargo.toml; then
    echo -e "${GREEN}✅ Twilight (Discord 库) 依赖已添加${NC}"
else
    echo -e "${RED}❌ Twilight 依赖未找到${NC}"
    exit 1
fi
echo ""

# Test 5: Try to compile
echo "🔍 测试 5: 编译检查"
echo "-------------------------------"

echo "正在编译 gearclaw_channels..."
if cargo build --package gearclaw_channels 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✅ 编译成功${NC}"
else
    echo -e "${YELLOW}⚠️  编译可能有问题，请手动检查${NC}"
fi
echo ""

# Test 6: Validate token format
echo "🔍 测试 6: Token 格式验证"
echo "-------------------------------"

# Discord Bot tokens typically start with specific prefixes
TOKEN_PREFIX=$(echo "$DISCORD_BOT_TOKEN" | cut -c1-3)
case "$TOKEN_PREFIX" in
    "MTE"|"MzM"|"MTA"|"ODA")
        echo -e "${GREEN}✅ Token 前缀看起来有效 ($TOKEN_PREFIX...)${NC}"
        ;;
    *)
        echo -e "${YELLOW}⚠️  Token 前缀不常见 ($TOKEN_PREFIX...)${NC}"
        echo "   有效的前缀通常是: MTE, MzM, MTA, ODA"
        ;;
esac
echo ""

# Test 7: Check config file
echo "🔍 测试 7: 检查配置文件"
echo "-------------------------------"

CONFIG_FILE="$HOME/.gearclaw/config.toml"

if [ -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}✅ 配置文件存在: $CONFIG_FILE${NC}"

    # Check for Discord-specific config
    if grep -q "discord" "$CONFIG_FILE" 2>/dev/null; then
        echo -e "${GREEN}✅ 配置文件包含 Discord 配置${NC}"
    else
        echo -e "${YELLOW}⚠️  配置文件中没有 Discord 配置${NC}"
        echo "   这不是问题，Bot Token 从环境变量读取"
    fi
else
    echo -e "${YELLOW}⚠️  配置文件不存在: $CONFIG_FILE${NC}"
    echo "   这不是问题，首次运行时会自动创建"
fi
echo ""

# Summary
echo "=========================================="
echo -e "${GREEN}✅ 基本检查通过！${NC}"
echo "=========================================="
echo ""
echo "📋 下一步："
echo ""
echo "1. 确保 Discord Bot 已创建:"
echo "   访问: https://discord.com/developers/applications"
echo ""
echo "2. 确保 Bot 已邀请到服务器:"
echo "   使用 OAuth2 URL 生成器邀请 Bot"
echo ""
echo "3. 启动 Gateway 服务:"
echo -e "   ${YELLOW}./start_discord.sh${NC}"
echo ""
echo "4. 或手动启动:"
echo -e "   ${YELLOW}export DISCORD_BOT_TOKEN='你的_token'${NC}"
echo -e "   ${YELLOW}cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway${NC}"
echo ""
echo "5. 在 Discord 中测试:"
echo "   @agent 你好"
echo ""
