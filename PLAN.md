# guard - 実装計画書

## 1. 実装方針

### 1.1 全体アプローチ
- **段階的実装**: Phase 1 (Core MVP) → Phase 2 (Production Ready) → Phase 3 (Enhancement)
- **テスト駆動開発**: 各モジュールに対してユニットテストを先行実装
- **依存関係の最小化**: まずはシンプルな実装から始め、必要に応じて高度な機能を追加

### 1.2 開発環境セットアップ
1. Rustツールチェーンの確認 (edition 2021)
2. Gitリポジトリの初期化 (既存の場合はスキップ)
3. 基本的なディレクトリ構造の作成
4. Cargo.tomlの設定

---

## 2. 実装順序とマイルストーン

### Phase 1: Core MVP (目標: 基本動作の確立)

#### Milestone 1.1: プロジェクト基盤構築 ✓
**タスク:**
- [x] `Cargo.toml` の作成と依存関係の設定
- [x] ディレクトリ構造の作成
- [x] `src/lib.rs` と `src/main.rs` の骨組み作成
- [x] エラー型定義 (`src/error.rs`)

**成果物:**
- コンパイル可能な最小構成のプロジェクト
- 基本的なエラーハンドリング機構

#### Milestone 1.2: 設定ファイル機能
**タスク:**
1. `src/config/types.rs` の実装
   - `GuardConfig` 構造体
   - `GuardRule` 構造体
   - `LineRange` 構造体とそのメソッド
   - `GuardSettings` 構造体
   - `Violation` 構造体

2. `src/config/parser.rs` の実装
   - `ConfigParser::load()` - TOML読み込み
   - `ConfigParser::validate()` - バリデーション
   - `ConfigParser::init()` - 初期設定ファイル生成

3. テストの作成
   - `LineRange::contains()` のテスト
   - `LineRange::overlaps()` のテスト
   - 設定ファイルパースのテスト

**成果物:**
- `.guard.toml` の読み書き機能
- バリデーション機能
- ユニットテスト

**検証方法:**
```bash
cargo test config::
```

#### Milestone 1.3: CLIインターフェース
**タスク:**
1. `src/main.rs` の実装
   - `clap` を使用したCLI定義
   - サブコマンド定義 (init, codex, claude-code, gemini-cli, check)
   - 引数パース

2. `cmd_init()` 関数の実装
   - `.guard.toml` の初期化処理

3. 統合テストの作成
   - `tests/integration/init_test.rs`

**成果物:**
- 動作する `guard init` コマンド

**検証方法:**
```bash
cargo run -- init
cat .guard.toml
```

#### Milestone 1.4: Git操作モジュール
**タスク:**
1. `src/git/operations.rs` の実装
   - `GitOperations::new()` - リポジトリルート検索
   - `GitOperations::find_repo_root()`
   - `GitOperations::restore_file()` - ファイルのロールバック
   - `GitOperations::create_snapshot()` - Git状態スナップショット

2. `src/git/snapshot.rs` の実装
   - `GitSnapshot` 構造体

3. ユニットテストの作成

**成果物:**
- Git操作の基本機能
- ファイルリストア機能

**検証方法:**
```bash
cargo test git::
# 手動テスト: ファイルを変更してrestoreが動作するか確認
```

#### Milestone 1.5: ファイル監視システム
**タスク:**
1. `src/watcher/file_watcher.rs` の実装
   - `FileWatcher::new()` - notify統合
   - イベント受信機能
   - デバウンス処理 (notify-debouncer-full使用)

2. `src/watcher/event_handler.rs` の実装
   - `EventHandler::new()`
   - `EventHandler::handle_event()` - イベント処理
   - `EventHandler::handle_file_change()` - ファイル変更処理
   - `EventHandler::is_temp_file()` - 一時ファイル判定

3. テストの作成

**成果物:**
- ファイルシステム監視機能
- イベントハンドリング機構

**検証方法:**
```bash
cargo test watcher::
# 手動テスト: ファイル変更が検知されるか確認
```

#### Milestone 1.6: 違反検出システム
**タスク:**
1. `src/detector/violation.rs` の実装
   - `ViolationDetector::new()`
   - `ViolationDetector::check_file()` - ファイルチェック
   - `ViolationDetector::find_rule()` - 適用ルール検索
   - `ViolationDetector::get_original_content()` - Git HEADから元ファイル取得
   - `ViolationDetector::find_modified_lines()` - 変更行検出（シンプルな行比較）

2. `Violation::format_message()` の実装

3. テストの作成

**成果物:**
- 保護範囲違反の検出機能
- 行ごとの差分比較

**検証方法:**
```bash
cargo test detector::
# 統合テスト: 保護範囲を変更して検出されるか確認
```

#### Milestone 1.7: エージェント管理
**タスク:**
1. `src/agent/types.rs` の実装
   - `AgentType` enum
   - `AgentType::command()`
   - `AgentType::from_str()`

2. `src/agent/runner.rs` の実装
   - `AgentRunner::new()`
   - `AgentRunner::spawn()` - エージェント起動
   - `AgentRunner::wait()` - エージェント終了待機

**成果物:**
- エージェント起動・管理機能

**検証方法:**
```bash
# 手動テスト: guard codex (または他のエージェント) が起動するか確認
```

#### Milestone 1.8: メイン処理の統合
**タスク:**
1. `src/main.rs` に `cmd_run_agent()` 実装
   - 設定ファイル読み込み
   - Git状態スナップショット
   - ファイル監視開始
   - エージェント起動
   - イベントループ
   - 終了処理

2. `cmd_check()` の実装
   - 現在の状態チェック

**成果物:**
- 完全に動作するCore MVP

**検証方法:**
```bash
# E2Eテスト
guard init
# .guard.tomlを編集して保護範囲を設定
guard codex
# 保護範囲を変更して自動ロールバックを確認
```

---

### Phase 2: Production Ready (目標: 実用レベルの品質)

#### Milestone 2.1: ロギング機能
**タスク:**
1. `src/output/logger.rs` の実装
   - `Logger::new()` - ログファイル作成
   - `Logger::log_violation()` - 違反ログ
   - `Logger::log_error()` - エラーログ

2. chrono依存関係の追加
3. ログディレクトリ自動作成

**成果物:**
- ログファイル出力機能

**検証方法:**
```bash
# 違反発生後、.guard/guard.log を確認
```

#### Milestone 2.2: レポート生成
**タスク:**
1. `src/output/reporter.rs` の実装
   - `Reporter::new()`
   - `Reporter::add_violation()`
   - `Reporter::print_summary()` - 最終レポート表示

2. colored依存関係を使った見やすい出力

**成果物:**
- セッション終了時のサマリー表示

#### Milestone 2.3: エラーハンドリング強化
**タスク:**
1. `src/error.rs` の拡張
   - thiserrorを使った包括的なエラー型定義
   - エラーメッセージの改善

2. 各モジュールのエラー処理見直し
3. ユーザーフレンドリーなエラーメッセージ

**成果物:**
- 堅牢なエラーハンドリング

#### Milestone 2.4: テストスイート完成
**タスク:**
1. ユニットテストの網羅率向上
   - 各モジュールで80%以上のカバレッジ目標

2. 統合テストの追加
   - `tests/integration/codex_test.rs`
   - `tests/integration/rollback_test.rs`

3. テストフィクスチャの準備
   - `tests/fixtures/sample.guard.toml`
   - `tests/fixtures/test_repo/`

**成果物:**
- 包括的なテストスイート

**検証方法:**
```bash
cargo test --all
cargo tarpaulin # カバレッジ測定
```

#### Milestone 2.5: ドキュメント整備
**タスク:**
1. `README.md` の作成
   - インストール方法
   - 使用方法
   - サンプル設定

2. `CONTRIBUTING.md` の作成
3. コード内ドキュメント (`//!` と `///`)
4. `examples/basic_usage.rs` の作成

**成果物:**
- 完全なドキュメント

#### Milestone 2.6: リファクタリングと最適化
**タスク:**
1. Clippy警告の解決
2. コードフォーマット統一
3. パフォーマンスプロファイリング
4. 不要な依存関係の削除

**成果物:**
- クリーンで保守性の高いコードベース

**検証方法:**
```bash
cargo clippy -- -D warnings
cargo fmt --check
cargo build --release
```

---

### Phase 3: Enhancement (将来の拡張)

#### Milestone 3.1: 高度な差分検出
**タスク:**
- `similar` crateを使った高度な差分アルゴリズム実装
- `src/detector/diff_parser.rs` の実装

#### Milestone 3.2: LSP Server実装
**タスク:**
- LSPプロトコル実装
- エディタプラグイン開発
- リアルタイム保護範囲表示

#### Milestone 3.3: アノテーション方式
**タスク:**
- コメントベースの保護範囲指定
- 自動`.guard.toml`生成

#### Milestone 3.4: CI/CD統合
**タスク:**
- GitHub Actionsワークフロー
- 自動チェック機能

---

## 3. 開発ワークフロー

### 3.1 日常的な開発サイクル
```bash
# 1. 機能ブランチ作成
git checkout -b feature/config-parser

# 2. 実装
vim src/config/parser.rs

# 3. テスト
cargo test
cargo clippy

# 4. コミット
git add .
git commit -m "feat: implement config parser"

# 5. マージ
git checkout main
git merge feature/config-parser
```

### 3.2 品質チェックリスト
各Milestoneの完了時に以下を確認:
- [ ] ユニットテストが全て通る
- [ ] Clippy警告が無い
- [ ] フォーマットが統一されている
- [ ] ドキュメントが更新されている
- [ ] 手動テストで動作確認済み

---

## 4. リスクと対策

### 4.1 技術的リスク

| リスク | 影響度 | 対策 |
|--------|--------|------|
| notify crateの不安定性 | 中 | 代替クレートの調査、エラーハンドリング強化 |
| Git操作の失敗 | 高 | 十分なバリデーション、ロールバック前のバックアップ |
| パフォーマンス問題 | 中 | デバウンス処理、キャッシュ活用 |
| エージェント起動失敗 | 中 | 詳細なエラーメッセージ、フォールバック処理 |

### 4.2 スコープクリープ対策
- Phase 1のMVPに集中
- 追加機能はPhase 2以降に延期
- 「将来の拡張可能性」は設計のみ、実装は後回し

---

## 5. 実装開始チェックリスト

Phase 1開始前に以下を確認:
- [ ] Rust 1.70以上がインストール済み
- [ ] Git 2.30以上がインストール済み
- [ ] エディタ/IDE環境が整っている (rust-analyzer推奨)
- [ ] DESIGN.mdを理解している
- [ ] このPLAN.mdを理解している

---

## 6. 成功基準

### Phase 1完了の定義
- [ ] `guard init` で `.guard.toml` が作成できる
- [ ] `.guard.toml` に保護範囲を設定できる
- [ ] `guard codex` (または他のエージェント) が起動する
- [ ] 保護範囲への変更が検出される
- [ ] 自動ロールバックが動作する
- [ ] 基本的なエラーメッセージが表示される
- [ ] ユニットテストが通る

### Phase 2完了の定義
- [ ] ログファイルが正しく出力される
- [ ] セッション終了時にサマリーが表示される
- [ ] 全てのエラーケースが適切に処理される
- [ ] テストカバレッジが80%以上
- [ ] READMEとドキュメントが完成
- [ ] Clippy警告がゼロ

---

## 7. 次のステップ

1. **即座に開始**: Milestone 1.1 (プロジェクト基盤構築)
2. **最初のタスク**:
   - `Cargo.toml` の作成
   - ディレクトリ構造の作成
   - `src/lib.rs` と `src/main.rs` の骨組み

3. **推奨される実装順序**:
   ```
   Milestone 1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6 → 1.7 → 1.8
   ```

---

## 8. 参考コマンド集

```bash
# プロジェクト作成
cargo new guard --bin
cd guard

# 開発ビルド
cargo build

# テスト実行
cargo test
cargo test -- --nocapture  # 標準出力表示

# 静的解析
cargo clippy -- -D warnings

# フォーマット
cargo fmt

# ドキュメント生成
cargo doc --open

# リリースビルド
cargo build --release

# インストール
cargo install --path .

# ベンチマーク (将来的に)
cargo bench

# カバレッジ (tarpaulinが必要)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

---

## 9. 実装時の注意事項

### 9.1 コーディング規約
- Rustの標準的な命名規則に従う
- `cargo fmt` でフォーマット統一
- `cargo clippy` の警告を全て解決
- 公開APIには必ずドキュメントコメント

### 9.2 エッジケースへの配慮
実装時に以下のケースを考慮:
- 空ファイル
- バイナリファイル (検出して無視)
- シンボリックリンク
- 同時に複数ファイルが変更される場合
- 保護範囲が重複している場合
- 行数が変わる変更 (挿入・削除)

### 9.3 セキュリティ考慮事項
- ファイルパスのサニタイズ (パスインジェクション対策)
- コマンドラインインジェクション対策
- `.guard.toml` のパーミッションチェック

---

## まとめ

この実装計画に従えば、以下のタイムラインで実装できます:

- **Week 1-2**: Phase 1 (Core MVP) - 基本機能完成
- **Week 3**: Phase 2 (Production Ready) - 実用レベル到達
- **Week 4+**: Phase 3 (Enhancement) - 拡張機能追加

最初のマイルストーン (1.1) から順番に実装を進めることで、段階的に機能を追加し、常に動作する状態を維持できます。
