//! Git operations (finding repo root, restoring files, snapshots)

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::git::snapshot::GitSnapshot;

pub struct GitOperations {
    repo_root: PathBuf,
}

impl GitOperations {
    /// Initialize `GitOperations` and resolve the repository root.
    pub fn new() -> Result<Self> {
        let repo_root = Self::find_repo_root()?;
        Ok(Self { repo_root })
    }

    /// Find the Git repository root.
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

    /// Restore the given file to the state in HEAD.
    pub fn restore_file(&self, path: &Path) -> Result<()> {
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

    /// Capture a snapshot of the current working tree state.
    pub fn create_snapshot(&self) -> Result<GitSnapshot> {
        let branch = self.get_current_branch()?;
        let head_hash = self.get_head_hash()?;
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
            .output()
            .context("Failed to execute git rev-parse for branch")?;

        Ok(String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string())
    }

    fn get_head_hash(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to execute git rev-parse for HEAD")?;

        Ok(String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string())
    }

    fn has_uncommitted_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to execute git status")?;

        Ok(!output.stdout.is_empty())
    }
}
