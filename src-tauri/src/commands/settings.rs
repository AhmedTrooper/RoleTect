use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// Create a struct to easily return both settings to the frontend
#[derive(Serialize, Deserialize)]
pub struct AiConfig {
    provider: String,
    model: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub config: String,
    pub is_builtin: bool,
}

#[tauri::command]
pub async fn save_model_pref(
    state: State<'_, AppState>,
    provider: String,
    model: String,
) -> Result<(), String> {
    state.with_db(|conn| {
        // Save Provider
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('ai_provider', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [&provider],
        ).map_err(|e| e.to_string())?;

        // Save Model
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('ai_model', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [&model],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }).await
}

#[tauri::command]
pub async fn get_model_pref(state: State<'_, AppState>) -> Result<AiConfig, String> {
    state.with_db(|conn| {
        // Fetch Provider (Default to 'openai')
        let provider: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_provider'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "openai".to_string());

        // Fetch Model (Default to 'gpt-4o')
        let model: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "gpt-4o".to_string());

        Ok(AiConfig { provider, model })
    }).await
}

#[tauri::command]
pub async fn get_all_themes(state: State<'_, AppState>) -> Result<Vec<Theme>, String> {
    state.with_db(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, name, config, is_builtin FROM themes ORDER BY is_builtin DESC, created_at DESC")
            .map_err(|e| e.to_string())?;

        let theme_iter = stmt
            .query_map([], |row| {
                Ok(Theme {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    config: row.get(2)?,
                    is_builtin: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut themes = Vec::new();
        for theme in theme_iter {
            themes.push(theme.map_err(|e| e.to_string())?);
        }
        Ok(themes)
    }).await
}

#[tauri::command]
pub async fn save_custom_theme(
    state: State<'_, AppState>,
    id: String,
    name: String,
    config: String,
) -> Result<(), String> {
    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO themes (id, name, config, is_builtin) VALUES (?1, ?2, ?3, 0)
             ON CONFLICT(id) DO UPDATE SET name=excluded.name, config=excluded.config",
            [&id, &name, &config],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await
}

#[tauri::command]
pub async fn delete_theme(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.with_db(|conn| {
        // Don't allow deleting builtin themes
        let is_builtin: bool = conn.query_row(
            "SELECT is_builtin FROM themes WHERE id = ?1",
            [&id],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        if is_builtin {
            return Err("Cannot delete built-in themes".to_string());
        }

        conn.execute("DELETE FROM themes WHERE id = ?1", [&id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }).await
}

#[tauri::command]
pub async fn save_active_theme(state: State<'_, AppState>, theme_id: String) -> Result<(), String> {
    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('active_theme', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [&theme_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }).await
}

#[tauri::command]
pub async fn get_active_theme(state: State<'_, AppState>) -> Result<Theme, String> {
    state.with_db(|conn| {
        let theme_id: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_theme'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "github-dark".to_string());

        let theme = conn.query_row(
            "SELECT id, name, config, is_builtin FROM themes WHERE id = ?1",
            [&theme_id],
            |row| Ok(Theme {
                id: row.get(0)?,
                name: row.get(1)?,
                config: row.get(2)?,
                is_builtin: row.get(3)?,
            })
        ).or_else(|_| {
            // Fallback if the active theme was deleted
            conn.query_row(
                "SELECT id, name, config, is_builtin FROM themes WHERE id = 'github-dark'",
                [],
                |row| Ok(Theme {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    config: row.get(2)?,
                    is_builtin: row.get(3)?,
                })
            )
        }).map_err(|e| e.to_string())?;

        Ok(theme)
    }).await
}
