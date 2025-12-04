//! 設定ファイル関連のモジュール

pub mod parser;
pub mod types;

pub use parser::ConfigParser;
pub use types::{GuardConfig, GuardRule, GuardSettings, LineRange, Violation};
