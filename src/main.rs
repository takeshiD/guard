//! guard - protect files from coding agent modifications

use clap::{Parser, Subcommand};
use guard::agent::{AgentRunner, AgentType};
use guard::config::{ConfigParser, GuardConfig};
use guard::git::GitOperations;
use guard::output::Reporter;
use guard::watcher::{EventHandler, FileWatcher};
use std::path::PathBuf;
use std::time::Duration;

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
    #[command(name = "claude")]
    ClaudeCode {
        /// Arguments to pass to claude-code
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Run gemini-cli with guard protection
    #[command(name = "gemini")]
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

fn cmd_init() -> anyhow::Result<()> {
    println!("Initializing guard configuration...");
    let config_path = PathBuf::from(".guard.toml");
    if config_path.exists() {
        println!(".guard.toml already exists");
        return Ok(());
    }
    ConfigParser::init(&config_path)?;
    Ok(())
}

fn cmd_run_agent(agent_type: AgentType, args: Vec<String>) -> anyhow::Result<()> {
    let config_path = PathBuf::from(".guard.toml");
    let config = ConfigParser::load(&config_path)?;

    // Gitスナップショット作成（現状は情報利用のみ）
    let git = GitOperations::new()?;
    let snapshot = git.create_snapshot()?;
    println!(
        "Git snapshot: branch {}, head {}, uncommitted: {}",
        snapshot.branch, snapshot.head_hash, snapshot.has_uncommitted
    );

    // 監視対象ディレクトリ（保護対象ファイルの親ディレクトリを集約）
    let watch_dirs = collect_watch_dirs(&config);
    println!("Watching {} protected file(s)", config.guards.len());

    let watcher = FileWatcher::new(watch_dirs)?;
    let mut handler = EventHandler::new(config.clone())?;
    let reporter = Reporter::new();

    // エージェント起動
    let runner = AgentRunner::new(agent_type);
    let mut child = runner.spawn(args)?;

    // イベントループ
    loop {
        // プロセス終了チェック（簡易ポーリング）
        if let Some(status) = child.try_wait()? {
            let code = status.code().unwrap_or(1);
            if code == 0 {
                println!("✅ Agent exited successfully");
            } else {
                println!("❌ Agent exited with code {}", code);
            }
            break;
        }
        if let Some(event) = watcher.recv_timeout(Duration::from_millis(200)) {
            handler.handle_event(event)?;
        }
    }

    reporter.print_summary();

    // If any violation occurred during the session, use a distinct exit code
    if handler.had_violation() {
        std::process::exit(10);
    }

    Ok(())
}

fn cmd_check(config: PathBuf) -> anyhow::Result<()> {
    println!("🔍 Checking protected ranges...");
    println!("   Config: {}", config.display());

    let cfg = ConfigParser::load(&config)?;

    // 現在は単にパースとバリデーションのみ行う
    println!(
        "✅ Config loaded: version {}, {} guard rule(s)",
        cfg.version,
        cfg.guards.len()
    );

    Ok(())
}

fn collect_watch_dirs(config: &GuardConfig) -> Vec<PathBuf> {
    use std::collections::HashSet;

    let mut dirs = HashSet::new();
    for guard in &config.guards {
        if let Some(parent) = guard.path.parent() {
            dirs.insert(parent.to_path_buf());
        }
    }

    if dirs.is_empty() {
        dirs.insert(PathBuf::from("."));
    }

    dirs.into_iter().collect()
}
