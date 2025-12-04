//! Logging of violations and errors

use crate::config::Violation;
use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(path: &PathBuf) -> Result<Self> {
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

    pub fn log_error(&mut self, path: &Path, error: &anyhow::Error) -> Result<()> {
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
