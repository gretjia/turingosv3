/// TuringOS Librarian — Management Layer Agent (Engine 4)
///
/// Architect directives 2026-04-02:
///   "压缩功能作为管理层，Librarian作为管理层agent，用最好的模型"
///   "log作为ground Truth不可篡改"
///   "success/failure rejection 被 Dedup 压过了 — log没有履行ground Truth的职责"
///
/// Ground Truth Architecture:
///   success.jsonl — append-only log of all successful appends
///   failure.jsonl — append-only log of all rejected appends
///   These are NEVER cleared, NEVER truncated. They are immutable fact.
///   Memory (learned.md) is derived from logs. Memory is falsifiable theory.
///   When memory contradicts logs, logs win.

use crate::sdk::tool::TuringTool;
use crate::kernel::Tape;
use log::info;
use std::collections::HashMap;
use std::io::{BufRead, Write};

pub struct LibrarianTool {
    skills_dir: String,
    log_dir: String,
    swarm_size: usize,
    compress_interval: usize,
    compressions_done: usize,
}

impl LibrarianTool {
    pub fn new(skills_dir: &str, swarm_size: usize, compress_interval: usize, log_dir: &str) -> Self {
        let _ = std::fs::create_dir_all(log_dir);
        info!(">>> [LIBRARIAN] Mounted. Skills: {} | Logs: {} | Compress every {} appends",
            skills_dir, log_dir, compress_interval);
        Self {
            skills_dir: skills_dir.to_string(),
            log_dir: log_dir.to_string(),
            swarm_size,
            compress_interval,
            compressions_done: 0,
        }
    }

    pub fn compress_interval(&self) -> usize { self.compress_interval }
    pub fn log_dir(&self) -> &str { &self.log_dir }

    /// Append one line to a JSONL log file. Append-only, never truncate.
    pub fn log_success(&self, node_id: &str, author: &str, payload: &str, timestamp: u64) {
        let path = format!("{}/success.jsonl", self.log_dir);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let preview: String = payload.chars().take(200).collect();
            let _ = writeln!(f, r#"{{"node_id":"{}","author":"{}","payload":"{}","ts":{}}}"#,
                node_id, author, preview.replace('"', "'").replace('\n', " "), timestamp);
        }
    }

    pub fn log_failure(&self, author: &str, payload: &str, reason: &str, timestamp: u64) {
        let path = format!("{}/failure.jsonl", self.log_dir);
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
            let preview: String = payload.chars().take(100).collect();
            let reason_clean: String = reason.chars().take(120).collect();
            let _ = writeln!(f, r#"{{"author":"{}","payload":"{}","reason":"{}","ts":{}}}"#,
                author, preview.replace('"', "'").replace('\n', " "),
                reason_clean.replace('"', "'").replace('\n', " "), timestamp);
        }
    }

    /// Read ALL rejection reasons from failure.jsonl, grouped by category.
    /// Returns Vec<(category, count, sample_reason)> — ALL categories, not top N.
    fn read_rejection_log(&self) -> Vec<(String, usize, String)> {
        let path = format!("{}/failure.jsonl", self.log_dir);
        let mut category_counts: HashMap<String, (usize, String)> = HashMap::new();

        if let Ok(file) = std::fs::File::open(&path) {
            for line in std::io::BufReader::new(file).lines().flatten() {
                // Extract reason field from JSON line
                if let Some(start) = line.find(r#""reason":""#) {
                    let rest = &line[start + 10..];
                    if let Some(end) = rest.find('"') {
                        let reason = &rest[..end];
                        // Categorize by first recognizable pattern
                        let category = if reason.contains("too short") {
                            "Step too short".to_string()
                        } else if reason.contains("Duplicate in branch") {
                            "Duplicate in branch".to_string()
                        } else if reason.contains("Bankrupt") {
                            "Bankrupt agent".to_string()
                        } else if reason.contains("FRONT-RUNNING") || reason.contains("too long") {
                            "Step too long (front-running)".to_string()
                        } else if reason.contains("BLACKLIST") || reason.contains("Lean") {
                            "Blacklisted syntax".to_string()
                        } else {
                            // Use first 40 chars as category
                            reason.chars().take(40).collect()
                        };
                        let entry = category_counts.entry(category).or_insert((0, reason.to_string()));
                        entry.0 += 1;
                    }
                }
            }
        }

        let mut result: Vec<_> = category_counts.into_iter()
            .map(|(cat, (count, sample))| (cat, count, sample))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    /// Build compression prompt from tape (success) + failure.jsonl (rejections).
    /// Ground Truth: reads from persistent logs, not ephemeral memory buffers.
    pub fn build_compression_prompt(&self, tape: &Tape) -> (String, usize, usize) {
        // ── 1. Classify tape nodes by market price ──
        let mut success_nodes: Vec<(&str, &str, f64)> = Vec::new();
        let mut failure_nodes: Vec<(&str, &str, f64)> = Vec::new();

        for (nid, node) in &tape.files {
            if node.price >= 0.7 {
                success_nodes.push((nid, &node.payload, node.price));
            } else if node.price <= 0.3 && !node.payload.is_empty() {
                failure_nodes.push((nid, &node.payload, node.price));
            }
        }

        success_nodes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        failure_nodes.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        // ── 2. Read ALL rejection categories from persistent log ──
        let rejection_categories = self.read_rejection_log();

        // ── 3. Find deepest chain ──
        let mut max_depth = 0;
        let mut deepest_leaf = String::new();
        for (nid, _) in &tape.files {
            let d = chain_depth(nid, tape);
            if d > max_depth {
                max_depth = d;
                deepest_leaf = nid.clone();
            }
        }

        // ── 4. Build prompt ──
        let mut prompt = String::from(
            "You are the Librarian of a mathematical proof-search system.\n\
            Below are logs from agents attempting to prove: 1+2+3+... = -1/12 (regularization).\n\
            Logs are GROUND TRUTH — do not alter, reinterpret, or contradict them.\n\n\
            Your task: Extract reusable mathematical strategies and common errors.\n\
            ALWAYS cite source node IDs (e.g., 'from tx_42_by_3') so claims are traceable.\n\n"
        );

        prompt.push_str("## SUCCESS LOG (market-endorsed, P >= 70%):\n");
        for (nid, payload, price) in success_nodes.iter().take(10) {
            let preview: String = payload.chars().take(200).collect();
            prompt.push_str(&format!("[{}] P={:.0}%: {}\n", nid, price * 100.0, preview.replace('\n', " ")));
        }
        if success_nodes.is_empty() {
            prompt.push_str("(no high-price nodes yet)\n");
        }

        prompt.push_str("\n## FAILURE LOG (market-rejected, P <= 30%):\n");
        for (nid, payload, price) in failure_nodes.iter().take(10) {
            let preview: String = payload.chars().take(200).collect();
            prompt.push_str(&format!("[{}] P={:.0}%: {}\n", nid, price * 100.0, preview.replace('\n', " ")));
        }
        if failure_nodes.is_empty() {
            prompt.push_str("(no shorted nodes yet)\n");
        }

        // ALL rejection categories — not top 5, ALL of them
        prompt.push_str("\n## REJECTION LOG (system-rejected, from failure.jsonl — COMPLETE, ALL categories):\n");
        let total_rejections: usize = rejection_categories.iter().map(|(_, c, _)| c).sum();
        for (category, count, _sample) in &rejection_categories {
            let pct = if total_rejections > 0 { *count * 100 / total_rejections } else { 0 };
            prompt.push_str(&format!("- {}x ({}%): {}\n", count, pct, category));
        }
        if rejection_categories.is_empty() {
            prompt.push_str("(no rejections recorded)\n");
        }

        // Log depth to evaluator output (Ground Truth — sweep reads this for ERS)
        info!(">>> [LIBRARIAN] STATS: {} nodes, deepest chain = {} steps (leaf: {}), {} rejections",
            tape.files.len(), max_depth, deepest_leaf, total_rejections);

        prompt.push_str(&format!("\n## STATS: {} total nodes, deepest chain = {} steps (leaf: {}), {} total rejections\n",
            tape.files.len(), max_depth, deepest_leaf, total_rejections));

        prompt.push_str("\n---\nWrite a concise memory document for agents. Structure:\n\
            1. WHAT WORKS: 3-5 strategies that led to high-price nodes (cite node IDs)\n\
            2. WHAT FAILS: 3-5 anti-patterns to avoid (cite node IDs)\n\
            3. COMMON ERRORS: ALL rejection categories with counts (don't skip any!)\n\
            4. RECOMMENDED NEXT STEP: What the proof needs next based on the deepest chain\n\
            Keep it under 500 words. Be specific, not generic.\n");

        let sc = success_nodes.len();
        let fc = failure_nodes.len();
        (prompt, sc, fc)
    }

    /// Write compressed memory to all agents' learned.md files.
    pub fn write_memory(&mut self, memory_text: &str) {
        // Strip reasoning trace
        let clean_text = if let Some(idx) = memory_text.rfind("</think>") {
            memory_text[idx + 8..].trim()
        } else {
            memory_text.trim()
        };

        let mut memory = String::new();
        memory.push_str("\n# LIBRARIAN MEMORY (compressed from tape by DeepSeek V3)\n\n");
        memory.push_str(clean_text);

        let mut updated = 0;
        for i in 0..self.swarm_size {
            let path = format!("{}/agent_{}/learned.md", self.skills_dir, i);
            match std::fs::read_to_string(&path) {
                Ok(existing) => {
                    let base = if let Some(idx) = existing.find("\n# LIBRARIAN MEMORY") {
                        &existing[..idx]
                    } else {
                        &existing
                    };
                    let _ = std::fs::write(&path, format!("{}{}", base, memory));
                    updated += 1;
                }
                Err(_) => {
                    let _ = std::fs::create_dir_all(format!("{}/agent_{}", self.skills_dir, i));
                    let _ = std::fs::write(&path, &memory);
                    updated += 1;
                }
            }
        }

        self.compressions_done += 1;
        // NO clear() — logs are persistent files, not memory buffers

        info!(">>> [LIBRARIAN] Memory written to {}/{} agents (compression #{})",
            updated, self.swarm_size, self.compressions_done);
    }

    /// Fallback: local compression without LLM
    pub fn compress_local(&mut self, tape: &Tape) {
        let mut success_nodes: Vec<(&str, &str, f64)> = Vec::new();
        let mut failure_nodes: Vec<(&str, &str, f64)> = Vec::new();

        for (nid, node) in &tape.files {
            if node.price >= 0.7 {
                success_nodes.push((nid, &node.payload, node.price));
            } else if node.price <= 0.3 && !node.payload.is_empty() {
                failure_nodes.push((nid, &node.payload, node.price));
            }
        }
        success_nodes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        failure_nodes.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        // Read from persistent log, not memory buffer
        let rejection_categories = self.read_rejection_log();

        let mut max_depth = 0;
        for (nid, _) in &tape.files {
            let d = chain_depth(nid, tape);
            if d > max_depth { max_depth = d; }
        }

        let mut memory = String::new();
        memory.push_str("## WHAT WORKS (high-price nodes, endorsed by market):\n");
        for (nid, payload, price) in success_nodes.iter().take(7) {
            let preview: String = payload.chars().take(120).collect();
            memory.push_str(&format!("- [P={:.0}% from {}] {}\n", price * 100.0, nid, preview.replace('\n', " ")));
        }
        if success_nodes.is_empty() {
            memory.push_str("- (no high-price nodes yet — keep building)\n");
        }

        memory.push_str("\n## WHAT FAILS (low-price / shorted nodes — avoid these):\n");
        for (nid, payload, price) in failure_nodes.iter().take(5) {
            let preview: String = payload.chars().take(120).collect();
            memory.push_str(&format!("- [P={:.0}% from {}] {}\n", price * 100.0, nid, preview.replace('\n', " ")));
        }
        if failure_nodes.is_empty() {
            memory.push_str("- (no shorted nodes yet)\n");
        }

        // ALL categories from persistent log
        memory.push_str("\n## COMMON ERRORS (ALL categories from failure.jsonl):\n");
        for (category, count, _) in &rejection_categories {
            memory.push_str(&format!("- {}x: {}\n", count, category));
        }

        memory.push_str(&format!("\n## PROOF DEPTH: {} steps\n", max_depth));

        self.write_memory(&memory);
        info!(">>> [LIBRARIAN] Local fallback: {} success + {} failure nodes, {} rejection categories",
            success_nodes.len(), failure_nodes.len(), rejection_categories.len());
    }
}

fn chain_depth(node_id: &str, tape: &Tape) -> usize {
    let mut depth = 0;
    let mut cur = node_id.to_string();
    while let Some(node) = tape.files.get(&cur) {
        if node.citations.is_empty() { break; }
        cur = node.citations[0].clone();
        depth += 1;
    }
    depth
}

impl TuringTool for LibrarianTool {
    fn manifest(&self) -> &'static str { "core.tool.librarian" }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    fn on_halt(&mut self, _golden_path: &[String], tape: &mut Tape) {
        info!(">>> [LIBRARIAN] Final compression on halt");
        self.compress_local(tape);
    }
}
