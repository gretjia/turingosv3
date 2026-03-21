/// TuringOS Free Search Tool
///
/// Universal zero-cost information retrieval for agents.
/// Aligns with Magna Carta Law 1: "Thinking and inquiry is free."
///
/// The search tool does NOT write to Tape, does NOT cost currency.
/// Agents must explicitly choose to use it (no auto-oracle).
/// This is the gene selector for Scholar-type agents.

use std::process::Command;
use std::time::Duration;

pub struct SearchTool {
    /// Directories to search in (domain-configurable)
    pub search_paths: Vec<String>,
    /// Maximum results to return
    pub max_results: usize,
    /// Timeout for search execution
    pub timeout: Duration,
}

impl SearchTool {
    pub fn new(search_paths: Vec<String>) -> Self {
        Self {
            search_paths,
            max_results: 10,
            timeout: Duration::from_secs(5),
        }
    }

    /// Execute a search query. Returns formatted results string.
    /// Input is sanitized: only alphanumeric, underscore, apostrophe, dot, space allowed.
    pub fn search(&self, raw_query: &str) -> String {
        let safe_query: String = raw_query.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '\'' || *c == '.' || *c == ' ')
            .collect();

        if safe_query.is_empty() || safe_query.len() > 100 {
            return String::new();
        }

        let mut all_results = Vec::new();

        for path in &self.search_paths {
            if !std::path::Path::new(path).exists() {
                continue;
            }

            let result = Command::new("grep")
                .args(&["-r", "-l", "--include=*.lean", &safe_query, path])
                .output();

            if let Ok(out) = result {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().take(self.max_results) {
                    all_results.push(line.to_string());
                }
            }
        }

        if all_results.is_empty() {
            return format!("[SEARCH '{}': no results]\n", safe_query);
        }

        let mut output = format!("[SEARCH '{}': {} files found]\n", safe_query, all_results.len());
        for r in all_results.iter().take(self.max_results) {
            output.push_str(&format!("  {}\n", r));
        }
        output
    }
}
