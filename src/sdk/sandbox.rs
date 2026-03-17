use std::time::Duration;
use std::process::{Command, Stdio};
use std::io::Write;
use std::sync::mpsc;
use std::thread;

/// 隔离预言机契约 (The Isolated Oracle Protocol)
/// 任何社区方案 (本地进程, Wasm, MicroVM) 只需实现这个极其纯粹接口
pub trait SandboxEngine: Send + Sync {
    /// 引擎的物理签名 (例如 "sandbox.native_subprocess", "sandbox.wasmtime")
    fn engine_name(&self) -> &'static str;

    /// 传入拼接好的、包含隐藏 Metric (ROM) 的完整代码，在绝对隔离的环境中运行。
    /// 必须保证：支持超时物理熔断 (防止 LLM 输出死循环拖垮总线)
    fn execute_safely(&self, sealed_code: &str, timeout: Duration) -> Result<String, String>;
}

pub struct LocalProcessSandbox {
    compiler_cmd: String,
    args: Vec<String>,
}

impl LocalProcessSandbox {
    pub fn new(compiler_cmd: &str, args: Vec<String>) -> Self {
        Self { 
            compiler_cmd: compiler_cmd.to_string(),
            args 
        }
    }
}

impl SandboxEngine for LocalProcessSandbox {
    fn engine_name(&self) -> &'static str { "sandbox.ephemeral_native_process" }

    fn execute_safely(&self, sealed_code: &str, timeout: Duration) -> Result<String, String> {
        let mut child = Command::new(&self.compiler_cmd)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("FATAL: Failed to spawn local jail: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            let code = sealed_code.to_string();
            // Write code in a separate thread to avoid deadlock if pipes are full
            thread::spawn(move || {
                let _ = stdin.write_all(code.as_bytes());
            });
        }

        let (tx, rx) = mpsc::channel();
        
        // Wait for output in a separate thread
        thread::spawn(move || {
            let result = child.wait_with_output();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
                } else {
                    // 将沙盒内的痛苦嚎叫（Error Message）返回给上层，实现单向痛觉反馈
                    Err(String::from_utf8_lossy(&output.stderr).into_owned())
                }
            }
            Ok(Err(e)) => Err(format!("Execution failed: {}", e)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err("Sandbox Timeout: LLM generated an infinite loop or exceeded gas limit.".into())
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err("Sandbox internal error: thread disconnected".into())
            }
        }
    }
}
