#!/bin/bash
# LMStudio Embedding 诊断脚本

echo "🔍 LMStudio Embedding 诊断"
echo "================================"
echo ""

# 配置
ENDPOINT="http://127.0.0.1:1234/v1"
MODEL="qwen3-embedding-8b-mlx"  # 这是您当前配置的模型
API_KEY="xxx"

echo "📋 当前配置:"
echo "  Endpoint: $ENDPOINT"
echo "  Model: $MODEL"
echo ""

# 测试 1: 检查 LMStudio 是否运行
echo "🔗 测试 1: 检查 LMStudio 服务"
echo "-------------------------------"
HEALTH_CHECK=$(curl -s "$ENDPOINT/models" -H "Authorization: Bearer $API_KEY" 2>&1)
if [ $? -eq 0 ]; then
    echo "✅ LMStudio 服务正在运行"
    echo ""
    echo "可用模型:"
    echo "$HEALTH_CHECK" | python3 -m json.tool 2>/dev/null || echo "$HEALTH_CHECK"
else
    echo "❌ 无法连接到 LMStudio"
    echo "   请确保 LMStudio 正在运行并且端口是 1234"
    exit 1
fi
echo ""

# 测试 2: 检查模型列表
echo ""
echo "🔍 测试 2: 查找 Embedding 模型"
echo "-------------------------------"
MODELS=$(curl -s "$ENDPOINT/models" -H "Authorization: Bearer $API_KEY")
echo "$MODELS" | python3 -c "
import json, sys
data = json.load(sys.stdin)
print('所有模型:')
for obj in data.get('data', []):
    model_id = obj.get('id', 'unknown')
    model_type = '未知'

    # 检查模型名称
    if 'embedding' in model_id.lower() or 'embed' in model_id.lower():
        model_type = '✅ EMBEDDING 模型'
    elif 'chat' in model_id.lower() or 'gpt' in model_id.lower() or 'claude' in model_id.lower():
        model_type = '❌ 聊天模型 (不能用于 embedding)'

    print(f'  - {model_id}')
    print(f'    {model_type}')
    print()
" 2>/dev/null || echo "无法解析模型列表"
echo ""

# 测试 3: 尝试调用 embedding API
echo "🧪 测试 3: 测试 Embedding API"
echo "-----------------------------"
echo "使用模型: $MODEL"
echo ""

RESPONSE=$(curl -s "$ENDPOINT/embeddings" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"input\": \"test text\"
  }" 2>&1)

echo "响应:"
echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE"
echo ""

# 检查是否有错误
if echo "$RESPONSE" | grep -q "error"; then
    echo "❌ Embedding API 返回错误"
    echo ""
    echo "可能的原因:"
    echo "  1. 模型 '$MODEL' 不是 embedding 模型"
    echo "  2. LMStudio 未加载该模型"
    echo "  3. 模型名称不正确"
    echo ""
    echo "💡 解决方案:"
    echo "  1. 在 LMStudio 中加载正确的 embedding 模型"
    echo "  2. 更新 config.toml 中的 embedding_model 名称"
    echo "  3. 或使用远程 embedding API (如 OpenAI)"
else
    echo "✅ Embedding API 工作正常！"
fi
echo ""

# 测试 4: 列出推荐的 embedding 模型
echo "📚 LMStudio 推荐的 Embedding 模型"
echo "------------------------------------"
echo "常见的本地 embedding 模型:"
echo "  • nomic-ai/nomic-embed-text-v1.5"
echo "  • sentence-transformers/all-MiniLM-L6-v2"
echo "  • BAAI/bge-small-en-v1.5"
echo "  • where-is-ai/political-mpt-7b-embedding"
echo ""
echo "请在 LMStudio 中搜索并加载这些模型之一"
echo ""
