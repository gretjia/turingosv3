/// TuringOS Librarian — Memory Compression Engine (Engine 4)
///
/// Tape = raw experience (append-only, never delete)
/// Memory = compressed wisdom (extracted from tape → learned.md)
/// Librarian = the compression pipeline (periodic + on_halt)
///
/// Architecture:
///   on_post_append → accumulate observations (node content, author)
///   record_rejection → accumulate failure signals
///   compress(tape) → classify success/failure by price, extract patterns, write learned.md
///   on_halt → final compression with golden path
///
/// Architect directive 2026-04-02:
///   "记忆库是压缩出来的。成功和失败都要log，而且分开log。"

use crate::sdk::tool::TuringTool;
use crate::kernel::{Tape, File as TapeNode};
use log::info;
use std::collections::HashMap;

pub struct LibrarianTool {
    skills_dir: String,
    swarm_size: usize,
    compress_interval: usize,
    // Internal counters
    append_count: usize,
    last_compressed_at: usize,
    // Raw observation buffers (accumulated between compressions)
    rejection_log: Vec<(String, String)>,  // (agent_id, reason)
}

impl LibrarianTool {
    pub fn new(skills_dir: &str, swarm_size: usize, compress_interval: usize) -> Self {
        info!(">>> [LIBRARIAN] Mounted. Skills: {} | Compress every {} appends", skills_dir, compress_interval);
        Self {
            skills_dir: skills_dir.to_string(),
            swarm_size,
            compress_interval,
            append_count: 0,
            last_compressed_at: 0,
            rejection_log: Vec::new(),
        }
    }

    /// Called by evaluator when a node is rejected (agents learn from others' mistakes)
    pub fn record_rejection(&mut self, agent: &str, reason: &str) {
        self.rejection_log.push((agent.to_string(), reason.to_string()));
    }

    /// Should the evaluator trigger compression now?
    pub fn should_compress(&self) -> bool {
        self.compress_interval > 0
            && self.append_count > 0
            && self.append_count - self.last_compressed_at >= self.compress_interval
    }

    /// Core compression: read tape, classify nodes, write memory to learned.md
    pub fn compress(&mut self, tape: &Tape) {
        // ── 1. Classify nodes by market price ──
        let mut success_nodes: Vec<(&str, &str, f64)> = Vec::new(); // (id, payload, price)
        let mut failure_nodes: Vec<(&str, &str, f64)> = Vec::new();

        for (nid, node) in &tape.files {
            if node.price >= 0.7 {
                success_nodes.push((nid, &node.payload, node.price));
            } else if node.price <= 0.3 && !node.payload.is_empty() {
                failure_nodes.push((nid, &node.payload, node.price));
            }
        }

        // Sort by price (best first for success, worst first for failure)
        success_nodes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        failure_nodes.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        // ── 2. Aggregate rejection reasons ──
        let mut rejection_counts: HashMap<String, usize> = HashMap::new();
        for (_, reason) in &self.rejection_log {
            let key: String = reason.chars().take(60).collect();
            *rejection_counts.entry(key).or_insert(0) += 1;
        }
        let mut top_rejections: Vec<_> = rejection_counts.into_iter().collect();
        top_rejections.sort_by(|a, b| b.1.cmp(&a.1));

        // ── 3. Find deepest chains ──
        let mut max_depth = 0;
        let mut deepest_leaf = String::new();
        for (nid, _) in &tape.files {
            let d = chain_depth(nid, tape);
            if d > max_depth {
                max_depth = d;
                deepest_leaf = nid.clone();
            }
        }

        // ── 4. Build compressed memory ──
        let mut memory = String::new();
        memory.push_str("\n# LIBRARIAN MEMORY (compressed from tape)\n\n");

        // Success patterns
        memory.push_str("## WHAT WORKS (high-price nodes, endorsed by market):\n");
        for (nid, payload, price) in success_nodes.iter().take(7) {
            let preview: String = payload.chars().take(120).collect();
            memory.push_str(&format!("- [P={:.0}%] {}\n", price * 100.0, preview.replace('\n', " ")));
        }
        if success_nodes.is_empty() {
            memory.push_str("- (no high-price nodes yet — keep building)\n");
        }

        // Failure patterns (via negativa)
        memory.push_str("\n## WHAT FAILS (low-price / shorted nodes — avoid these):\n");
        for (nid, payload, price) in failure_nodes.iter().take(5) {
            let preview: String = payload.chars().take(120).collect();
            memory.push_str(&format!("- [P={:.0}%] {}\n", price * 100.0, preview.replace('\n', " ")));
        }
        if failure_nodes.is_empty() {
            memory.push_str("- (no shorted nodes yet)\n");
        }

        // Common errors
        memory.push_str("\n## COMMON ERRORS (from rejection log — don't repeat these):\n");
        for (reason, count) in top_rejections.iter().take(5) {
            memory.push_str(&format!("- {}x rejected: {}\n", count, reason));
        }
        if top_rejections.is_empty() {
            memory.push_str("- (no rejections recorded)\n");
        }

        // Depth info
        memory.push_str(&format!("\n## PROOF DEPTH: deepest chain = {} steps (leaf: {})\n",
            max_depth, deepest_leaf));

        // ── 5. Write to ALL agents' learned.md ──
        let mut updated = 0;
        for i in 0..self.swarm_size {
            let path = format!("{}/agent_{}/learned.md", self.skills_dir, i);
            match std::fs::read_to_string(&path) {
                Ok(existing) => {
                    let base = if let Some(idx) = existing.find("\n# LIBRARIAN MEMORY") {
                        &existing[..idx]  // strip old memory, keep role preamble
                    } else {
                        &existing
                    };
                    let _ = std::fs::write(&path, format!("{}{}", base, memory));
                    updated += 1;
                }
                Err(_) => {
                    // No learned.md yet — create with just memory
                    let _ = std::fs::create_dir_all(format!("{}/agent_{}", self.skills_dir, i));
                    let _ = std::fs::write(&path, &memory);
                    updated += 1;
                }
            }
        }

        self.last_compressed_at = self.append_count;

        info!(">>> [LIBRARIAN] Compressed: {} success + {} failure nodes, {} rejections, depth={} → {}/{} agents updated",
            success_nodes.len(), failure_nodes.len(), self.rejection_log.len(), max_depth, updated, self.swarm_size);

        // Clear rejection buffer (node observations persist in tape)
        self.rejection_log.clear();
    }
}

/// Trace chain depth from a node back to root
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

    fn on_post_append(&mut self, _author: &str, _node: &TapeNode) {
        self.append_count += 1;
    }

    fn on_halt(&mut self, _golden_path: &[String], tape: &mut Tape) {
        info!(">>> [LIBRARIAN] Final compression on halt ({} appends total)", self.append_count);
        self.compress(tape);
    }
}
