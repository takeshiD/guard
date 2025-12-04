//! コーディングエージェント種別

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Codex,
    ClaudeCode,
    GeminiCli,
}

impl AgentType {
    pub fn command(&self) -> &'static str {
        match self {
            AgentType::Codex => "codex",
            AgentType::ClaudeCode => "claude-code",
            AgentType::GeminiCli => "gemini-cli",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "codex" => Some(AgentType::Codex),
            "claude-code" | "claude" => Some(AgentType::ClaudeCode),
            "gemini-cli" | "gemini" => Some(AgentType::GeminiCli),
            _ => None,
        }
    }
}
