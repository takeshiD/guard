//! Loading and validating `.guard.toml` configuration

use crate::config::types::{GuardConfig, GuardSettings};
use anyhow::{Context, Result};
use std::path::Path;

pub struct ConfigParser;

impl ConfigParser {
    /// Load `.guard.toml` from the given path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GuardConfig> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read .guard.toml")?;

        let config: GuardConfig =
            toml::from_str(&content).context("Failed to parse .guard.toml")?;

        Self::validate(&config)?;
        Ok(config)
    }

    /// Validate configuration consistency.
    fn validate(config: &GuardConfig) -> Result<()> {
        // version check
        if config.version != "1.0" {
            anyhow::bail!("Unsupported config version: {}", config.version);
        }

        // validate each rule
        for guard in &config.guards {
            // ensure file exists
            if !guard.path.exists() {
                anyhow::bail!("Protected file does not exist: {:?}", guard.path);
            }

            // validate line ranges
            for range in &guard.ranges {
                if range.start == 0 || range.end == 0 {
                    anyhow::bail!("Line numbers must be 1-indexed");
                }
                if range.start > range.end {
                    anyhow::bail!("Invalid range: start > end");
                }
            }

            // check for overlapping ranges
            for (i, r1) in guard.ranges.iter().enumerate() {
                for r2 in guard.ranges.iter().skip(i + 1) {
                    if r1.overlaps(r2) {
                        anyhow::bail!(
                            "Overlapping ranges in {}: {:?} and {:?}",
                            guard.path.display(),
                            r1,
                            r2
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate an initial configuration file.
    pub fn init<P: AsRef<Path>>(path: P) -> Result<()> {
        let default_config = GuardConfig {
            version: "1.0".to_string(),
            guards: vec![],
            settings: GuardSettings::default(),
        };

        let toml = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        std::fs::write(path.as_ref(), toml).context("Failed to write .guard.toml")?;

        println!("✅ Created .guard.toml");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_init_creates_valid_default_config() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        ConfigParser::init(&path).unwrap();

        let loaded = ConfigParser::load(&path).unwrap();
        assert_eq!(loaded.version, "1.0");
        assert!(loaded.guards.is_empty());
        assert!(loaded.settings.auto_rollback);
    }

    #[test]
    fn test_validate_rejects_invalid_version() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
version = "0.9"
guards = []
"#
        )
        .unwrap();

        let res = ConfigParser::load(file.path());
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_rejects_nonexistent_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
version = "1.0"

[[guards]]
path = "nonexistent.rs"
ranges = [{{ start = 1, end = 2 }}]
"#
        )
        .unwrap();

        let res = ConfigParser::load(file.path());
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_rejects_overlapping_ranges() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("file.rs");
        fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let mut config_file = NamedTempFile::new().unwrap();
        writeln!(
            config_file,
            r#"
version = "1.0"

[[guards]]
path = "{}"
ranges = [
  {{ start = 1, end = 2 }},
  {{ start = 2, end = 3 }},
]
"#,
            file_path.display()
        )
        .unwrap();

        let res = ConfigParser::load(config_file.path());
        assert!(res.is_err());
    }
}
