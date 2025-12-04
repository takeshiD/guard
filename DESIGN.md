# guard - コーディングエージェント向けファイル保護ツール 詳細設計書

## 1. プロジェクト概要

### 1.1 目的
Claude Code、OpenAI Codex、Gemini CLI等のコーディングエージェントが、指定されたファイルの特定行範囲を変更できないようにするRust製CLIツール。

### 1.2 主要機能
- ファイルの行単位での変更保護設定
- ファイルシステム監視によるリアルタイム保護
- 保護範囲違反時の自動ロールバック
- 複数のコーディングエージェントへの対応
- LSP連携による保護範囲の可視化（Phase 2）

### 1.3 技術スタック
- 言語: Rust (edition 2021)
- 主要クレート:
  - `notify` (6.x): ファイルシステム監視
  - `serde` + `toml`: 設定ファイルの読み書き
  - `clap` (4.x): CLIインターフェース
  - `git2`: Git操作
  - `anyhow`: エラーハンドリング
  - `colored`: ターミナル出力の色付け

---

## 2. システムアーキテクチャ

### 2.1 全体構成

```
┌─────────────────────────────────────────────┐
│         guard CLI (main process)            │
│                                             │
│  ┌─────────────┐      ┌─────────────────┐ │
│  │ Config      │      │  File Watcher   │ │
│  │ Loader      │◄─────┤  (notify crate) │ │
│  └─────────────┘      └─────────────────┘ │
│         │                      │           │
│         │                      ▼           │
│         │              ┌──────────────┐   │
│         └─────────────►│ Violation    │   │
│                        │ Detector     │   │
│                        └──────────────┘   │
│                               │            │
│                               ▼            │
│                        ┌──────────────┐   │
│                        │ Git Rollback │   │
│                        │ Engine       │   │
│                        └──────────────┘   │
└─────────────────────────────────────────────┘
                    │
                    ▼
         ┌──────────────────────┐
         │  Coding Agent        │
         │  (codex/claude/etc)  │
         └──────────────────────┘
```

### 2.2 処理フロー

```
1. guard codex 実行
   ↓
2. .guard.toml 読み込み
   ↓
3. Git状態のスナップショット保存
   ↓
4. ファイル監視開始 (notify)
   ↓
5. エージェント起動 (subprocess)
   ↓
6. ファイル変更イベント検知
   ↓
7. 保護範囲チェック
   ↓
8. 違反があれば:
   - Git rollback実行
   - エラーメッセージ表示
   - ログ記録
   ↓
9. エージェント終了まで監視継続
   ↓
10. 最終レポート表示
```

---

## 3. ディレクトリ構成

```
guard/
├── Cargo.toml
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   ├── main.rs              # エントリーポイント、CLIパース
│   ├── lib.rs               # ライブラリルート
│   ├── config/
│   │   ├── mod.rs           # 設定関連のモジュール再エクスポート
│   │   ├── parser.rs        # .guard.toml パーサー
│   │   └── types.rs         # GuardConfig等の型定義
│   ├── watcher/
│   │   ├── mod.rs
│   │   ├── file_watcher.rs  # notify統合
│   │   └── event_handler.rs # ファイルイベント処理
│   ├── detector/
│   │   ├── mod.rs
│   │   ├── violation.rs     # 違反検出ロジック
│   │   └── diff_parser.rs   # ファイル差分解析
│   ├── git/
│   │   ├── mod.rs
│   │   ├── operations.rs    # Git操作 (restore, diff)
│   │   └── snapshot.rs      # Git状態のスナップショット
│   ├── agent/
│   │   ├── mod.rs
│   │   ├── runner.rs        # エージェント起動・管理
│   │   └── types.rs         # AgentType enum等
│   ├── output/
│   │   ├── mod.rs
│   │   ├── logger.rs        # ログ出力
│   │   └── reporter.rs      # 最終レポート生成
│   └── error.rs             # エラー型定義
├── tests/
│   ├── integration/
│   │   ├── init_test.rs
│   │   ├── codex_test.rs
│   │   └── rollback_test.rs
│   └── fixtures/
│       ├── sample.guard.toml
│       └── test_repo/
└── examples/
    └── basic_usage.rs
```

---

## 4. データ構造設計

### 4.1 .guard.toml フォーマット

```toml
# .guard.toml
version = "1.0"

[[guards]]
path = "src/main.rs"
reason = "Critical authentication logic"
ranges = [
  { start = 10, end = 100 },
  { start = 120, end = 250 },
]

[[guards]]
path = "src/lib.rs"
reason = "Public API contract"
ranges = [
  { start = 1, end = 10 },
  { start = 34, end = 35 },
]

# オプション: 全体設定
[settings]
auto_rollback = true          # 自動ロールバック (デフォルト: true)
log_file = ".guard/guard.log" # ログファイルパス
strict_mode = false           # 厳格モード: 違反時にエージェントを強制終了
```

### 4.2 Rustデータ構造

```rust
// src/config/types.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// .guard.toml のルート構造
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuardConfig {
    pub version: String,
    pub guards: Vec<GuardRule>,
    #[serde(default)]
    pub settings: GuardSettings,
}

/// 個別のガードルール
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuardRule {
    pub path: PathBuf,
    pub reason: Option<String>,
    pub ranges: Vec<LineRange>,
}

/// 行範囲 (1-indexed)
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    /// 指定行が範囲内かチェック
    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }

    /// 範囲が重複しているかチェック
    pub fn overlaps(&self, other: &LineRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

/// 全体設定
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuardSettings {
    #[serde(default = "default_auto_rollback")]
    pub auto_rollback: bool,
    #[serde(default = "default_log_file")]
    pub log_file: PathBuf,
    #[serde(default)]
    pub strict_mode: bool,
}

impl Default for GuardSettings {
    fn default() -> Self {
        Self {
            auto_rollback: true,
            log_file: PathBuf::from(".guard/guard.log"),
            strict_mode: false,
        }
    }
}

fn default_auto_rollback() -> bool { true }
fn default_log_file() -> PathBuf { PathBuf::from(".guard/guard.log") }

/// 違反情報
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule: GuardRule,
    pub modified_lines: Vec<usize>,
    pub timestamp: std::time::SystemTime,
}

impl Violation {
    pub fn format_message(&self) -> String {
        let path = self.rule.path.display();
        let lines = self.modified_lines
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        
        let reason = self.rule.reason
            .as_deref()
            .unwrap_or("No reason specified");
        
        format!(
            "Protected range modified in {}\n  Lines: {}\n  Reason: {}",
            path, lines, reason
        )
    }
}
```

---

## 5. 各モジュールの詳細仕様

### 5.1 config モジュール

**責務**: .guard.toml の読み書き、バリデーション

```rust
// src/config/parser.rs

use anyhow::{Context, Result};
use std::path::Path;

pub struct ConfigParser;

impl ConfigParser {
    /// .guard.toml を読み込む
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GuardConfig> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read .guard.toml")?;
        
        let config: GuardConfig = toml::from_str(&content)
            .context("Failed to parse .guard.toml")?;
        
        Self::validate(&config)?;
        Ok(config)
    }

    /// 設定をバリデーション
    fn validate(config: &GuardConfig) -> Result<()> {
        // バージョンチェック
        if config.version != "1.0" {
            anyhow::bail!("Unsupported config version: {}", config.version);
        }

        // 各ルールをバリデーション
        for guard in &config.guards {
            // ファイルの存在確認
            if !guard.path.exists() {
                anyhow::bail!("Protected file does not exist: {:?}", guard.path);
            }

            // 行範囲の妥当性チェック
            for range in &guard.ranges {
                if range.start == 0 || range.end == 0 {
                    anyhow::bail!("Line numbers must be 1-indexed");
                }
                if range.start > range.end {
                    anyhow::bail!("Invalid range: start > end");
                }
            }

            // 範囲の重複チェック
            for (i, r1) in guard.ranges.iter().enumerate() {
                for r2 in guard.ranges.iter().skip(i + 1) {
                    if r1.overlaps(r2) {
                        anyhow::bail!(
                            "Overlapping ranges in {}: {:?} and {:?}",
                            guard.path.display(),
                            r1,
                            r2
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// 初期設定ファイルを生成
    pub fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        let default_config = GuardConfig {
            version: "1.0".to_string(),
            guards: vec![],
            settings: GuardSettings::default(),
        };

        let toml = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        std::fs::write(path.as_ref(), toml)
            .context("Failed to write .guard.toml")?;

        println!("✅ Created .guard.toml");
        Ok(())
    }
}
```

### 5.2 watcher モジュール

**責務**: ファイルシステム監視、イベント検知

```rust
// src/watcher/file_watcher.rs

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use anyhow::Result;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<Event>,
}

impl FileWatcher {
    /// 新しいFileWatcherを作成
    pub fn new(watch_dirs: Vec<PathBuf>) -> Result<Self> {
        let (tx, rx): (Sender<Event>, Receiver<Event>) = channel();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            match res {
                Ok(event) => {
                    // Modifyイベントのみ処理
                    if matches!(event.kind, notify::EventKind::Modify(_)) {
                        let _ = tx.send(event);
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })?;

        // 監視対象ディレクトリを追加
        for dir in watch_dirs {
            watcher.watch(&dir, RecursiveMode::Recursive)?;
        }

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    /// イベントを待機（非ブロッキング）
    pub fn try_recv(&self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }

    /// イベントを待機（ブロッキング、タイムアウト付き）
    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Option<Event> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

// src/watcher/event_handler.rs

use crate::config::GuardConfig;
use crate::detector::ViolationDetector;
use crate::git::GitOperations;
use crate::output::Logger;
use notify::Event;
use std::path::PathBuf;

pub struct EventHandler {
    config: GuardConfig,
    detector: ViolationDetector,
    git: GitOperations,
    logger: Logger,
}

impl EventHandler {
    pub fn new(config: GuardConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            detector: ViolationDetector::new(config.clone()),
            git: GitOperations::new()?,
            logger: Logger::new(&config.settings.log_file)?,
        })
    }

    /// ファイルイベントを処理
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        for path in event.paths {
            if let Err(e) = self.handle_file_change(&path) {
                eprintln!("Error handling file change: {}", e);
                self.logger.log_error(&path, &e)?;
            }
        }
        Ok(())
    }

    fn handle_file_change(&mut self, path: &PathBuf) -> Result<()> {
        // .guard.toml自体の変更は無視
        if path.ends_with(".guard.toml") {
            return Ok(());
        }

        // 一時ファイル、swapファイルを無視
        if Self::is_temp_file(path) {
            return Ok(());
        }

        // 違反検出
        if let Some(violation) = self.detector.check_file(path)? {
            eprintln!("🛡️  GUARD VIOLATION DETECTED!");
            eprintln!("{}", violation.format_message());

            // ログに記録
            self.logger.log_violation(&violation)?;

            // 自動ロールバック
            if self.config.settings.auto_rollback {
                eprintln!("⏪ Rolling back changes...");
                self.git.restore_file(path)?;
                eprintln!("✅ File restored");
            }

            // 厳格モードならエラーで終了
            if self.config.settings.strict_mode {
                anyhow::bail!("Strict mode: Exiting due to violation");
            }
        }

        Ok(())
    }

    fn is_temp_file(path: &PathBuf) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        
        // Vim swap files
        if name.starts_with('.') && name.ends_with(".swp") {
            return true;
        }
        // Emacs autosave
        if name.starts_with('#') && name.ends_with('#') {
            return true;
        }
        // VSCode temp files
        if name.ends_with('~') {
            return true;
        }
        
        false
    }
}
```

### 5.3 detector モジュール

**責務**: 保護範囲への変更を検出

```rust
// src/detector/violation.rs

use crate::config::{GuardConfig, GuardRule, Violation};
use anyhow::Result;
use std::path::PathBuf;

pub struct ViolationDetector {
    config: GuardConfig,
}

impl ViolationDetector {
    pub fn new(config: GuardConfig) -> Self {
        Self { config }
    }

    /// ファイルが保護範囲を侵害しているかチェック
    pub fn check_file(&self, path: &PathBuf) -> Result<Option<Violation>> {
        // このファイルに適用されるガードルールを取得
        let rule = match self.find_rule(path) {
            Some(r) => r,
            None => return Ok(None), // 保護対象外
        };

        // ファイル内容を読み込み
        let current_content = std::fs::read_to_string(path)?;
        let current_lines: Vec<&str> = current_content.lines().collect();

        // Git HEADとの差分を取得
        let original_content = self.get_original_content(path)?;
        let original_lines: Vec<&str> = original_content.lines().collect();

        // 変更された行を検出
        let modified_lines = self.find_modified_lines(
            &original_lines,
            &current_lines,
            &rule.ranges,
        );

        if modified_lines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Violation {
                rule: rule.clone(),
                modified_lines,
                timestamp: std::time::SystemTime::now(),
            }))
        }
    }

    fn find_rule(&self, path: &PathBuf) -> Option<&GuardRule> {
        self.config.guards.iter().find(|g| {
            // 絶対パス、相対パスの両方に対応
            &g.path == path || g.path.canonicalize().ok() == path.canonicalize().ok()
        })
    }

    fn get_original_content(&self, path: &PathBuf) -> Result<String> {
        // git show HEAD:<path> で元のファイルを取得
        use std::process::Command;
        
        let output = Command::new("git")
            .args(["show", &format!("HEAD:{}", path.display())])
            .output()?;

        if !output.status.success() {
            // 新規ファイルの場合は空文字列
            return Ok(String::new());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn find_modified_lines(
        &self,
        original: &[&str],
        current: &[&str],
        ranges: &[crate::config::types::LineRange],
    ) -> Vec<usize> {
        let mut modified = Vec::new();

        // シンプルな行ごと比較
        let max_len = original.len().max(current.len());
        
        for line_num in 1..=max_len {
            let idx = line_num - 1; // 0-indexed

            let orig_line = original.get(idx).copied().unwrap_or("");
            let curr_line = current.get(idx).copied().unwrap_or("");

            // 行が変更されているか
            if orig_line != curr_line {
                // 保護範囲内か
                if ranges.iter().any(|r| r.contains(line_num)) {
                    modified.push(line_num);
                }
            }
        }

        modified
    }
}

// より高度な差分検出が必要な場合
// similar crate を使用した実装例

/*
use similar::{ChangeTag, TextDiff};

fn find_modified_lines_advanced(
    original: &str,
    current: &str,
    ranges: &[LineRange],
) -> Vec<usize> {
    let diff = TextDiff::from_lines(original, current);
    let mut modified = Vec::new();

    for change in diff.iter_all_changes() {
        let line_num = match change.tag() {
            ChangeTag::Delete => change.old_index().map(|i| i + 1),
            ChangeTag::Insert => change.new_index().map(|i| i + 1),
            ChangeTag::Equal => continue,
        };

        if let Some(num) = line_num {
            if ranges.iter().any(|r| r.contains(num)) {
                modified.push(num);
            }
        }
    }

    modified.sort_unstable();
    modified.dedup();
    modified
}
*/
```

### 5.4 git モジュール

**責務**: Git操作（rollback、diff取得）

```rust
// src/git/operations.rs

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub struct GitOperations {
    repo_root: PathBuf,
}

impl GitOperations {
    pub fn new() -> Result<Self> {
        let repo_root = Self::find_repo_root()?;
        Ok(Self { repo_root })
    }

    /// Gitリポジトリのルートを探す
    fn find_repo_root() -> Result<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to execute git command")?;

        if !output.status.success() {
            anyhow::bail!("Not a git repository");
        }

        let path = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();

        Ok(PathBuf::from(path))
    }

    /// ファイルをHEADの状態に戻す
    pub fn restore_file(&self, path: &PathBuf) -> Result<()> {
        let status = Command::new("git")
            .args(["restore", path.to_str().unwrap()])
            .current_dir(&self.repo_root)
            .status()
            .context("Failed to execute git restore")?;

        if !status.success() {
            anyhow::bail!("git restore failed");
        }

        Ok(())
    }

    /// 現在のワーキングツリーの状態を保存
    pub fn create_snapshot(&self) -> Result<GitSnapshot> {
        // 現在のブランチを取得
        let branch = self.get_current_branch()?;

        // HEADのコミットハッシュを取得
        let head_hash = self.get_head_hash()?;

        // 未コミットの変更があるか
        let has_uncommitted = self.has_uncommitted_changes()?;

        Ok(GitSnapshot {
            branch,
            head_hash,
            has_uncommitted,
        })
    }

    fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.repo_root)
            .output()?;

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_head_hash(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_root)
            .output()?;

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn has_uncommitted_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_root)
            .output()?;

        Ok(!output.stdout.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct GitSnapshot {
    pub branch: String,
    pub head_hash: String,
    pub has_uncommitted: bool,
}
```

### 5.5 agent モジュール

**責務**: コーディングエージェントの起動・管理

```rust
// src/agent/types.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Codex,
    ClaudeCode,
    GeminiCli,
}

impl AgentType {
    pub fn command(&self) -> &'static str {
        match self {
            AgentType::Codex => "codex",
            AgentType::ClaudeCode => "claude-code",
            AgentType::GeminiCli => "gemini-cli",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "codex" => Some(AgentType::Codex),
            "claude-code" | "claude" => Some(AgentType::ClaudeCode),
            "gemini-cli" | "gemini" => Some(AgentType::GeminiCli),
            _ => None,
        }
    }
}

// src/agent/runner.rs

use anyhow::{Context, Result};
use std::process::{Child, Command};
use colored::Colorize;

pub struct AgentRunner {
    agent_type: AgentType,
}

impl AgentRunner {
    pub fn new(agent_type: AgentType) -> Self {
        Self { agent_type }
    }

    /// エージェントを起動
    pub fn spawn(&self, args: Vec<String>) -> Result<Child> {
        let cmd = self.agent_type.command();

        println!("{} Starting {} with guard protection...", 
            "🛡️".bold(), 
            cmd.cyan().bold()
        );

        Command::new(cmd)
            .args(args)
            .spawn()
            .with_context(|| format!("Failed to start {}", cmd))
    }

    /// エージェントの終了を待つ
    pub fn wait(&self, mut child: Child) -> Result<i32> {
        let status = child.wait()
            .context("Failed to wait for agent process")?;

        let code = status.code().unwrap_or(1);
        
        if code == 0 {
            println!("{} Agent exited successfully", "✅".bold());
        } else {
            println!("{} Agent exited with code {}", "❌".bold(), code);
        }

        Ok(code)
    }
}
```

### 5.6 output モジュール

**責務**: ログ出力、レポート生成

```rust
// src/output/logger.rs

use crate::config::Violation;
use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(path: &PathBuf) -> Result<Self> {
        // ディレクトリを作成
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self { file })
    }

    pub fn log_violation(&mut self, violation: &Violation) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message = format!(
            "[{}] VIOLATION: {}\n",
            timestamp,
            violation.format_message()
        );

        self.file.write_all(message.as_bytes())?;
        self.file.flush()?;
        Ok(())
    }

    pub fn log_error(&mut self, path: &PathBuf, error: &anyhow::Error) -> Result<()> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let message = format!(
            "[{}] ERROR in {}: {}\n",
            timestamp,
            path.display(),
            error
        );

        self.file.write_all(message.as_bytes())?;
        self.file.flush()?;
        Ok(())
    }
}

// src/output/reporter.rs

use colored::Colorize;

pub struct Reporter {
    violations: Vec<Violation>,
    start_time: std::time::Instant,
}

impl Reporter {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    pub fn print_summary(&self) {
        let duration = self.start_time.elapsed();
        
        println!("\n{}", "=".repeat(60).bold());
        println!("{}", "Guard Session Summary".bold().cyan());
        println!("{}", "=".repeat(60).bold());
        
        println!("Duration: {:.2}s", duration.as_secs_f64());
        println!("Violations detected: {}", self.violations.len());

        if self.violations.is_empty() {
            println!("{}", "✅ No violations detected!".green().bold());
        } else {
            println!("{}", "⚠️  Violations occurred:".yellow().bold());
            for (i, v) in self.violations.iter().enumerate() {
                println!("\n{}. {}", i + 1, v.format_message());
            }
        }

        println!("{}", "=".repeat(60).bold());
    }
}
```

---

## 6. CLIインターフェース設計

### 6.1 コマンド仕様

```rust
// src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "guard")]
#[command(about = "Protect files from coding agent modifications", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .guard.toml in current directory
    Init,
    
    /// Run codex with guard protection
    Codex {
        /// Arguments to pass to codex
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Run claude-code with guard protection
    #[command(name = "claude-code")]
    ClaudeCode {
        /// Arguments to pass to claude-code
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Run gemini-cli with guard protection
    #[command(name = "gemini-cli")]
    GeminiCli {
        /// Arguments to pass to gemini-cli
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Check if current state violates guards
    Check {
        /// Path to guard config file
        #[arg(short, long, default_value = ".guard.toml")]
        config: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),
        Commands::Codex { args } => cmd_run_agent(AgentType::Codex, args),
        Commands::ClaudeCode { args } => cmd_run_agent(AgentType::ClaudeCode, args),
        Commands::GeminiCli { args } => cmd_run_agent(AgentType::GeminiCli, args),
        Commands::Check { config } => cmd_check(config),
    }
}
```

### 6.2 使用例

```bash
# 1. 初期化
$ guard init
✅ Created .guard.toml

# 2. .guard.toml を編集
$ vim .guard.toml

# 3. エージェントを保護付きで起動
$ guard codex
🛡️ Starting codex with guard protection...
📁 Watching 1 protected file(s)
✅ Guard active

# エージェント実行中...
🛡️ GUARD VIOLATION DETECTED!
Protected range modified in src/main.rs
  Lines: 15, 16, 17
  Reason: Critical authentication logic
⏪ Rolling back changes...
✅ File restored

# 4. 現在の状態をチェック
$ guard check
🛡️ Checking protected ranges...
✅ No violations detected
```

---

## 7. エラーハンドリング

### 7.1 エラー型定義

```rust
// src/error.rs

use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum GuardError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid line range: {start}-{end}")]
    InvalidRange { start: usize, end: usize },

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Violation in strict mode")]
    StrictModeViolation,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, GuardError>;
```

---

## 8. テスト戦略

### 8.1 ユニットテスト

```rust
// src/config/types.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range_contains() {
        let range = LineRange { start: 10, end: 20 };
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(range.contains(20));
        assert!(!range.contains(9));
        assert!(!range.contains(21));
    }

    #[test]
    fn test_line_range_overlaps() {
        let r1 = LineRange { start: 10, end: 20 };
        let r2 = LineRange { start: 15, end: 25 };
        let r3 = LineRange { start: 21, end: 30 };

        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }
}
```

### 8.2 統合テスト

```rust
// tests/integration/init_test.rs

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn test_init_creates_config_file() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("guard").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created .guard.toml"));

    let config_path = temp_dir.path().join(".guard.toml");
    assert!(config_path.exists());
}

// tests/integration/rollback_test.rs

#[test]
fn test_rollback_on_violation() {
    // 1. テストリポジトリを作成
    // 2. .guard.tomlを設定
    // 3. 保護されたファイルを変更
    // 4. 自動ロールバックを確認
    todo!("Implement rollback integration test");
}
```

---

## 9. 依存関係（Cargo.toml）

```toml
[package]
name = "guard"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Protect files from coding agent modifications"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/guard"

[dependencies]
# CLI
clap = { version = "4.5", features = ["derive", "cargo"] }
colored = "2.1"

# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# File watching
notify = "6.1"
notify-debouncer-full = "0.3" # デバウンス機能

# Git operations
git2 = "0.18" # オプション: より高度なGit操作用

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging & Time
chrono = "0.4"

# Diff (オプション: より高度な差分検出用)
similar = "2.4"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
tempfile = "3.10"

[[bin]]
name = "guard"
path = "src/main.rs"
```

---

## 10. 実装の優先順位

### Phase 1: Core MVP (1-2週間)
1. ✅ CLIスケルトン (`clap`統合)
2. ✅ `.guard.toml` パーサー
3. ✅ 基本的なファイル監視 (`notify`)
4. ✅ シンプルな違反検出（行ベース比較）
5. ✅ Git rollback機能
6. ✅ 基本的なエラーメッセージ

### Phase 2: Production Ready (1週間)
1. ✅ 包括的なエラーハンドリング
2. ✅ ログ機能
3. ✅ レポート生成
4. ✅ ユニット・統合テスト
5. ✅ ドキュメント整備
6. ✅ 一時ファイル無視ロジック

### Phase 3: Enhancement (将来)
1. LSP server実装
2. より高度な差分検出（`similar` crate）
3. 複数ブランチ対応
4. パフォーマンス最適化
5. プラグインシステム

---

## 11. 実装時の注意点

### 11.1 パフォーマンス考慮
- ファイル監視は`notify-debouncer-full`でデバウンス処理
- 大きなファイルの差分検出は部分読み込みを検討
- 頻繁な違反チェックはキャッシュ活用

### 11.2 エッジケース
- シンボリックリンク
- ハードリンク
- ファイル名変更
- ファイル移動
- Gitサブモジュール
- 空ファイル
- バイナリファイル

### 11.3 セキュリティ
- `.guard.toml`のパーミッションチェック
- パスインジェクション対策
- コマンドインジェクション対策

---

## 12. 補足資料

### 12.1 サンプル .guard.toml

```toml
version = "1.0"

[[guards]]
path = "src/auth/mod.rs"
reason = "Authentication logic is security-critical"
ranges = [
  { start = 45, end = 120 },  # verify_token function
  { start = 150, end = 180 }, # password hashing
]

[[guards]]
path = "Cargo.toml"
reason = "Dependency versions must be manually reviewed"
ranges = [
  { start = 8, end = 30 }, # [dependencies] section
]

[settings]
auto_rollback = true
log_file = ".guard/violations.log"
strict_mode = false
```

### 12.2 開発のための参考コマンド

```bash
# 開発ビルド
cargo build

# テスト実行
cargo test

# Clippy実行
cargo clippy -- -D warnings

# フォーマット
cargo fmt

# リリースビルド
cargo build --release

# インストール
cargo install --path .

# ローカルテスト
./target/release/guard init
```

---

## 13. 今後の拡張可能性

1. **LSP Server**: エディタでリアルタイム保護範囲表示
2. **Web UI**: ブラウザベースの設定管理
3. **CI/CD統合**: GitHub Actions等での自動チェック
4. **アノテーション方式**:
   ```rust
   // @guard-start: reason="Critical section"
   fn important() { }
   // @guard-end
   ```
5. **機械学習**: コード変更パターンから自動保護提案

---

## まとめ

この設計書に従えば、実用的なMVPを2週間程度で実装できます。Rust、ファイル監視、Git操作の基本的な知識があれば十分です。

**最優先実装項目**:
1. CLIパーサー
2. Config loader
3. File watcher + Event handler
4. Violation detector
5. Git rollback

これらを順番に実装すれば、動作するプロトタイプが完成します。
