//! 保護範囲への変更検出ロジック

use crate::config::{GuardConfig, GuardRule, Violation};
use crate::config::types::LineRange;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ViolationDetector {
    config: GuardConfig,
}

impl ViolationDetector {
    pub fn new(config: GuardConfig) -> Self {
        Self { config }
    }

    /// ファイルが保護範囲を侵害しているかチェック
    pub fn check_file(&self, path: &PathBuf) -> Result<Option<Violation>> {
        let rule = match self.find_rule(path) {
            Some(r) => r,
            None => return Ok(None),
        };

        let current_content = std::fs::read_to_string(path)?;
        let current_lines: Vec<&str> = current_content.lines().collect();

        let original_content = self.get_original_content(path)?;
        let original_lines: Vec<&str> = original_content.lines().collect();

        let modified_lines =
            self.find_modified_lines(&original_lines, &current_lines, &rule.ranges);

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
            &g.path == path || g.path.canonicalize().ok() == path.canonicalize().ok()
        })
    }

    fn get_original_content(&self, path: &Path) -> Result<String> {
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
        ranges: &[LineRange],
    ) -> Vec<usize> {
        let mut modified = Vec::new();
        let max_len = original.len().max(current.len());

        for line_num in 1..=max_len {
            let idx = line_num - 1;
            let orig_line = original.get(idx).copied().unwrap_or("");
            let curr_line = current.get(idx).copied().unwrap_or("");

            if orig_line != curr_line && ranges.iter().any(|r| r.contains(line_num)) {
                modified.push(line_num);
            }
        }

        modified
    }
}
