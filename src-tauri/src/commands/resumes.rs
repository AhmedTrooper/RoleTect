use tauri::State;
use nanoid::nanoid;
use crate::AppState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResumeItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResumeDetail {
    pub id: String,
    pub name: String,
    pub category: String,
    pub latex_content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResumeArgs {
    pub name: String,
    pub category: String,
    pub latex_content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResumeArgs {
    pub resume_id: String,
    pub name: String,
    pub category: String,
    pub latex_content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResumeArgs {
    pub resume_id: String,
}

#[tauri::command]
pub fn get_all_resumes(state: State<'_, AppState>) -> Result<Vec<ResumeItem>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        let mut stmt = conn
            .prepare("SELECT id, name, category, created_at, updated_at FROM base_resumes ORDER BY created_at DESC")
            .map_err(|e| format!("Query prepare error: {}", e))?;
        
        let resumes = stmt
            .query_map([], |row| {
                Ok(ResumeItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row collection error: {}", e))?;
        
        Ok(resumes)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn get_resume_by_id(state: State<'_, AppState>, resume_id: String) -> Result<ResumeDetail, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        let mut stmt = conn
            .prepare("SELECT id, name, category, latex_content, created_at, updated_at FROM base_resumes WHERE id = ?1")
            .map_err(|e| format!("Query prepare error: {}", e))?;
        
        let resume = stmt
            .query_row([resume_id], |row| {
                Ok(ResumeDetail {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    latex_content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Resume not found: {}", e))?;
        
        Ok(resume)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn create_new_resume(
    state: State<'_, AppState>,
    args: CreateResumeArgs,
) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        let resume_id = nanoid!(10);
        
        conn.execute(
            "INSERT INTO base_resumes (id, name, category, latex_content) VALUES (?1, ?2, ?3, ?4)",
            [&resume_id, &args.name, &args.category, &args.latex_content],
        ).map_err(|e| format!("Database error: {}", e))?;
        
        Ok(resume_id)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn update_resume(
    state: State<'_, AppState>,
    args: UpdateResumeArgs,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "UPDATE base_resumes SET name = ?1, category = ?2, latex_content = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
            [&args.name, &args.category, &args.latex_content, &args.resume_id],
        ).map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn delete_resume(state: State<'_, AppState>, args: DeleteResumeArgs) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "DELETE FROM base_resumes WHERE id = ?1",
            [&args.resume_id],
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    } else {
        Err("Database connection lost".to_string())
    }
}
