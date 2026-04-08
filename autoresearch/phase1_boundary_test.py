#!/usr/bin/env python3
"""
Phase 1: Single-Agent OMEGA Boundary Test

Two modes:
  A) One-shot: model produces full proof in one response
  B) Iterative: step-by-step chain building (feed back each step)

Goal: find the largest model that CANNOT complete the proof alone.
"""

import json, os, sys, time, re
from pathlib import Path
from datetime import datetime

PROJECT = Path(__file__).resolve().parent.parent
RESULTS_DIR = Path(__file__).resolve().parent / "phase1_results"
RESULTS_DIR.mkdir(exist_ok=True)

PROBLEM = "证明所有自然数之和 = -1/12，想办法利用已知提示的公式 m * exp(-m/N) * cos(m/N)"
CONTEXT = "You are a reasoning agent collaborating with others on a mathematical proof. Follow all formatting instructions."

MODELS = [
    {"name": "qwen3-32b", "proxy": "http://127.0.0.1:8088", "model_id": "qwen3-32b"},
    {"name": "qwen3-14b", "proxy": "http://127.0.0.1:8088", "model_id": "qwen3-14b"},
    {"name": "Qwen2.5-72B", "proxy": "http://127.0.0.1:8089", "model_id": "Qwen/Qwen2.5-72B-Instruct"},
    {"name": "DeepSeek-V3", "proxy": "http://127.0.0.1:8089", "model_id": "deepseek-ai/DeepSeek-V3"},
]

RUNS_PER_TEST = 5
MAX_ITERATIVE_STEPS = 50
TEMPERATURE = 0.5


def call_llm(proxy_url, model_id, messages, max_tokens=3000):
    """Call LLM via proxy. Returns (content, completion_tokens, prompt_tokens)."""
    import urllib.request
    body = json.dumps({
        "model": model_id,
        "messages": messages,
        "temperature": TEMPERATURE,
        "max_tokens": max_tokens,
        "enable_thinking": False,
    }).encode()

    url = proxy_url + "/v1/chat/completions"
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})

    for attempt in range(5):
        try:
            with urllib.request.urlopen(req, timeout=180) as resp:
                result = json.loads(resp.read())
                content = result["choices"][0]["message"].get("content", "")
                usage = result.get("usage", {})
                return content, usage.get("completion_tokens", 0), usage.get("prompt_tokens", 0)
        except Exception as e:
            if "429" in str(e) and attempt < 4:
                time.sleep(2 ** attempt + 1)
            else:
                raise


def check_completed(text):
    """Check if the proof reached -1/12 as a conclusion (not just a mention)."""
    lower = text.lower()
    # Must contain -1/12 in a conclusive context
    patterns = [
        r"=\s*-?\s*1\s*/\s*12",
        r"=\s*-\s*\\frac\{1\}\{12\}",
        r"证毕", r"q\.?e\.?d", r"this completes the proof",
        r"this proves", r"we have shown",
        r"therefore.*sum.*=.*-1/12",
        r"hence.*=.*-1/12",
        r"thus.*=.*-1/12",
    ]
    for p in patterns:
        if re.search(p, lower):
            return True
    return False


def test_oneshot(model_config, run_id):
    """Test A: One-shot proof generation."""
    messages = [
        {"role": "system", "content": CONTEXT},
        {"role": "user", "content": f"""{PROBLEM}

Please prove this step by step. Write each step as a separate numbered reasoning step.
Each step should contain exactly ONE atomic mathematical reasoning step.
Do not skip steps. Show all intermediate work.
When the proof is complete, explicitly state the final result."""}
    ]

    start = time.time()
    content, comp_tokens, prompt_tokens = call_llm(
        model_config["proxy"], model_config["model_id"], messages, max_tokens=8000)
    elapsed = time.time() - start

    completed = check_completed(content)
    steps = len(re.findall(r'(?:Step|步骤|第)\s*(\d+)', content, re.IGNORECASE))
    if steps == 0:
        steps = content.count('\n\n')

    return {
        "mode": "oneshot",
        "model": model_config["name"],
        "run": run_id,
        "completed": completed,
        "steps": steps,
        "chars": len(content),
        "completion_tokens": comp_tokens,
        "prompt_tokens": prompt_tokens,
        "elapsed_s": round(elapsed),
        "response": content,
    }


def test_iterative(model_config, run_id):
    """Test B: Iterative chain building."""
    chain = []
    total_comp_tokens = 0
    total_prompt_tokens = 0
    start = time.time()

    for step_num in range(MAX_ITERATIVE_STEPS):
        chain_text = f"{PROBLEM}\n\n"
        if chain:
            chain_text += "=== CURRENT PROOF CHAIN ===\n"
            for i, step in enumerate(chain):
                chain_text += f"Step {i+1}: {step}\n"
            chain_text += "=== WRITE THE NEXT STEP ===\n"

        messages = [
            {"role": "system", "content": CONTEXT},
            {"role": "user", "content": f"""{chain_text}

Write exactly ONE mathematical reasoning step that advances this proof.
Be specific and show the mathematical work. Do not repeat previous steps.
If the proof is complete, write your final conclusion with the result.
Output ONLY the single next step, nothing else."""}
        ]

        try:
            content, comp_tok, prompt_tok = call_llm(
                model_config["proxy"], model_config["model_id"], messages, max_tokens=3000)
        except Exception as e:
            break

        total_comp_tokens += comp_tok
        total_prompt_tokens += prompt_tok

        step_text = content.strip()
        if not step_text:
            break

        # Check for repetition
        prefix = step_text[:80].lower()
        if any(prev[:80].lower() == prefix for prev in chain):
            break

        chain.append(step_text)

        # Check if proof is complete
        if check_completed(step_text):
            break

        time.sleep(0.5)

    elapsed = time.time() - start
    full_text = "\n\n".join(chain)
    completed = check_completed(full_text)

    return {
        "mode": "iterative",
        "model": model_config["name"],
        "run": run_id,
        "completed": completed,
        "steps": len(chain),
        "chars": len(full_text),
        "completion_tokens": total_comp_tokens,
        "prompt_tokens": total_prompt_tokens,
        "elapsed_s": round(elapsed),
        "chain": chain,
    }


def main():
    filter_model = None
    filter_mode = None
    for i, arg in enumerate(sys.argv[1:], 1):
        if arg == "--model" and i < len(sys.argv) - 1:
            filter_model = sys.argv[i + 1]
        if arg == "--mode" and i < len(sys.argv) - 1:
            filter_mode = sys.argv[i + 1]

    models = MODELS
    if filter_model:
        models = [m for m in MODELS if filter_model.lower() in m["name"].lower()]

    modes = ["oneshot", "iterative"]
    if filter_mode:
        modes = [filter_mode]

    ts = datetime.now().strftime("%Y%m%d_%H%M%S")
    all_results = []

    print("=" * 60)
    print("Phase 1: Single-Agent OMEGA Boundary Test")
    print(f"  Models: {[m['name'] for m in models]}")
    print(f"  Modes: {modes}")
    print(f"  Runs per test: {RUNS_PER_TEST}")
    print(f"  Total tests: {len(models) * len(modes) * RUNS_PER_TEST}")
    print("=" * 60)

    for model in models:
        for mode in modes:
            print(f"\n=== {model['name']} / {mode} ===")
            for run_id in range(1, RUNS_PER_TEST + 1):
                try:
                    if mode == "oneshot":
                        result = test_oneshot(model, run_id)
                    else:
                        result = test_iterative(model, run_id)

                    status = "COMPLETED" if result["completed"] else "INCOMPLETE"
                    print(f"  run={run_id}: {status} steps={result['steps']} "
                          f"comp_tok={result['completion_tokens']} elapsed={result['elapsed_s']}s")

                    all_results.append(result)
                except Exception as e:
                    print(f"  run={run_id}: ERROR {e}")
                    all_results.append({
                        "mode": mode, "model": model["name"], "run": run_id,
                        "completed": False, "steps": 0, "error": str(e),
                    })

                time.sleep(2)

    # Save results
    results_file = RESULTS_DIR / f"phase1_{ts}.json"
    with open(results_file, "w") as f:
        json.dump(all_results, f, indent=2, ensure_ascii=False)

    # Summary TSV
    tsv_file = RESULTS_DIR / f"phase1_{ts}.tsv"
    with open(tsv_file, "w") as f:
        f.write("model\tmode\trun\tcompleted\tsteps\tcomp_tokens\tprompt_tokens\telapsed_s\n")
        for r in all_results:
            f.write("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n".format(
                r.get("model",""), r.get("mode",""), r.get("run",""),
                r.get("completed",False), r.get("steps",0),
                r.get("completion_tokens",0), r.get("prompt_tokens",0),
                r.get("elapsed_s",0)))

    # Print summary
    print("\n" + "=" * 60)
    print("PHASE 1 SUMMARY")
    print("=" * 60)
    for model in models:
        for mode in modes:
            runs = [r for r in all_results if r["model"] == model["name"] and r["mode"] == mode]
            completed = sum(1 for r in runs if r.get("completed"))
            steps = [r.get("steps", 0) for r in runs]
            med_steps = sorted(steps)[len(steps)//2] if steps else 0
            print(f"  {model['name']:>20} / {mode:>10}: {completed}/{len(runs)} completed, "
                  f"median steps={med_steps}")

    print(f"\nResults: {results_file}")
    print(f"TSV: {tsv_file}")


if __name__ == "__main__":
    main()
