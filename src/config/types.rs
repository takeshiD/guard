//! .guard.toml 用の型定義

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

fn default_auto_rollback() -> bool {
    true
}

fn default_log_file() -> PathBuf {
    PathBuf::from(".guard/guard.log")
}

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
        let lines = self
            .modified_lines
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let reason = self
            .rule
            .reason
            .as_deref()
            .unwrap_or("No reason specified");

        format!(
            "Protected range modified in {}\n  Lines: {}\n  Reason: {}",
            path, lines, reason
        )
    }
}

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

