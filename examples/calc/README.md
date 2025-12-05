# Simple Calculator - Guard Tool サンプル

シンプルな計算機の例で、guardツールの基本的な使い方を学べます。

## 📁 プロジェクト構成（3ファイルのみ）

```
simple-calc/
├── .guard.toml          🛡️ 保護ルール
├── Cargo.toml
└── src/
    ├── main.rs          🟡 一部保護（10-25行目）
    ├── config.rs        🔴 完全保護（全行）
    └── calc.rs          🟢 変更OK
```

## 🎯 ファイルの役割

### 🔴 `src/config.rs` - 完全保護
重要な定数を定義：
- `PRECISION`: 計算精度
- `MAX_RESULT`: 結果の上限
- `TAX_RATE`: 消費税率

**なぜ保護？** → これらの値が勝手に変わると計算結果が狂う

### 🟡 `src/main.rs` - 一部保護（10-25行目）
エラーハンドリングとコマンドライン引数の処理

**なぜ保護？** → プログラムの安定性に関わる基本構造

### 🟢 `src/calc.rs` - 変更OK
計算ロジック（add, subtract, multiply, divide, with_tax）

**なぜOK？** → 新しい計算機能を自由に追加できる

## 🚀 使い方

### 1. セットアップ

```bash
cd simple-calc

# Gitリポジトリを初期化（guardに必須）
git init
git add .
git commit -m "Initial commit"
```

### 2. 普通に実行してみる

```bash
# ビルド
cargo build

# 実行
cargo run add 10 20      # → 30
cargo run mul 5 6        # → 30
cargo run div 100 3      # → 33.33
cargo run tax 1000       # → 1100 (消費税10%)
```

### 3. guardで保護しながらエージェントを起動

```bash
guard codex
# または
guard claude-code
```

## 💡 試してみよう

### ✅ 成功する例（calc.rsの変更）

エージェントに以下のように指示：

```
「平方根を計算する関数を追加してください」
```

**結果**: ✅ 成功
- `calc.rs`は保護されていないため、`sqrt`関数が追加される
- guardは何も警告しない

```rust
// calc.rsに追加される
pub fn sqrt(a: f64) -> Result<f64, String> {
    if a < 0.0 {
        return Err("Error: Cannot calculate square root of negative number".to_string());
    }
    let result = a.sqrt();
    Ok(round(result))
}
```

### 🛡️ ブロックされる例（config.rsの変更）

エージェントに以下のように指示：

```
「税率を8%に変更してください」
```

**結果**: 🛡️ ブロック
```
🛡️ GUARD VIOLATION DETECTED!
Protected range modified in src/config.rs
  Lines: 16
  Reason: 重要な定数（精度、最大値、税率など）を含むため変更禁止
⏪ Rolling back changes...
✅ File restored
```

### 🛡️ ブロックされる例（main.rsの保護範囲）

エージェントに以下のように指示：

```
「エラーメッセージを日本語にしてください」
```

**結果**: 🛡️ 一部ブロック
- 10-25行目のエラーハンドリングは保護されているため変更できない
- それ以外の部分（操作のmatch文など）は変更可能

## 🎓 学べること

1. **完全保護 vs 部分保護の違い**
   - `config.rs`: ファイル全体が保護
   - `main.rs`: 特定の行だけ保護
   - `calc.rs`: 保護なし

2. **なぜ保護が必要か**
   - 設定値: 勝手に変わると影響が大きい
   - エラー処理: 安定性に関わる
   - 計算ロジック: 自由に拡張したい

3. **エージェントとの協働**
   - 保護範囲外なら自由に作業させる
   - 保護範囲は自分で変更する

## 📊 .guard.toml の内容

```toml
[[guards]]
path = "src/config.rs"
ranges = [
  { start = 1, end = 999 },  # 全行保護
]

[[guards]]
path = "src/main.rs"
ranges = [
  { start = 10, end = 25 },  # 特定の行のみ保護
]

# src/calc.rs には guards がない → 変更OK
```

## 🔧 カスタマイズ

### 税率の変更を許可したい場合

`.guard.toml`から該当行を削除するか、範囲を調整：

```toml
[[guards]]
path = "src/config.rs"
ranges = [
  { start = 1, end = 12 },   # 税率の行（16行目）を除外
  { start = 18, end = 999 },
]
```

### 厳格モードを有効化

```toml
[settings]
strict_mode = true  # 違反時に即座に終了
```

## 🎯 次のステップ

1. エージェントに新しい機能を追加させてみる
2. 保護範囲を変えて動作を確認
3. 自分のプロジェクトに応用

---

**このサンプルのポイント**: たった3ファイルで、guardの基本的な使い方が全て理解できます！
