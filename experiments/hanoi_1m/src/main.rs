use std::env;
use std::fs;
use std::path::Path;
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
struct GateReport {
    #[serde(rename = "generatedAt")]
    generated_at: String,
    #[serde(rename = "paperUrl")]
    paper_url: String,
    #[serde(rename = "requiredSteps")]
    required_steps: usize,
    #[serde(rename = "observedSteps")]
    observed_steps: usize,
    #[serde(rename = "reachedStepThreshold")]
    reached_step_threshold: bool,
    #[serde(rename = "answerCorrect")]
    answer_correct: bool,
    pass: bool,
    #[serde(rename = "tracePath")]
    trace_path: Option<String>,
    #[serde(rename = "answerPath")]
    answer_path: Option<String>,
    notes: Vec<String>,
}

fn parse_args(args: &[String]) -> (Option<String>, Option<String>) {
    let mut trace = None;
    let mut answer = None;
    for i in 0..args.len() {
        let key = &args[i];
        if key == "--trace" && i + 1 < args.len() {
            trace = Some(args[i+1].clone());
        } else if key == "--answer" && i + 1 < args.len() {
            answer = Some(args[i+1].clone());
        }
    }
    (trace, answer)
}

fn read_step_count(trace_path: &str) -> usize {
    if let Ok(content) = fs::read_to_string(trace_path) {
        content.lines().filter(|l| !l.trim().is_empty()).count()
    } else {
        0
    }
}

fn read_answer_correct(answer_path: &str) -> bool {
    if let Ok(content) = fs::read_to_string(answer_path) {
        let raw = content.trim();
        if raw.is_empty() { return false; }
        
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) {
            if let Some(b) = parsed.get("final_answer_correct").and_then(|v| v.as_bool()) {
                return b;
            }
            if let Some(b) = parsed.get("pass").and_then(|v| v.as_bool()) {
                return b;
            }
            if let Some(s) = parsed.get("verdict").and_then(|v| v.as_str()) {
                let norm = s.trim().to_uppercase();
                if norm == "PASS" || norm == "CORRECT" || norm == "SUCCESS" {
                    return true;
                }
            }
        }
        
        let norm = raw.to_uppercase();
        if norm.contains("FINAL_ANSWER_CORRECT=TRUE") { return true; }
        if norm == "PASS" || norm == "CORRECT" || norm == "SUCCESS" { return true; }
    }
    false
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (arg_trace, arg_answer) = parse_args(&args);
    
    let required_steps = 1_000_000;
    let paper_url = "https://arxiv.org/html/2511.09030v1".to_string();
    let mut notes = vec![];
    
    let trace_path = arg_trace.or_else(|| env::var("TURINGOS_MAKER_TRACE").ok()).unwrap_or_default();
    let answer_path = arg_answer.or_else(|| env::var("TURINGOS_MAKER_ANSWER").ok()).unwrap_or_default();
    
    let mut observed_steps = 0;
    if !trace_path.is_empty() && Path::new(&trace_path).exists() {
        observed_steps = read_step_count(&trace_path);
    } else {
        notes.push("Trace file missing. Provide --trace or TURINGOS_MAKER_TRACE.".to_string());
    }
    
    let mut answer_correct = false;
    if !answer_path.is_empty() && Path::new(&answer_path).exists() {
        answer_correct = read_answer_correct(&answer_path);
    } else {
        notes.push("Answer verdict file missing. Provide --answer or TURINGOS_MAKER_ANSWER.".to_string());
    }
    
    let reached_step_threshold = observed_steps >= required_steps;
    let pass = reached_step_threshold && answer_correct;
    
    if !reached_step_threshold {
        notes.push(format!("Step threshold not met: observed={}, required={}. Continue hardening and rerun.", observed_steps, required_steps));
    }
    if !answer_correct {
        notes.push("Final answer correctness proof missing or failed.".to_string());
    }
    
    let report = GateReport {
        generated_at: Utc::now().to_rfc3339(),
        paper_url,
        required_steps,
        observed_steps,
        reached_step_threshold,
        answer_correct,
        pass,
        trace_path: if trace_path.is_empty() { None } else { Some(trace_path.clone()) },
        answer_path: if answer_path.is_empty() { None } else { Some(answer_path.clone()) },
        notes,
    };
    
    let stamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let out_dir = Path::new("benchmarks/audits/final_gate");
    fs::create_dir_all(out_dir).unwrap();
    
    let stamped_path = out_dir.join(format!("maker_1m_steps_gate_{}.json", stamp));
    let latest_path = out_dir.join("maker_1m_steps_gate_latest.json");
    
    let json_str = serde_json::to_string_pretty(&report).unwrap();
    fs::write(&stamped_path, format!("{}\n", json_str)).unwrap();
    fs::write(&latest_path, format!("{}\n", json_str)).unwrap();
    
    println!("[maker-1m-gate] required_steps={}", required_steps);
    println!("[maker-1m-gate] observed_steps={}", observed_steps);
    println!("[maker-1m-gate] answer_correct={}", if answer_correct { "true" } else { "false" });
    println!("[maker-1m-gate] report={}", stamped_path.display());
    println!("[maker-1m-gate] {}", if pass { "PASS" } else { "FAIL" });
    
    if !pass {
        std::process::exit(1);
    }
}