//! guard - ファイル保護ツール
//!
//! コーディングエージェントが指定されたファイルの特定行範囲を
//! 変更できないようにするRust製CLIツール

pub mod config;
pub mod watcher;
pub mod detector;
pub mod git;
pub mod agent;
pub mod output;
pub mod error;

// Re-exports for convenience
pub use error::{GuardError, Result};
