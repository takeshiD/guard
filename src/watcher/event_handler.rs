//! ファイルイベント処理

use crate::config::GuardConfig;
use crate::detector::ViolationDetector;
use crate::git::GitOperations;
use crate::output::Logger;
use anyhow::Result;
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

        if let Some(violation) = self.detector.check_file(path)? {
            eprintln!("🛡️  GUARD VIOLATION DETECTED!");
            eprintln!("{}", violation.format_message());

            self.logger.log_violation(&violation)?;

            if self.config.settings.auto_rollback {
                eprintln!("⏪ Rolling back changes...");
                self.git.restore_file(path)?;
                eprintln!("✅ File restored");
            }

            if self.config.settings.strict_mode {
                anyhow::bail!("Strict mode: Exiting due to violation");
            }
        }

        Ok(())
    }

    fn is_temp_file(path: &std::path::Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if name.starts_with('.') && name.ends_with(".swp") {
            return true;
        }
        if name.starts_with('#') && name.ends_with('#') {
            return true;
        }
        if name.ends_with('~') {
            return true;
        }

        false
    }
}
