#!/usr/bin/env python3
"""
LLM Proxy — OpenAI-compatible local HTTP server that forwards to cloud APIs.
Evaluator connects to this via reqwest (HTTP, no TLS) — the proven stable path.

Usage:
  DASHSCOPE_API_KEY=sk-xxx python3 llm_proxy.py [--port 8088]

Endpoints:
  POST /v1/chat/completions  (OpenAI-compatible)
  GET  /health
"""
import os, sys, json, logging, argparse
from http.server import HTTPServer, BaseHTTPRequestHandler
from socketserver import ThreadingMixIn
from openai import OpenAI

logging.basicConfig(level=logging.INFO, format='%(asctime)s %(levelname)s %(message)s')
log = logging.getLogger("llm_proxy")

PROVIDERS = {
    "dashscope": ("https://dashscope.aliyuncs.com/compatible-mode/v1", "DASHSCOPE_API_KEY"),
    "aliyun":    ("https://dashscope.aliyuncs.com/compatible-mode/v1", "DASHSCOPE_API_KEY"),
    "siliconflow": ("https://api.siliconflow.cn/v1", "SILICONFLOW_API_KEY"),
    "deepseek":  ("https://api.deepseek.com", "DEEPSEEK_API_KEY"),
    "volcengine": ("https://ark.cn-beijing.volces.com/api/v3", "VOLCENGINE_API_KEY"),
    "nvidia":    ("https://integrate.api.nvidia.com/v1", "NVIDIA_NIM_API_KEY"),
}

clients = {}  # provider -> OpenAI client (cached)

def get_client(provider):
    if provider not in clients:
        base_url, key_env = PROVIDERS.get(provider, PROVIDERS["dashscope"])
        api_key = os.environ.get(key_env, "")
        if not api_key:
            raise ValueError(f"Missing env: {key_env}")
        clients[provider] = OpenAI(api_key=api_key, base_url=base_url)
    return clients[provider]


def detect_provider(model):
    """Auto-detect provider from model name."""
    m = model.lower()
    if "deepseek" in m: return "deepseek"
    if "/" in model and not m.startswith("qwen"): return "siliconflow"  # SiliconFlow uses Org/Model format
    return "dashscope"  # Default: Aliyun DashScope


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status":"ok"}')
        else:
            self.send_error(404)

    def do_POST(self):
        if "/v1/chat/completions" not in self.path:
            self.send_error(404)
            return

        length = int(self.headers.get("Content-Length", 0))
        body = json.loads(self.rfile.read(length)) if length else {}

        model = body.get("model", "qwen3-8b")
        messages = body.get("messages", [])
        temperature = body.get("temperature", 0.5)
        max_tokens = body.get("max_tokens", 3072)
        enable_thinking = body.get("enable_thinking", False)

        provider = detect_provider(model)

        try:
            client = get_client(provider)

            extra = {}
            if enable_thinking:
                extra["extra_body"] = {"enable_thinking": True}
            elif "qwen3" in model.lower():
                extra["extra_body"] = {"enable_thinking": False}

            log.info(f"→ {provider}/{model} (temp={temperature}, max_tok={max_tokens})")

            resp = client.chat.completions.create(
                model=model,
                messages=messages,
                temperature=temperature,
                max_tokens=max_tokens,
                stream=True,
                **extra,
            )

            content = ""
            reasoning = ""
            for chunk in resp:
                delta = chunk.choices[0].delta
                if hasattr(delta, "reasoning_content") and delta.reasoning_content:
                    reasoning += delta.reasoning_content
                if hasattr(delta, "content") and delta.content:
                    content += delta.content

            # Return OpenAI-compatible response
            result = {
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": content,
                        "reasoning_content": reasoning if reasoning else None,
                    },
                    "finish_reason": "stop",
                }],
                "model": model,
            }

            log.info(f"← {provider}/{model}: {len(content)} chars content, {len(reasoning)} chars reasoning")

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(result).encode())

        except Exception as e:
            log.error(f"Error: {e}")
            self.send_response(500)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": {"message": str(e)}}).encode())

    def log_message(self, format, *args):
        pass  # Suppress default access log (we have our own)


class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=int(os.environ.get("LLM_PROXY_PORT", "8088")))
    args = parser.parse_args()

    server = ThreadedHTTPServer(("127.0.0.1", args.port), Handler)
    log.info(f"LLM Proxy listening on 127.0.0.1:{args.port}")
    log.info(f"Providers: {', '.join(k for k, (_, env) in PROVIDERS.items() if os.environ.get(env))}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        log.info("Shutting down")
