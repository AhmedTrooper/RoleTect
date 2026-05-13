use tauri::State;
use nanoid::nanoid;
use crate::AppState;
use crate::ai;

#[tauri::command]
pub async fn parse_and_save_job(
    state: State<'_, AppState>, 
    api_key: String, 
    raw_jd: String
) -> Result<String, String> {
    // 1. Call the AI to parse the raw text
    let parsed_data = ai::parse_job_description(&api_key, &raw_jd).await?;
    
    // 2. Augment parsed JSON with raw_job_content field
    let mut json_value = serde_json::to_value(&parsed_data)
        .map_err(|e| format!("JSON conversion error: {}", e))?;
    
    // Add raw_job_content to the JSON object for later reference during tailoring
    if let Some(obj) = json_value.as_object_mut() {
        obj.insert("raw_job_content".to_string(), serde_json::Value::String(raw_jd.clone()));
    }
    
    let parsed_json_string = serde_json::to_string(&json_value)
        .map_err(|e| format!("JSON Serialization error: {}", e))?;

    // 3. Generate a unique 10-character slug
    let job_slug = nanoid!(10);
    
    // 4. Save to SQLite
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "INSERT INTO jobs (id, company_name, job_title, raw_jd, parsed_json, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, 'Drafting')",
            [
                &job_slug, 
                &parsed_data.company, 
                &parsed_data.title, 
                &raw_jd, 
                &parsed_json_string
            ],
        ).map_err(|e| format!("Database error: {}", e))?;
        
        Ok(job_slug)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub async fn tailor_resume(
    state: State<'_, AppState>,
    api_key: String,
    job_id: String,
    base_resume_id: String,
    custom_instruction: Option<String>,
) -> Result<String, String> {
    // 1. Fetch job and resume data (hold lock only briefly)
    let (raw_job_content, base_latex) = {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
        
        if let Some(conn) = db_guard.as_mut() {
            // Fetch job data to get raw_job_content from parsed_json
            let mut stmt = conn
                .prepare("SELECT parsed_json FROM jobs WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;
            
            let job_json_str: String = stmt
                .query_row([&job_id], |row| row.get(0))
                .map_err(|_| format!("Job not found: {}", job_id))?;
            
            let job_json: serde_json::Value = serde_json::from_str(&job_json_str)
                .map_err(|e| format!("JSON parse error: {}", e))?;
            
            let raw_job = job_json
                .get("raw_job_content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "raw_job_content not found in job data".to_string())?
                .to_string();
            
            // Fetch base resume LaTeX content
            let mut stmt = conn
                .prepare("SELECT latex_content FROM base_resumes WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;
            
            let latex: String = stmt
                .query_row([&base_resume_id], |row| row.get(0))
                .map_err(|_| format!("Base resume not found: {}", base_resume_id))?;
            
            (raw_job, latex)
        } else {
            return Err("Database connection lost".to_string());
        }
    }; // db_guard is dropped here
    
    // 2. Call AI to tailor the resume (now safe, no non-Send types held)
    let tailored_latex = ai::tailor_latex_for_job(
        &api_key,
        &base_latex,
        &raw_job_content,
        custom_instruction.as_deref(),
    )
    .await?;
    
    // 3. Save to database
    {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
        
        if let Some(conn) = db_guard.as_mut() {
            let tailored_id = nanoid!(10);
            
            conn.execute(
                "INSERT INTO tailored_resumes (id, job_id, base_resume_id, final_latex_content, is_active)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                [
                    &tailored_id,
                    &job_id,
                    &base_resume_id,
                    &tailored_latex,
                ],
            ).map_err(|e| format!("Database error: {}", e))?;
            
            Ok(tailored_id)
        } else {
            Err("Database connection lost".to_string())
        }
    }
}