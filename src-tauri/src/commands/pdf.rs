use tauri::command;

use crate::ai;

#[command]
pub async fn fix_latex_with_ai(
    provider: String,
    model: String,
    api_key: String,
    broken_latex: String,
    error_logs: String,
) -> Result<String, String> {
    ai::fix_latex_errors(&provider, &model, &api_key, &broken_latex, &error_logs).await
}

#[command]
pub async fn compile_resume_to_pdf(latex_code: String) -> Result<Vec<u8>, String> {
    // TeX engines are notoriously stack-heavy. Segfaults are frequently caused by 
    // stack overflows in threads with default sizes. We use a dedicated thread 
    // with a 10MB stack size to ensure the heavy TeX logic has enough room to run safely.
    tokio::task::spawn_blocking(move || {
        let thread_handle = std::thread::Builder::new()
            .name("tectonic-compiler".into())
            .stack_size(10 * 1024 * 1024) // 10MB
            .spawn(move || {
                tectonic::latex_to_pdf(latex_code)
                    .map_err(|e| format!("Tectonic compilation error: {}", e))
            })
            .map_err(|e| format!("Failed to spawn compiler thread: {}", e))?;

        thread_handle.join()
            .map_err(|_| "Compiler thread panicked or exited unexpectedly".to_string())?
    })
    .await
    .map_err(|e| format!("Blocking task failed: {}", e))?
}
