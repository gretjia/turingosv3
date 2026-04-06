#!/usr/bin/env python3
"""
LLM Proxy — OpenAI-compatible local HTTP server with token metering.

Endpoints:
  POST /v1/chat/completions  (OpenAI-compatible, forwards to cloud APIs)
  GET  /health
  GET  /stats               (token counters: prompt, completion, total, requests)
  POST /stats/reset         (reset counters — call before each experiment)

Usage:
  DASHSCOPE_API_KEY=sk-xxx python3 llm_proxy.py --port 8088 --provider dashscope
"""
import os, sys, json, logging, argparse, time, threading
from http.server import HTTPServer, BaseHTTPRequestHandler
from socketserver import ThreadingMixIn
from openai import OpenAI, RateLimitError, APIStatusError

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

clients = {}

# ── Token Metering ──
_stats_lock = threading.Lock()
_stats = {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0,
    "requests": 0,
    "errors": 0,
    "retries_429": 0,
    "estimated_count": 0,
}


def _record_usage(prompt_tokens, completion_tokens):
    with _stats_lock:
        _stats["prompt_tokens"] += prompt_tokens
        _stats["completion_tokens"] += completion_tokens
        _stats["total_tokens"] += prompt_tokens + completion_tokens
        _stats["requests"] += 1


def _record_estimated():
    with _stats_lock:
        _stats["estimated_count"] += 1


def _record_error():
    with _stats_lock:
        _stats["errors"] += 1


def _record_retry():
    with _stats_lock:
        _stats["retries_429"] += 1


def _reset_stats():
    with _stats_lock:
        for k in _stats:
            _stats[k] = 0


def _get_stats():
    with _stats_lock:
        return dict(_stats)


# ── Rate Limiter ──
_rate_lock = threading.Lock()
_rate_semaphore = threading.Semaphore(int(os.environ.get("LLM_PROXY_CONCURRENCY", "5")))
_cooldown_until = 0.0


def _wait_for_cooldown():
    global _cooldown_until
    now = time.time()
    if now < _cooldown_until:
        wait = _cooldown_until - now
        log.info(f"[RATE LIMITER] Cooling down {wait:.1f}s")
        time.sleep(wait)


def _trigger_cooldown(seconds):
    global _cooldown_until
    with _rate_lock:
        new_until = time.time() + seconds
        if new_until > _cooldown_until:
            _cooldown_until = new_until
            log.warning(f"[RATE LIMITER] Global cooldown {seconds}s")


def get_client(provider):
    if provider not in clients:
        base_url, key_env = PROVIDERS.get(provider, PROVIDERS["dashscope"])
        api_key = os.environ.get(key_env, "")
        if not api_key:
            raise ValueError(f"Missing env: {key_env}")
        clients[provider] = OpenAI(api_key=api_key, base_url=base_url)
    return clients[provider]


def detect_provider(model):
    m = model.lower()
    if "deepseek" in m: return "deepseek"
    if "/" in model and not m.startswith("qwen"): return "siliconflow"
    return "dashscope"


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/health":
            self._json_response(200, {"status": "ok"})
        elif self.path == "/stats":
            self._json_response(200, _get_stats())
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path == "/stats/reset":
            _reset_stats()
            self._json_response(200, {"status": "reset"})
            log.info("[STATS] Counters reset")
            return

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

        provider = FORCED_PROVIDER or detect_provider(model)

        try:
            client = get_client(provider)

            extra = {}
            if enable_thinking:
                extra["extra_body"] = {"enable_thinking": True}
            elif "qwen3" in model.lower():
                extra["extra_body"] = {"enable_thinking": False}

            max_retries = 8
            content = ""
            reasoning = ""
            usage_prompt = 0
            usage_completion = 0

            _wait_for_cooldown()
            _rate_semaphore.acquire()
            try:
                for attempt in range(max_retries + 1):
                    _wait_for_cooldown()
                    try:
                        if attempt == 0:
                            log.info(f"→ {provider}/{model} (temp={temperature}, max_tok={max_tokens})")
                        else:
                            log.info(f"→ {provider}/{model} (retry {attempt}/{max_retries})")

                        # Non-streaming call for accurate token counting
                        resp = client.chat.completions.create(
                            model=model,
                            messages=messages,
                            temperature=temperature,
                            max_tokens=max_tokens,
                            stream=False,
                            **extra,
                        )

                        msg = resp.choices[0].message
                        content = msg.content or ""
                        reasoning = getattr(msg, "reasoning_content", None) or ""

                        # Extract real token counts from API response
                        estimated = False
                        if resp.usage and resp.usage.completion_tokens:
                            usage_prompt = resp.usage.prompt_tokens or 0
                            usage_completion = resp.usage.completion_tokens or 0
                        else:
                            # Fallback estimate — marked as estimated for PPUT validation
                            estimated = True
                            usage_prompt = sum(len(m.get("content", "")) for m in messages) // 3
                            usage_completion = (len(content) + len(reasoning)) // 3

                        break  # success

                    except (RateLimitError, APIStatusError) as e:
                        is_429 = isinstance(e, RateLimitError) or (hasattr(e, 'status_code') and e.status_code == 429)
                        if is_429 and attempt < max_retries:
                            _record_retry()
                            wait = min(2 ** attempt + 1, 30)
                            _trigger_cooldown(wait)
                            time.sleep(wait)
                        else:
                            raise
            finally:
                _rate_semaphore.release()

            # Record token usage
            _record_usage(usage_prompt, usage_completion)
            if estimated:
                _record_estimated()

            # Return OpenAI-compatible response with usage
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
                "usage": {
                    "prompt_tokens": usage_prompt,
                    "completion_tokens": usage_completion,
                    "total_tokens": usage_prompt + usage_completion,
                    "estimated": estimated,
                },
            }

            log.info(f"← {provider}/{model}: {len(content)}c content, {len(reasoning)}c reasoning, {usage_prompt}+{usage_completion}={usage_prompt+usage_completion} tokens")

            self._json_response(200, result)

        except Exception as e:
            _record_error()
            log.error(f"Error: {e}")
            self._json_response(500, {"error": {"message": str(e)}})

    def _json_response(self, code, data):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def log_message(self, format, *args):
        pass


class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True


FORCED_PROVIDER = None


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=int(os.environ.get("LLM_PROXY_PORT", "8088")))
    parser.add_argument("--provider", type=str, default=None,
                        help="Force all requests to this provider")
    args = parser.parse_args()

    if args.provider:
        if args.provider not in PROVIDERS:
            log.error(f"Unknown provider: {args.provider}. Available: {list(PROVIDERS.keys())}")
            sys.exit(1)
        FORCED_PROVIDER = args.provider
        log.info(f"Provider forced to: {args.provider}")

    server = ThreadedHTTPServer(("127.0.0.1", args.port), Handler)
    log.info(f"LLM Proxy listening on 127.0.0.1:{args.port}")
    log.info(f"Providers: {', '.join(k for k, (_, env) in PROVIDERS.items() if os.environ.get(env))}")
    log.info(f"Token metering: enabled (/stats, /stats/reset)")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        log.info("Shutting down")
