//! Session summary reporting

use crate::config::Violation;
use colored::Colorize;

pub struct Reporter {
    violations: Vec<Violation>,
    start_time: std::time::Instant,
}

impl Default for Reporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    pub fn print_summary(&self) {
        let duration = self.start_time.elapsed();

        println!("\n{}", "=".repeat(60).bold());
        println!("{}", "Guard Session Summary".bold().cyan());
        println!("{}", "=".repeat(60).bold());

        println!("Duration: {:.2}s", duration.as_secs_f64());
        println!("Violations detected: {}", self.violations.len());

        if self.violations.is_empty() {
            println!("{}", "✅ No violations detected!".green().bold());
        } else {
            println!("{}", "⚠️  Violations occurred:".yellow().bold());
            for (i, v) in self.violations.iter().enumerate() {
                println!("\n{}. {}", i + 1, v.format_message());
            }
        }

        println!("{}", "=".repeat(60).bold());
    }
}
