use crate::ai::{self};
use crate::commands::TailoredContent;
use crate::AppState;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct JobPayload {
    pub id: String,
    pub company_name: String,
    pub job_title: String,
    pub work_model: String,
    pub employment_type: String,
    pub status: String,
    pub raw_jd: String,
    pub requirements: Option<String>,
    pub core_responsibilities: Option<String>,
    pub custom_instruction: Option<String>,
    pub reference_name: Option<String>,
    pub reference_email: Option<String>,
    pub social_link: Option<String>,
    pub job_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[tauri::command]
pub async fn parse_job(
    provider: String,
    model: String,
    api_key: String,
    raw_jd: String,
    job_url: Option<String>,
) -> Result<ai::JobParseResult, String> {
    ai::parse_job_description(&provider, &model, &api_key, &raw_jd, job_url.as_deref()).await
}

#[tauri::command]
pub async fn save_job(state: State<'_, AppState>, payload: JobPayload) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    conn.execute(
        "INSERT INTO jobs (
            id, company_name, job_title, work_model, employment_type, 
            status, raw_jd, requirements, core_responsibilities,
            custom_instruction, reference_name, 
            reference_email, social_link, job_url
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        [
            &payload.id,
            &payload.company_name,
            &payload.job_title,
            &payload.work_model,
            &payload.employment_type,
            &payload.status,
            &payload.raw_jd,
            &payload.requirements.unwrap_or_default(),
            &payload.core_responsibilities.unwrap_or_default(),
            &payload.custom_instruction.unwrap_or_default(),
            &payload.reference_name.unwrap_or_default(),
            &payload.reference_email.unwrap_or_default(),
            &payload.social_link.unwrap_or_default(),
            &payload.job_url.unwrap_or_default(),
        ],
    )
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(payload.id)
}

#[tauri::command]
pub async fn get_job_by_id(state: State<'_, AppState>, id: String) -> Result<JobPayload, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, requirements, core_responsibilities,
                custom_instruction, reference_name, 
                reference_email, social_link, job_url,
                created_at, updated_at
         FROM jobs WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let job = stmt
        .query_row([&id], |row| {
            Ok(JobPayload {
                id: row.get(0)?,
                company_name: row.get(1)?,
                job_title: row.get(2)?,
                work_model: row.get(3)?,
                employment_type: row.get(4)?,
                status: row.get(5)?,
                raw_jd: row.get(6)?,
                requirements: row.get(7)?,
                core_responsibilities: row.get(8)?,
                custom_instruction: row.get(9)?,
                reference_name: row.get(10)?,
                reference_email: row.get(11)?,
                social_link: row.get(12)?,
                job_url: row.get(13)?,
                created_at: Some(row.get(14)?),
                updated_at: Some(row.get(15)?),
            })
        })
        .map_err(|e| format!("Job not found: {}", e))?;

    Ok(job)
}

#[tauri::command]
pub async fn get_all_jobs(state: State<'_, AppState>) -> Result<Vec<JobPayload>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, requirements, core_responsibilities,
                custom_instruction, reference_name, 
                reference_email, social_link, job_url,
                created_at, updated_at
         FROM jobs ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let job_iter = stmt
        .query_map([], |row| {
            Ok(JobPayload {
                id: row.get(0)?,
                company_name: row.get(1)?,
                job_title: row.get(2)?,
                work_model: row.get(3)?,
                employment_type: row.get(4)?,
                status: row.get(5)?,
                raw_jd: row.get(6)?,
                requirements: row.get(7)?,
                core_responsibilities: row.get(8)?,
                custom_instruction: row.get(9)?,
                reference_name: row.get(10)?,
                reference_email: row.get(11)?,
                social_link: row.get(12)?,
                job_url: row.get(13)?,
                created_at: Some(row.get(14)?),
                updated_at: Some(row.get(15)?),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut jobs = Vec::new();
    for job in job_iter {
        jobs.push(job.map_err(|e| e.to_string())?);
    }
    Ok(jobs)
}

#[tauri::command]
pub async fn delete_job(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    // Disable foreign keys temporarily to delete from all related tables if necessary,
    // though tailored_resumes has ON DELETE RESTRICT by default in many SQLites if not specified.
    // Better to delete related tailored_resumes first.
    conn.execute("DELETE FROM tailored_resumes WHERE job_id = ?1", [&id])
        .map_err(|e| format!("Database error (tailored_resumes): {}", e))?;

    conn.execute("DELETE FROM jobs WHERE id = ?1", [&id])
        .map_err(|e| format!("Database error (jobs): {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_jobs_batch(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    for id in ids {
        tx.execute("DELETE FROM tailored_resumes WHERE job_id = ?1", [&id])
            .map_err(|e| format!("Database error (tailored_resumes): {}", e))?;
        tx.execute("DELETE FROM jobs WHERE id = ?1", [&id])
            .map_err(|e| format!("Database error (jobs): {}", e))?;
    }

    tx.commit().map_err(|e| format!("Commit error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_all_jobs(state: State<'_, AppState>) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction error: {}", e))?;

    tx.execute("DELETE FROM tailored_resumes", [])
        .map_err(|e| format!("Database error (tailored_resumes): {}", e))?;
    tx.execute("DELETE FROM jobs", [])
        .map_err(|e| format!("Database error (jobs): {}", e))?;

    tx.commit().map_err(|e| format!("Commit error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_job_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    conn.execute("UPDATE jobs SET status = ?1 WHERE id = ?2", [&status, &id])
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_tailored_resume(
    state: State<'_, AppState>,
    job_id: String,
    base_resume_id: Option<String>,
    latex_content: String,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let rows_affected = conn.execute(
        "UPDATE tailored_resumes SET final_latex_content = ?1, updated_at = CURRENT_TIMESTAMP 
         WHERE job_id = ?2",
        [&latex_content, &job_id],
    )
    .map_err(|e| format!("Database error (update): {}", e))?;

    if rows_affected == 0 {
        if let Some(base_id) = base_resume_id {
            let id = nanoid!(10);
            conn.execute(
                "INSERT INTO tailored_resumes (id, job_id, base_resume_id, final_latex_content, is_active)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                [&id, &job_id, &base_id, &latex_content],
            )
            .map_err(|e| format!("Database error (insert): {}", e))?;
        } else {
            return Err("No tailored resume found to update. Please generate one first or select a template to initialize.".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tailored_resume(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare("SELECT final_latex_content FROM tailored_resumes WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let content: String = stmt
        .query_row([&id], |row| row.get(0))
        .map_err(|_| "Tailored resume not found".to_string())?;

    Ok(content)
}

#[tauri::command]
pub async fn get_latest_tailored_resume(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<TailoredContent>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, final_latex_content FROM tailored_resumes 
         WHERE job_id = ?1 
         ORDER BY created_at DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let result: Option<TailoredContent> = match stmt.query_row([&job_id], |row| {
        Ok(TailoredContent {
            id: row.get(0)?,
            content: row.get(1)?,
        })
    }) {
        Ok(v) => Some(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(e.to_string()),
    };

    Ok(result)
}

#[tauri::command]
pub async fn tailor_resume(
    state: State<'_, AppState>,
    provider: String,
    model: String,
    api_key: String,
    job_id: String,
    base_resume_id: String,
    custom_instruction: Option<String>,
) -> Result<String, String> {
    // 1. Fetch job and resume data
    let (raw_job_content, requirements, core_responsibilities, base_latex) = {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

        if let Some(conn) = db_guard.as_mut() {
            let mut stmt = conn
                .prepare(
                    "SELECT raw_jd, requirements, core_responsibilities FROM jobs WHERE id = ?1",
                )
                .map_err(|e| format!("Query prepare error: {}", e))?;

            let (raw_job, reqs, resps): (String, Option<String>, Option<String>) = stmt
                .query_row([&job_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map_err(|_| format!("Job not found: {}", job_id))?;

            let mut stmt = conn
                .prepare("SELECT latex_content FROM base_resumes WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;

            let latex: String = stmt
                .query_row([&base_resume_id], |row| row.get(0))
                .map_err(|_| format!("Base resume not found: {}", base_resume_id))?;

            (raw_job, reqs, resps, latex)
        } else {
            return Err("Database connection lost".to_string());
        }
    };

    // 2. Prepare tailored prompt content
    let job_context = format!(
        "Raw JD:\n{}\n\nExtracted Requirements:\n{}\n\nExtracted Responsibilities:\n{}",
        raw_job_content,
        requirements.unwrap_or_default(),
        core_responsibilities.unwrap_or_default()
    );

    // 3. Call AI to tailor the resume
    let tailored_latex = ai::tailor_latex_for_job(
        &provider,
        &model,
        &api_key,
        &base_latex,
        &job_context,
        custom_instruction.as_deref(),
    )
    .await?;

    // 4. Save to database
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
