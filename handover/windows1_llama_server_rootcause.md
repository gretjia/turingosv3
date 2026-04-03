# Windows1 llama-server 启动问题 — 根因分析

## 症状
通过 SSH 使用 `Start-Process -WindowStyle Hidden` 启动 llama-server，进程加载 RPC DLL 后立即崩溃。
tasklist 看不到进程。llama_err.txt 只有一行 "loaded RPC backend"。

## 根因
**Windows OpenSSH Session 0 无法初始化 Vulkan GPU 上下文。**

- SSH 默认在 Session 0 (非桌面服务会话) 中运行
- `Start-Process -WindowStyle Hidden` 创建的子进程也在 Session 0
- llama-server 初始化 Vulkan GPU 时需要桌面会话 (Session 1+) 的 GPU 上下文
- Session 0 没有 GPU 访问权限 → Vulkan 初始化失败 → 进程崩溃

## 验证
- **前台运行 (ssh cmd 中直接执行)**: 成功 ✓ — SSH 控制台模拟了足够的会话上下文
- **Start-Process Hidden**: 失败 ✗ — 完全脱离控制台
- **schtasks**: 失败 ✗ — 以 SYSTEM 用户在 Session 0 运行
- **PowerShell Start-Job**: 成功 ✓ — 在当前 SSH 会话的上下文中后台运行

## 解决方案

### 方案 A: PowerShell Start-Job (当前使用)
```
ssh windows1-w1 "powershell -Command \"Start-Job -ScriptBlock { 
  cd C:\Users\jiazi\work\models; 
  .\llama-server.exe -m Qwen3.5-9B-Q4_K_M.gguf --host 0.0.0.0 --port 8080 -c 8192 --parallel 2 --threads 16 2>&1 > C:\Users\jiazi\work\llama_fg.log 
}; Start-Sleep -Seconds 30; Get-Job\""
```
SSH 退出后进程存活。GPU 上下文保持有效。

### 方案 B: 本地启动 (备选)
在 Windows 桌面上直接双击或打开 PowerShell 执行 start_server.ps1。

## 性能参数
- Mac (M4 32GB): `--parallel 2 -c 8192` → 37 tok/s, 避免 KV cache 占满内存
- Win1 (AMD 128GB): `--parallel 2 -c 8192` → 32 tok/s, 108GB 空闲
- **避免 `--parallel 4+` + 大 context**: 会导致 KV cache 分割降低单 slot 速度

## 日期
2026-04-03
