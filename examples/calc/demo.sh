#!/bin/bash
# Guard Tool デモ - シンプル版

echo "🛡️  Guard Tool Demo (Simple Calculator)"
echo "========================================"
echo ""

# 色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 1. Git初期化チェック
if [ ! -d .git ]; then
    echo "📋 Initializing Git repository..."
    git init
    git add .
    git commit -m "Initial commit"
    echo -e "${GREEN}✅ Git initialized${NC}"
else
    echo -e "${GREEN}✅ Git repository exists${NC}"
fi
echo ""

# 2. 保護ファイルの確認
echo "📋 Protected Files:"
echo "  🔴 src/config.rs    (全体保護)"
echo "  🟡 src/main.rs      (10-25行のみ保護)"
echo "  🟢 src/calc.rs      (保護なし)"
echo ""

# 3. シミュレーション1: 保護されたファイルの変更
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Test 1: 保護されたファイルを変更${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${YELLOW}🤖 Agent: \"税率を8%に変更します...\"${NC}"

# config.rsをバックアップ
cp src/config.rs src/config.rs.backup

# 税率を変更してみる
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' 's/TAX_RATE: f64 = 0.10/TAX_RATE: f64 = 0.08/' src/config.rs
else
    sed -i 's/TAX_RATE: f64 = 0.10/TAX_RATE: f64 = 0.08/' src/config.rs
fi

echo "→ src/config.rs の16行目を変更"
echo ""
echo -e "${RED}🛡️  GUARD VIOLATION DETECTED!${NC}"
echo "Protected range modified in src/config.rs"
echo "  Lines: 16"
echo "  Reason: 重要な定数を含むため変更禁止"
echo ""
echo -e "${YELLOW}⏪ Rolling back...${NC}"

# ロールバック
git restore src/config.rs
rm -f src/config.rs.backup

echo -e "${GREEN}✅ File restored${NC}"
echo ""

# 4. シミュレーション2: 保護されていないファイルの変更
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Test 2: 保護されていないファイルを変更${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${YELLOW}🤖 Agent: \"平方根の計算機能を追加します...\"${NC}"

# calc.rsに関数を追加
cat >> src/calc.rs << 'EOF'

/// 平方根を計算
pub fn sqrt(a: f64) -> Result<f64, String> {
    if a < 0.0 {
        return Err("Error: Cannot calculate square root of negative number".to_string());
    }
    let result = a.sqrt();
    Ok(round(result))
}
EOF

echo "→ src/calc.rs に sqrt 関数を追加"
echo ""
echo -e "${GREEN}✅ Modification allowed (file is not protected)${NC}"
echo ""

# 元に戻す
git restore src/calc.rs

# 5. まとめ
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🔴 完全保護: src/config.rs"
echo "   → 税率の変更をブロック"
echo ""
echo "🟡 部分保護: src/main.rs (10-25行)"
echo "   → エラーハンドリングの変更をブロック"
echo ""
echo "🟢 変更OK: src/calc.rs"
echo "   → 新機能の追加を許可"
echo ""
echo -e "${GREEN}✅ Guard tool is working correctly!${NC}"
echo ""
echo "Next: Run 'guard codex' to try with real agent"
