#!/bin/bash
# Discord Bot 实时消息监控工具

echo "🔍 Discord Bot 实时监控"
echo "======================"
echo ""
echo "Gateway 进程状态："
ps aux | grep -E "gearclaw.*gateway|target/debug/gearclaw" | grep -v grep | head -2
echo ""

if [ -f /tmp/gateway_final.log ]; then
    echo "📊 最近的 Discord 事件："
    echo "-----------------------------------"
    tail -20 /tmp/gateway_final.log | grep -E "Received Discord event|Ready|MessageCreate|Agent response"
    echo ""

    echo "📈 事件统计："
    echo "-----------------------------------"
    echo "总事件数: $(grep -c "Received Discord event" /tmp/gateway_final.log)"
    echo "Ready 事件: $(grep -c "Ready" /tmp/gateway_final.log)"
    echo "MessageCreate 事件: $(grep -c "MessageCreate" /tmp/gateway_final.log)"
    echo ""

    echo "⏰ 最后更新时间："
    echo "-----------------------------------"
    tail -1 /tmp/gateway_final.log | grep -oE '\[.*\]'
else
    echo "❌ 日志文件不存在"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 提示：请在 Discord 中发送测试消息："
echo ""
echo "   @agent hello"
echo "   @agent 今天天气怎么样"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
