use tauri::command;
use std::path::PathBuf;
use crate::ai;
use tectonic::status::StatusBackend;
use tectonic::status::MessageKind;
use std::fmt::Arguments;

pub struct CapturingStatusBackend {
    pub logs: String,
}

impl CapturingStatusBackend {
    pub fn new() -> Self {
        Self { logs: String::new() }
    }
}

impl StatusBackend for CapturingStatusBackend {
    fn report(&mut self, kind: MessageKind, args: Arguments, err: Option<&anyhow::Error>) {
        let prefix = match kind {
            MessageKind::Error => "error: ",
            MessageKind::Warning => "warning: ",
            MessageKind::Note => "note: ",
        };
        let msg = format!("{}", args);
        self.logs.push_str(prefix);
        self.logs.push_str(&msg);
        
        if let Some(e) = err {
            self.logs.push_str(&format!(" (error detail: {})", e));
            // Also push to stdout for tauri dev logs
            eprintln!("Tectonic Error: {} - Detail: {}", msg, e);
        } else if kind == MessageKind::Error {
            eprintln!("Tectonic Error: {}", msg);
        }
        
        self.logs.push('\n');
    }

    fn dump_error_logs(&mut self, logs: &[u8]) {
        if let Ok(s) = std::str::from_utf8(logs) {
            self.logs.push_str("--- Underlying Error Logs ---\n");
            self.logs.push_str(s);
            self.logs.push('\n');
        }
    }
}

#[command]
pub async fn refine_latex_with_ai(
    provider: String,
    model: String,
    api_key: String,
    current_latex: String,
    instruction: String,
) -> Result<String, String> {
    ai::refine_technical_content(&provider, &model, &api_key, &current_latex, &instruction, "LaTeX").await
}

#[command]
pub async fn fix_latex_with_ai(
    provider: String,
    model: String,
    api_key: String,
    broken_latex: String,
    error_logs: String,
) -> Result<String, String> {
    ai::fix_technical_errors(&provider, &model, &api_key, &broken_latex, &error_logs, "LaTeX").await
}

#[command]
pub async fn refine_diagram_with_ai(
    provider: String,
    model: String,
    api_key: String,
    current_code: String,
    instruction: String,
    content_type: String,
) -> Result<String, String> {
    ai::refine_technical_content(&provider, &model, &api_key, &current_code, &instruction, &content_type).await
}

#[command]
pub async fn fix_diagram_with_ai(
    provider: String,
    model: String,
    api_key: String,
    broken_code: String,
    error_logs: String,
    content_type: String,
) -> Result<String, String> {
    ai::fix_technical_errors(&provider, &model, &api_key, &broken_code, &error_logs, &content_type).await
}

#[command]
pub async fn compile_resume_to_pdf(latex_code: String) -> Result<Vec<u8>, String> {
    tokio::task::spawn_blocking(move || {
        let thread_handle = std::thread::Builder::new()
            .name("tectonic-compiler".into())
            .stack_size(10 * 1024 * 1024)
            .spawn(move || {
                let mut status = CapturingStatusBackend::new();
                
                let config_loader = tectonic::config::PersistentConfig::default();
                let bundle = config_loader
                    .default_bundle(false)
                    .map_err(|e| format!("Failed to load Tectonic bundle: {}", e))?;

                let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
                sb.bundle(bundle)
                    .primary_input_buffer(latex_code.as_bytes())
                    .tex_input_name("texput")
                    .filesystem_root(std::env::temp_dir()) // Use temp dir for intermediate files
                    .format_name("latex")
                    .output_format(tectonic::driver::OutputFormat::Pdf)
                    .build_date(std::time::SystemTime::now());

                let mut sess = sb.create(&mut status)
                    .map_err(|e| format!("Failed to create Tectonic session: {}\n\nLogs:\n{}", e, status.logs))?;

                sess.run(&mut status)
                    .map_err(|e| format!("Compilation failed: {}\n\nLogs:\n{}", e, status.logs))?;

                let out_data = sess.into_file_data();
                
                // For standalone, the output is in the "primary" entry of the memory filesystem
                out_data.get("texput.pdf")
                    .cloned()
                    .ok_or_else(|| format!("Compilation appeared successful, but 'texput.pdf' was not generated.\n\nLogs:\n{}", status.logs))
                    .map(|f| f.data)
            })
            .map_err(|e| format!("Failed to spawn compiler thread: {}", e))?;

        thread_handle
            .join()
            .map_err(|_| "Compiler thread panicked".to_string())?
    })
    .await
    .map_err(|e| format!("Blocking task failed: {}", e))?
}

#[command]
pub async fn compile_workspace_to_pdf(workspace_dir: String, main_file_name: String) -> Result<Vec<u8>, String> {
    let workspace_path = PathBuf::from(&workspace_dir);
    
    if !workspace_path.is_dir() {
        return Err(format!("Workspace path '{}' is not a valid directory.", workspace_dir));
    }

    tokio::task::spawn_blocking(move || {
        let thread_handle = std::thread::Builder::new()
            .name("tectonic-workspace-compiler".into())
            .stack_size(10 * 1024 * 1024)
            .spawn(move || {
                let mut status = CapturingStatusBackend::new();
                let workspace_path = PathBuf::from(&workspace_dir);
                
                let config_loader = tectonic::config::PersistentConfig::default();
                let bundle = config_loader
                    .default_bundle(false)
                    .map_err(|e| format!("Failed to load Tectonic bundle: {}", e))?;

                // Determine the absolute path to the main file
                let main_file_path = workspace_path.join(&main_file_name);
                if !main_file_path.is_file() {
                    return Err(format!("Main TeX file '{}' not found in workspace.", main_file_name));
                }

                // Use file_stem for the logical input name (job name) to avoid .tex extension issues
                let logical_input_name = PathBuf::from(&main_file_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Invalid main file name".to_string())?;

                let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
                let temp_output_dir = std::env::temp_dir().join(format!("roleflux-{}", nanoid::nanoid!()));
                std::fs::create_dir_all(&temp_output_dir).map_err(|e| format!("Failed to create temp output dir: {}", e))?;

                sb.bundle(bundle)
                    .primary_input_path(&main_file_path)
                    .tex_input_name("texput.tex")
                    .filesystem_root(&workspace_path)
                    .output_dir(&temp_output_dir) // Use temp dir for ALL outputs
                    .format_name("latex")
                    .output_format(tectonic::driver::OutputFormat::Pdf);

                let mut sess = sb.create(&mut status)
                    .map_err(|e| format!("Failed to create Tectonic session: {}\n\nLogs:\n{}", e, status.logs))?;

                sess.run(&mut status)
                    .map_err(|e| format!("Compilation failed: {}\n\nLogs:\n{}", e, status.logs))?;

                // The PDF will be named texput.pdf in the temp_output_dir
                let temp_pdf_path = temp_output_dir.join("texput.pdf");
                
                if temp_pdf_path.exists() {
                    let pdf_data = std::fs::read(&temp_pdf_path).map_err(|e| format!("Failed to read generated PDF: {}", e))?;
                    
                    // Also copy it to the workspace root for the user, naming it after the original file
                    let mut final_pdf_path = workspace_path.join(&logical_input_name);
                    final_pdf_path.set_extension("pdf");
                    let _ = std::fs::write(&final_pdf_path, &pdf_data);

                    // Clean up temp dir
                    let _ = std::fs::remove_dir_all(&temp_output_dir);

                    Ok(pdf_data)
                } else {
                    let _ = std::fs::remove_dir_all(&temp_output_dir);
                    Err(format!("Compilation appeared successful, but PDF was not found at {:?}\n\nLogs:\n{}", temp_pdf_path, status.logs))
                }
            })
            .map_err(|e| format!("Failed to spawn compiler thread: {}", e))?;

        thread_handle
            .join()
            .map_err(|_| "The compiler thread panicked.".to_string())?
    })
    .await
    .map_err(|e| format!("The asynchronous task failed: {}", e))?
}
