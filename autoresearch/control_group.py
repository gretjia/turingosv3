#!/usr/bin/env python3
"""
Control Group: Single qwen3-8b vs TuringOS Swarm (depth=23)

Fair comparison — same model, same problem, same temperature.
No TuringOS, no market, no multi-agent. Just one 8B model reasoning alone.

Two experiments:
  A) ONE-SHOT: "prove this step by step" in a single prompt
  B) ITERATIVE: feed chain back, ask for next step (simulates swarm append loop)

Uses Aliyun DashScope API (same provider as swarm researcher α).
"""

import json, os, sys, time, re
from pathlib import Path
from datetime import datetime

# ── Config ──
API_KEY = None
API_URL = "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
MODEL = "qwen3-8b"
TEMPERATURE = 0.5
MAX_TOKENS = 3072

# Same problem as swarm agents see (problem.txt)
PROBLEM = "证明所有自然数之和 = -1/12，想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)"

# Same context as swarm agents see AFTER the fix (no regularization hint)
CONTEXT = "You are a reasoning agent collaborating with others on a mathematical proof. Follow all formatting instructions."

# Same skill rules as swarm agents
SKILL = """[LAW 1] APPEND IS FREE: Creating nodes costs ZERO. Explore freely.
[LAW 2] ONLY INVEST COSTS MONEY: Invest/Short are the ONLY actions that burn coins.
[LAW 5] TWO SACRED DUTIES:
  - To BUILD: propose correct steps that advance the proof.
  - To SCRUTINIZE: catch errors before others build on sand.
[LAW 6] ONE STEP PER SUBMISSION:
  - Write exactly ONE mathematical reasoning step per append.
  - NO multi-step proofs. NO bundling."""

OUTPUT_DIR = Path(__file__).resolve().parent / "control_group_results"


def load_api_key():
    global API_KEY
    env_file = Path(__file__).resolve().parent.parent / ".env"
    for line in env_file.read_text().splitlines():
        line = line.strip()
        if line.startswith("DASHSCOPE_API_KEY="):
            API_KEY = line.split("=", 1)[1]
            return
    print("ERROR: DASHSCOPE_API_KEY not found in .env", file=sys.stderr)
    sys.exit(1)


def call_qwen(messages, enable_thinking=False):
    """Call qwen3-8b via DashScope (same API path as swarm)."""
    import urllib.request
    body = json.dumps({
        "model": MODEL,
        "messages": messages,
        "temperature": TEMPERATURE,
        "max_tokens": MAX_TOKENS,
        "enable_thinking": enable_thinking,
        "stream": False,
    }).encode()

    req = urllib.request.Request(API_URL, data=body, headers={
        "Content-Type": "application/json",
        "Authorization": f"Bearer {API_KEY}",
    })

    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                result = json.loads(resp.read())
                content = result["choices"][0]["message"].get("content", "")
                return content
        except Exception as e:
            if "429" in str(e) and attempt < 3:
                wait = 2 ** attempt + 1
                print(f"  [429 rate limit, waiting {wait}s...]", flush=True)
                time.sleep(wait)
            else:
                raise


def experiment_a_oneshot():
    """Experiment A: One-shot — ask for full proof in one prompt."""
    print("=" * 60)
    print("EXPERIMENT A: ONE-SHOT (single prompt, full proof)")
    print("=" * 60)

    messages = [
        {"role": "system", "content": CONTEXT},
        {"role": "user", "content": f"""{PROBLEM}

{SKILL}

Please prove this step by step. Write each step as a separate numbered reasoning step.
Each step should contain exactly ONE atomic mathematical reasoning step.
Do not skip steps. Show all intermediate work."""}
    ]

    print(f"  Model: {MODEL} | Temp: {TEMPERATURE} | Thinking: off")
    print(f"  Calling API...", flush=True)

    start = time.time()
    response = call_qwen(messages)
    elapsed = time.time() - start

    # Count steps
    steps = re.findall(r'(?:Step|步骤|第)\s*(\d+)', response, re.IGNORECASE)
    step_count = max(int(s) for s in steps) if steps else response.count('\n\n')

    print(f"  Response: {len(response)} chars, {elapsed:.1f}s")
    print(f"  Steps detected: {step_count}")
    print(f"\n--- FULL RESPONSE ---")
    print(response)
    print(f"--- END ---\n")

    return {"experiment": "A_oneshot", "model": MODEL, "steps": step_count,
            "chars": len(response), "elapsed": elapsed, "response": response}


def experiment_b_iterative():
    """Experiment B: Iterative — feed chain back, ask for next step (simulates swarm)."""
    print("=" * 60)
    print("EXPERIMENT B: ITERATIVE (chain-feeding, simulates swarm append)")
    print("=" * 60)

    chain = []
    max_steps = 30  # cap to prevent infinite loop
    consecutive_failures = 0

    for step_num in range(max_steps):
        # Build prompt exactly like the swarm's build_chain_from_snapshot
        chain_text = f"{PROBLEM}\n\n"
        if chain:
            chain_text += "=== CURRENT BEST PROOF CHAIN ===\n"
            for i, step in enumerate(chain):
                chain_text += f"Step {i+1} [Price: 50]: {step}\n"
            chain_text += "=== WRITE THE NEXT STEP ===\n"

        messages = [
            {"role": "system", "content": CONTEXT},
            {"role": "user", "content": f"""{chain_text}

{SKILL}

Write exactly ONE mathematical reasoning step that advances this proof.
Be specific and show the mathematical work. Do not repeat previous steps.
Output ONLY the single next step, nothing else."""}
        ]

        print(f"  [{step_num}] Requesting step {step_num + 1}...", flush=True)

        try:
            response = call_qwen(messages)
        except Exception as e:
            print(f"  ERROR: {e}")
            consecutive_failures += 1
            if consecutive_failures >= 3:
                print(f"  3 consecutive failures, stopping.")
                break
            time.sleep(5)
            continue

        consecutive_failures = 0

        # Clean response
        step_text = response.strip()
        if not step_text:
            print(f"  Empty response, stopping.")
            break

        # Check for repetition (first 80 chars match any previous step)
        prefix = step_text[:80].lower()
        is_repeat = any(prev[:80].lower() == prefix for prev in chain)
        if is_repeat:
            print(f"  REPEAT detected at step {step_num + 1}, stopping.")
            break

        # Check for conclusion/QED (model thinks it's done)
        if any(marker in step_text.lower() for marker in ["q.e.d", "qed", "证毕", "thus we have shown", "this completes the proof"]):
            chain.append(step_text)
            print(f"  Step {step_num + 1}: CONCLUDED ({step_text[:80]}...)")
            break

        chain.append(step_text)
        preview = step_text[:100].replace('\n', ' ')
        print(f"  Step {step_num + 1}: {preview}...")

        time.sleep(1)  # rate limit courtesy

    print(f"\n  TOTAL STEPS: {len(chain)}")
    print(f"\n--- FULL CHAIN ---")
    for i, step in enumerate(chain):
        print(f"\n[Step {i+1}]")
        print(step)
    print(f"--- END ---\n")

    return {"experiment": "B_iterative", "model": MODEL, "steps": len(chain),
            "chain": chain}


def main():
    load_api_key()
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    ts = datetime.now().strftime("%Y%m%d_%H%M%S")

    print(f"Control Group Experiment — {MODEL} vs TuringOS Swarm")
    print(f"  Swarm benchmark: depth=23, 340 nodes, 5 agents, 0 repeats")
    print(f"  Control: single {MODEL}, same problem, same temperature")
    print(f"  Timestamp: {ts}")
    print()

    # Run both experiments
    result_a = experiment_a_oneshot()
    print("\n" + "=" * 60 + "\n")
    time.sleep(5)  # cooldown between experiments
    result_b = experiment_b_iterative()

    # Summary
    print("\n" + "=" * 60)
    print("CONTROL GROUP SUMMARY")
    print("=" * 60)
    print(f"  Experiment A (one-shot):  {result_a['steps']} steps")
    print(f"  Experiment B (iterative): {result_b['steps']} steps")
    print(f"  TuringOS Swarm:           23 steps (depth=23, 340 nodes)")
    print(f"  Emergence ratio (B/Swarm): {result_b['steps']}/23 = {result_b['steps']/23:.2f}x")

    # Save results
    output_file = OUTPUT_DIR / f"control_{ts}.json"
    with open(output_file, "w") as f:
        json.dump({
            "timestamp": ts,
            "model": MODEL,
            "temperature": TEMPERATURE,
            "problem": PROBLEM,
            "context": CONTEXT,
            "swarm_benchmark": {"depth": 23, "nodes": 340, "agents": 5, "dedup": 0},
            "experiment_a": {k: v for k, v in result_a.items() if k != "response"},
            "experiment_a_response": result_a["response"],
            "experiment_b": {k: v for k, v in result_b.items() if k != "chain"},
            "experiment_b_chain": result_b["chain"],
        }, f, ensure_ascii=False, indent=2)
    print(f"\n  Results saved: {output_file}")


if __name__ == "__main__":
    main()
