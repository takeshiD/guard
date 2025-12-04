//! エージェントプロセスの起動・待機

use crate::agent::AgentType;
use anyhow::{Context, Result};
use colored::Colorize;
use std::process::{Child, Command};

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

        println!(
            "{} Starting {} with guard protection...",
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
        let status = child
            .wait()
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

