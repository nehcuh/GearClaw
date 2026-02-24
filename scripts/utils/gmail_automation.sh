#!/bin/bash
# Gmail 自动化快速启动脚本
#
# 使用方法:
#   ./gmail_automation.sh
#
# 功能:
#   1. 启动 Chrome 浏览器
#   2. 访问 Gmail
#   3. 截图保存
#   4. 等待用户登录
#   5. 查找未读邮件

set -e

echo "🚀 Gmail 自动化快速启动"
echo "========================="
echo ""

# 检查 Node.js
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 未找到 Node.js"
    echo "请先安装 Node.js: https://nodejs.org/"
    exit 1
fi

# 检查 Puppeteer
if [ ! -d "/tmp/node_modules/puppeteer" ]; then
    echo "📦 安装 Puppeteer..."
    mkdir -p /tmp
    cd /tmp
    npm install puppeteer --silent
fi

# 使用已创建的脚本
echo "✅ 启动浏览器自动化..."
echo ""

cd /tmp
node gmail_with_chrome.js
