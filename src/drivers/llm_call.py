#!/usr/bin/env python3
"""
LLM API wrapper — called by evaluator as subprocess.
Replaces reqwest (which hangs on Chinese HTTPS endpoints).

Usage: echo '{"prompt":"...","model":"qwen3-8b","provider":"aliyun","temperature":0.5,"max_tokens":3072}' | python3 llm_call.py
Output: raw LLM response text to stdout. Errors to stderr.
"""
import sys, json, os

def call_llm(config):
    from openai import OpenAI

    provider = config.get("provider", "aliyun")
    model = config["model"]
    prompt = config["prompt"]
    system = config.get("system", "You are a reasoning agent. Follow all formatting instructions exactly.")
    temperature = config.get("temperature", 0.5)
    max_tokens = config.get("max_tokens", 3072)
    enable_thinking = config.get("enable_thinking", False)

    PROVIDERS = {
        "aliyun": {
            "base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "key_env": "DASHSCOPE_API_KEY",
        },
        "siliconflow": {
            "base_url": "https://api.siliconflow.cn/v1",
            "key_env": "SILICONFLOW_API_KEY",
        },
        "deepseek": {
            "base_url": "https://api.deepseek.com",
            "key_env": "DEEPSEEK_API_KEY",
        },
        "volcengine": {
            "base_url": "https://ark.cn-beijing.volces.com/api/v3",
            "key_env": "VOLCENGINE_API_KEY",
        },
        "nvidia": {
            "base_url": "https://integrate.api.nvidia.com/v1",
            "key_env": "NVIDIA_NIM_API_KEY",
        },
    }

    p = PROVIDERS.get(provider)
    if not p:
        print(f"Unknown provider: {provider}", file=sys.stderr)
        sys.exit(1)

    api_key = os.environ.get(p["key_env"], "")
    if not api_key:
        print(f"Missing env: {p['key_env']}", file=sys.stderr)
        sys.exit(1)

    client = OpenAI(api_key=api_key, base_url=p["base_url"])

    extra = {}
    if enable_thinking:
        extra["extra_body"] = {"enable_thinking": True}
    elif "qwen3" in model.lower():
        extra["extra_body"] = {"enable_thinking": False}

    try:
        resp = client.chat.completions.create(
            model=model,
            messages=[
                {"role": "system", "content": system},
                {"role": "user", "content": prompt},
            ],
            temperature=temperature,
            max_tokens=max_tokens,
            stream=True,
            **extra,
        )

        content = ""
        for chunk in resp:
            delta = chunk.choices[0].delta
            if hasattr(delta, "reasoning_content") and delta.reasoning_content:
                content += delta.reasoning_content
            if hasattr(delta, "content") and delta.content:
                content += delta.content

        print(content, end="")
    except Exception as e:
        print(f"LLM API error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    if len(sys.argv) > 1:
        config = json.loads(open(sys.argv[1]).read())
    else:
        config = json.loads(sys.stdin.read())
    call_llm(config)
