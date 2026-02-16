#!/bin/bash
# Discord Bot Setup Assistant
#
# 这个脚本会引导你完成 Discord Bot 的配置和邀请

echo "🤖 GearClaw Discord Bot 配置助手"
echo "=================================="
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Step 1: Check if token is set
echo -e "${BLUE}步骤 1: 检查 Discord Bot Token${NC}"
echo "-------------------------------"

if [ -z "$DISCORD_BOT_TOKEN" ]; then
    echo -e "${YELLOW}⚠️  DISCORD_BOT_TOKEN 环境变量未设置${NC}"
    echo ""
    echo "请按以下步骤获取 Token："
    echo ""
    echo "1️⃣  访问: https://discord.com/developers/applications"
    echo "2️⃣  点击 'New Application' 创建应用"
    echo "3️⃣  命名应用（如 'GearClaw Bot'）并创建"
    echo "4️⃣  在左侧菜单点击 'Bot'"
    echo "5️⃣  点击 'Add Bot' → 'Yes, do it!'"
    echo "6️⃣  点击 'Reset Token' 或复制现有 Token"
    echo "7️⃣  复制 Token（格式：MTAwNjMw... 很长的字符串）"
    echo ""
    echo "然后设置环境变量："
    echo -e "${GREEN}export DISCORD_BOT_TOKEN='你复制的_token'${NC}"
    echo ""
    read -p "按 Enter 继续，或者先按 Ctrl+C 退出去设置 Token..."
else
    TOKEN_LENGTH=${#DISCORD_BOT_TOKEN}
    if [ "$TOKEN_LENGTH" -gt 50 ]; then
        echo -e "${GREEN}✅ DISCORD_BOT_TOKEN 已设置 (长度: $TOKEN_LENGTH)${NC}"
    else
        echo -e "${YELLOW}⚠️  Token 长度不正确 (长度: $TOKEN_LENGTH)${NC}"
    fi
fi
echo ""

# Step 2: Configure Bot permissions
echo -e "${BLUE}步骤 2: 配置 Bot 权限${NC}"
echo "-------------------------------"
echo ""
echo "回到 Discord Developer Portal:"
echo "1️⃣  确保在 'Bot' 页面"
echo "2️⃣  在 'Privileged Gateway Intents' 部分："
echo "    ✅ 勾选 'MESSAGE CONTENT INTENT'（必需！）"
echo "    ✅ 可选勾选 'SERVER MEMBERS INTENT'"
echo "    ✅ 可选勾选 'PRESENCE INTENT'"
echo "3️⃣  点击 'Save Changes'"
echo ""
read -p "配置完成后按 Enter 继续..."
echo ""

# Step 3: Generate OAuth2 URL
echo -e "${BLUE}步骤 3: 生成 OAuth2 邀请链接${NC}"
echo "-------------------------------"
echo ""
echo "在 Discord Developer Portal:"
echo ""
echo "1️⃣  在左侧菜单点击 'OAuth2' → 'URL Generator'"
echo ""
echo "2️⃣  在 'Scopes' 部分，勾选："
echo "    ✅ bot"
echo "    ✅ applications.commands（可选）"
echo ""
echo "3️⃣  在 'Bot Permissions' 部分，勾选："
echo "    ✅ Send Messages"
echo "    ✅ Read Messages/View Channels"
echo "    ✅ Read Message History"
echo "    ✅ Add Reactions"
echo ""
echo "4️⃣  复制生成的 URL（在页面底部）"
echo ""
echo "生成的 URL 应该类似："
echo -e "${YELLOW}https://discord.com/oauth2/authorize?client_id=YOUR_ID&permissions=274878024768&scope=bot${NC}"
echo ""
read -p "复制 URL 后按 Enter 继续..."
echo ""

# Step 4: Invite bot to server
echo -e "${BLUE}步骤 4: 邀请 Bot 到服务器${NC}"
echo "-------------------------------"
echo ""
echo "1️⃣  在浏览器中打开刚才复制的 URL"
echo "2️⃣  选择你的服务器"
echo "3️⃣  点击 '授权' 或 '继续'"
echo "4️⃣  完成人机验证（如果需要）"
echo ""
echo -e "${GREEN}✅ Bot 现在应该在你的服务器中了！${NC}"
echo ""
read -p "按 Enter 继续..."
echo ""

# Step 5: Get channel ID
echo -e "${BLUE}步骤 5: 获取频道 ID（可选）${NC}"
echo "-------------------------------"
echo ""
echo "如果你想限制 Bot 只在特定频道响应，需要频道 ID："
echo ""
echo "1️⃣  Discord 设置 → Advanced → 开启 'Developer Mode'"
echo "2️⃣  右键点击你想让 Bot 工作的频道"
echo "3️⃣  点击 'Copy ID'"
echo "4️⃣  将频道 ID 添加到配置文件"
echo ""
echo "获取到的 ID 格式类似：${YELLOW}123456789012345678${NC}"
echo ""
read -p "获取频道 ID 后按 Enter 继续，或直接按 Enter 跳过..."
echo ""

# Step 6: Configure GearClaw
echo -e "${BLUE}步骤 6: 配置 GearClaw${NC}"
echo "-------------------------------"
echo ""
echo "编辑配置文件："
echo -e "${YELLOW}~/.gearclaw/config.toml${NC}"
echo ""
echo "添加以下配置："
echo ""
cat << 'EOF'
[agent]
# 触发模式
trigger_mode = "mention"  # mention | keyword | auto

# 提及触发词
mention_patterns = ["@agent", "@bot", "@gearclaw"]

# 只在特定频道响应（可选）
enabled_channels = [
    "discord:你的频道ID",
]

# 或设置黑名单（可选）
disabled_channels = [
    "discord:禁止的频道ID",
]
EOF
echo ""
read -p "配置完成后按 Enter 继续..."
echo ""

# Step 7: Start the bot
echo -e "${BLUE}步骤 7: 启动 GearClaw Gateway${NC}"
echo "-------------------------------"
echo ""
echo "现在可以启动服务了："
echo ""
echo -e "${GREEN}cargo run --package gearclaw_cli --bin gearclaw_cli -- gateway${NC}"
echo ""
echo "或者使用启动脚本："
echo -e "${GREEN}./start_discord.sh${NC}"
echo ""
echo "启动后，在 Discord 频道中输入："
echo -e "${YELLOW}@agent 你好${NC}"
echo ""
echo "如果 Bot 回复，说明配置成功！🎉"
echo ""

# Final checks
echo "=========================================="
echo -e "${GREEN}✅ 配置完成！${NC}"
echo "=========================================="
echo ""
echo "📋 快速检查清单："
echo ""
echo "  ✅ Discord Bot Token 已设置"
echo "  ✅ MESSAGE CONTENT INTENT 已开启"
echo "  ✅ Bot 已邀请到服务器"
echo "  ✅ 频道 ID 已获取（可选）"
echo "  ✅ GearClaw 配置已更新"
echo ""
echo "🚀 下一步："
echo ""
echo "  1. 启动服务: ./start_discord.sh"
echo "  2. 在 Discord 测试: @agent 你好"
echo ""
echo "📚 需要帮助？查看完整文档："
echo "   https://github.com/your-repo/DISCORD_SETUP.md"
echo ""
