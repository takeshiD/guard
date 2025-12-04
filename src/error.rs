//! エラー型定義

use thiserror::Error;
use std::path::PathBuf;

/// guardツールのエラー型
#[derive(Error, Debug)]
pub enum GuardError {
    /// 設定ファイルのエラー
    #[error("Configuration error: {0}")]
    Config(String),

    /// Git操作のエラー
    #[error("Git error: {0}")]
    Git(String),

    /// ファイルが見つからない
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// 無効な行範囲
    #[error("Invalid line range: {start}-{end}")]
    InvalidRange { start: usize, end: usize },

    /// エージェントが見つからない
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// 厳格モードでの違反
    #[error("Violation in strict mode")]
    StrictModeViolation,

    /// IO エラー
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// その他のエラー
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// guardツールの Result 型
pub type Result<T> = std::result::Result<T, GuardError>;
