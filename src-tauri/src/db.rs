use rusqlite::{Connection, Result};
use std::fs;
use tauri::{AppHandle, Manager};

pub fn init_db(app: &AppHandle) -> Result<Connection> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    let db_path = app_dir.join("cvsynth.db");

    let conn = Connection::open(db_path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute_batch(
        "
        -- 1. App Settings Table
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY, 
            value TEXT NOT NULL
        );

        -- 2. Base Resumes Table
        CREATE TABLE IF NOT EXISTS base_resumes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            latex_content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TRIGGER IF NOT EXISTS update_base_resumes_modtime 
            AFTER UPDATE ON base_resumes 
            BEGIN UPDATE base_resumes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 2b. Base Cover Letters Table
        CREATE TABLE IF NOT EXISTS base_cover_letters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            latex_content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TRIGGER IF NOT EXISTS update_base_cover_letters_modtime 
            AFTER UPDATE ON base_cover_letters 
            BEGIN UPDATE base_cover_letters SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 3. Jobs Table (Flexible Schema)
        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            company_name TEXT NOT NULL,
            job_title TEXT NOT NULL,
            work_model TEXT DEFAULT 'Remote',
            employment_type TEXT DEFAULT 'Full-time',
            status TEXT NOT NULL DEFAULT 'Drafting',
            raw_jd TEXT NOT NULL,
            requirements TEXT,
            core_responsibilities TEXT,
            custom_instruction TEXT,
            reference_name TEXT,
            reference_email TEXT,
            social_link TEXT,
            job_url TEXT,
            base_resume_id TEXT,
            base_cl_id TEXT,
            salary TEXT,
            applied_date TEXT,
            interview_date TEXT,
            offer_date TEXT,
            rejected_date TEXT,
            joining_date TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (base_resume_id) REFERENCES base_resumes(id),
            FOREIGN KEY (base_cl_id) REFERENCES base_cover_letters(id)
        );
        CREATE TRIGGER IF NOT EXISTS update_jobs_modtime 
            AFTER UPDATE ON jobs 
            BEGIN UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 4. Tailored Resumes Table (Generated Output)
        CREATE TABLE IF NOT EXISTS tailored_resumes (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            base_resume_id TEXT NOT NULL,
            final_latex_content TEXT NOT NULL,
            is_active BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (job_id) REFERENCES jobs(id),
            FOREIGN KEY (base_resume_id) REFERENCES base_resumes(id)
        );
        CREATE TRIGGER IF NOT EXISTS update_tailored_resumes_modtime 
            AFTER UPDATE ON tailored_resumes 
            BEGIN UPDATE tailored_resumes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 4b. Tailored Cover Letters Table (Generated Output)
        CREATE TABLE IF NOT EXISTS tailored_cover_letters (
            id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL,
            base_cl_id TEXT NOT NULL,
            final_latex_content TEXT NOT NULL,
            is_active BOOLEAN DEFAULT 1,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (job_id) REFERENCES jobs(id),
            FOREIGN KEY (base_cl_id) REFERENCES base_cover_letters(id)
        );
        CREATE TRIGGER IF NOT EXISTS update_tailored_cover_letters_modtime 
            AFTER UPDATE ON tailored_cover_letters 
            BEGIN UPDATE tailored_cover_letters SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 5. Standalone Compiler State Table
        CREATE TABLE IF NOT EXISTS compiler_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            latex_content TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 6. Downloads Table
        CREATE TABLE IF NOT EXISTS downloads (
            id TEXT PRIMARY KEY,
            filename TEXT NOT NULL,
            download_type TEXT NOT NULL,
            job_id TEXT,
            content_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (job_id) REFERENCES jobs(id)
        );

        -- 7. Themes Table
        CREATE TABLE IF NOT EXISTS themes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            config TEXT NOT NULL,
            is_builtin BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "
    )?;

    // --- MIGRATIONS ---

    // 0. Ensure 'name' is unique in themes table (for existing databases)
    let table_info: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='themes'",
        [],
        |row| row.get(0)
    ).unwrap_or_default();

    if !table_info.contains("UNIQUE") {
        println!("Migrating 'themes' table to ensure unique names...");
        conn.execute_batch("
            PRAGMA foreign_keys=OFF;
            BEGIN TRANSACTION;
            CREATE TABLE themes_new (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                config TEXT NOT NULL,
                is_builtin BOOLEAN DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            INSERT OR IGNORE INTO themes_new SELECT * FROM themes;
            DROP TABLE themes;
            ALTER TABLE themes_new RENAME TO themes;
            COMMIT;
            PRAGMA foreign_keys=ON;
        ")?;
    }

        // 1. Check if themes table is empty and insert default theme if so
        let theme_count: i32 = conn.query_row("SELECT COUNT(*) FROM themes", [], |row| row.get(0))?;
        if theme_count == 0 {
            let github_dark = r##" {
            "--bg": "#0d0f14",
            "--bg-accent": "#11141d",
            "--surface": "#161923",
            "--surface-soft": "#1d222e",
            "--ink": "#e6edf3",
            "--muted": "#8b949e",
            "--line": "#30363d",
            "--accent": "#238636",
            "--accent-soft": "rgba(35, 134, 54, 0.15)",
            "--warning": "#f85149"
        } "##;

            conn.execute(
                "INSERT INTO themes (id, name, config, is_builtin) VALUES ('github-dark', 'GitHub Dark', ?1, 1)",
                [github_dark],
            )?;

            let dracula = r##" {
            "--bg": "#282a36",
            "--bg-accent": "#1e1f29",
            "--surface": "#44475a",
            "--surface-soft": "#6272a4",
            "--ink": "#f8f8f2",
            "--muted": "#6272a4",
            "--line": "#44475a",
            "--accent": "#bd93f9",
            "--accent-soft": "rgba(189, 147, 249, 0.15)",
            "--warning": "#ff5555"
        } "##;

            conn.execute(
                "INSERT INTO themes (id, name, config, is_builtin) VALUES ('dracula', 'Dracula', ?1, 1)",
                [dracula],
            )?;

            let nord = r##" {
            "--bg": "#2e3440",
            "--bg-accent": "#242933",
            "--surface": "#3b4252",
            "--surface-soft": "#434c5e",
            "--ink": "#d8dee9",
            "--muted": "#4c566a",
            "--line": "#3b4252",
            "--accent": "#88c0d0",
            "--accent-soft": "rgba(136, 192, 208, 0.15)",
            "--warning": "#bf616a"
        } "##;

            conn.execute(
                "INSERT INTO themes (id, name, config, is_builtin) VALUES ('nord', 'Nord', ?1, 1)",
                [nord],
            )?;
        }

        // 1b. Overhaul built-in themes (8 Dark, 8 Light + GitHub Dark)
        let builtin_themes = vec![
            ("github-dark", "GitHub Dark", r##" {
                "--bg": "#0d1117",
                "--bg-accent": "#161b22",
                "--surface": "#21262d",
                "--surface-soft": "#30363d",
                "--ink": "#c9d1d9",
                "--muted": "#8b949e",
                "--line": "#30363d",
                "--accent": "#238636",
                "--accent-soft": "rgba(35, 134, 54, 0.15)",
                "--warning": "#f85149"
            } "##),
            ("dracula", "Dracula", r##" {
                "--bg": "#282a36",
                "--bg-accent": "#1e1f29",
                "--surface": "#343746",
                "--surface-soft": "#44475a",
                "--ink": "#f8f8f2",
                "--muted": "#6272a4",
                "--line": "#44475a",
                "--accent": "#bd93f9",
                "--accent-soft": "rgba(189, 147, 249, 0.15)",
                "--warning": "#ff5555"
            } "##),
            ("nord-dark", "Nord Dark", r##" {
                "--bg": "#2e3440",
                "--bg-accent": "#242933",
                "--surface": "#3b4252",
                "--surface-soft": "#434c5e",
                "--ink": "#eceff4",
                "--muted": "#4c566a",
                "--line": "#3b4252",
                "--accent": "#88c0d0",
                "--accent-soft": "rgba(136, 192, 208, 0.15)",
                "--warning": "#bf616a"
            } "##),
            ("one-dark", "One Dark", r##" {
                "--bg": "#282c34",
                "--bg-accent": "#21252b",
                "--surface": "#2c313a",
                "--surface-soft": "#3e4451",
                "--ink": "#abb2bf",
                "--muted": "#5c6370",
                "--line": "#3e4451",
                "--accent": "#61afef",
                "--accent-soft": "rgba(97, 175, 239, 0.15)",
                "--warning": "#e06c75"
            } "##),
            ("catppuccin-macchiato", "Catppuccin Macchiato", r##" {
                "--bg": "#24273a",
                "--bg-accent": "#1e2030",
                "--surface": "#363a4f",
                "--surface-soft": "#494d64",
                "--ink": "#cad3f5",
                "--muted": "#8087a2",
                "--line": "#494d64",
                "--accent": "#8aadf4",
                "--accent-soft": "rgba(138, 173, 244, 0.15)",
                "--warning": "#ed8796"
            } "##),
            ("everforest-dark", "Everforest Dark", r##" {
                "--bg": "#2d353b",
                "--bg-accent": "#232a2e",
                "--surface": "#343f44",
                "--surface-soft": "#3d484d",
                "--ink": "#d3c6aa",
                "--muted": "#859289",
                "--line": "#475258",
                "--accent": "#a7c080",
                "--accent-soft": "rgba(167, 192, 128, 0.15)",
                "--warning": "#e67e80"
            } "##),
            ("tokyo-night", "Tokyo Night", r##" {
                "--bg": "#1a1b26",
                "--bg-accent": "#16161e",
                "--surface": "#24283b",
                "--surface-soft": "#414868",
                "--ink": "#a9b1d6",
                "--muted": "#565f89",
                "--line": "#24283b",
                "--accent": "#7aa2f7",
                "--accent-soft": "rgba(122, 162, 247, 0.15)",
                "--warning": "#f7768e"
            } "##),
            ("night-owl", "Night Owl", r##" {
                "--bg": "#011627",
                "--bg-accent": "#010e1b",
                "--surface": "#0b2942",
                "--surface-soft": "#1d3b53",
                "--ink": "#d6deeb",
                "--muted": "#5f7e97",
                "--line": "#1d3b53",
                "--accent": "#82aaff",
                "--accent-soft": "rgba(130, 170, 255, 0.15)",
                "--warning": "#ef5350"
            } "##),
            ("rose-pine-moon", "Rosé Pine Moon", r##" {
                "--bg": "#232136",
                "--bg-accent": "#2a273f",
                "--surface": "#393552",
                "--surface-soft": "#44415a",
                "--ink": "#e0def4",
                "--muted": "#908caa",
                "--line": "#44415a",
                "--accent": "#ea9a97",
                "--accent-soft": "rgba(234, 154, 151, 0.15)",
                "--warning": "#eb6f92"
            } "##),
            ("github-light", "GitHub Light", r##" {
                "--bg": "#ffffff",
                "--bg-accent": "#f6f8fa",
                "--surface": "#ffffff",
                "--surface-soft": "#f6f8fa",
                "--ink": "#24292f",
                "--muted": "#57606a",
                "--line": "#d0d7de",
                "--accent": "#0969da",
                "--accent-soft": "rgba(9, 105, 218, 0.1)",
                "--warning": "#cf222e"
            } "##),
            ("everforest-light", "Everforest Light", r##" {
                "--bg": "#fdf6e3",
                "--bg-accent": "#fefcf0",
                "--surface": "#f8f0dc",
                "--surface-soft": "#efebd4",
                "--ink": "#5c6a72",
                "--muted": "#939f91",
                "--line": "#e8e5d5",
                "--accent": "#8da101",
                "--accent-soft": "rgba(141, 161, 1, 0.1)",
                "--warning": "#f85552"
            } "##),
            ("catppuccin-latte", "Catppuccin Latte", r##" {
                "--bg": "#eff1f5",
                "--bg-accent": "#e6e9ef",
                "--surface": "#ccd0da",
                "--surface-soft": "#bcc0cc",
                "--ink": "#4c4f69",
                "--muted": "#7c7f93",
                "--line": "#bcc0cc",
                "--accent": "#1e66f5",
                "--accent-soft": "rgba(30, 102, 245, 0.1)",
                "--warning": "#d20f39"
            } "##),
            ("nord-light", "Nord Light", r##" {
                "--bg": "#eceff4",
                "--bg-accent": "#e5e9f0",
                "--surface": "#d8dee9",
                "--surface-soft": "#cdd3de",
                "--ink": "#2e3440",
                "--muted": "#4c566a",
                "--line": "#d8dee9",
                "--accent": "#5e81ac",
                "--accent-soft": "rgba(94, 129, 172, 0.1)",
                "--warning": "#bf616a"
            } "##),
            ("one-light", "One Light", r##" {
                "--bg": "#fafafa",
                "--bg-accent": "#f0f0f0",
                "--surface": "#ffffff",
                "--surface-soft": "#e5e5e6",
                "--ink": "#383a42",
                "--muted": "#a0a1a7",
                "--line": "#dbdbdc",
                "--accent": "#4078f2",
                "--accent-soft": "rgba(64, 120, 242, 0.1)",
                "--warning": "#e45649"
            } "##),
            ("solarized-light", "Solarized Light", r##" {
                "--bg": "#fdf6e3",
                "--bg-accent": "#eee8d5",
                "--surface": "#fdf6e3",
                "--surface-soft": "#eee8d5",
                "--ink": "#657b83",
                "--muted": "#93a1a1",
                "--line": "#d5c4a1",
                "--accent": "#268bd2",
                "--accent-soft": "rgba(38, 139, 210, 0.1)",
                "--warning": "#dc322f"
            } "##),
            ("paper-color", "PaperColor", r##" {
                "--bg": "#f5f5f5",
                "--bg-accent": "#eeeeee",
                "--surface": "#ffffff",
                "--surface-soft": "#e4e4e4",
                "--ink": "#444444",
                "--muted": "#878787",
                "--line": "#d0d0d0",
                "--accent": "#005f87",
                "--accent-soft": "rgba(0, 95, 135, 0.1)",
                "--warning": "#df0000"
            } "##),
            ("rose-pine-dawn", "Rosé Pine Dawn", r##" {
                "--bg": "#faf4ed",
                "--bg-accent": "#fffaf3",
                "--surface": "#f2e9e1",
                "--surface-soft": "#ebe1d7",
                "--ink": "#575279",
                "--muted": "#797593",
                "--line": "#dfdad9",
                "--accent": "#d7827e",
                "--accent-soft": "rgba(215, 130, 126, 0.1)",
                "--warning": "#b4637a"
            } "##),
        ];

        // First, clear old built-in themes to handle the removal of non-compliant ones
        conn.execute("DELETE FROM themes WHERE is_builtin = 1 AND id NOT IN ('github-dark', 'dracula', 'nord-dark')", [])?;

        for (id, name, config) in builtin_themes {
            conn.execute(
                "INSERT INTO themes (id, name, config, is_builtin) VALUES (?1, ?2, ?3, 1)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, config=excluded.config
                 ON CONFLICT(name) DO UPDATE SET id=excluded.id, config=excluded.config",
                [id, name, config],
            )?;
        }

        // 2. Check if we need to remove CHECK constraints from 'jobs' (for flexibility with Temporary/Internship/etc.)
    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='jobs'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if table_sql.contains("CHECK(employment_type IN") || table_sql.contains("CHECK(work_model IN") {
        println!("Migrating 'jobs' table to flexible schema...");

        // Disable foreign keys for the duration of the migration
        conn.execute("PRAGMA foreign_keys=OFF", [])?;

        let migration_result = (|| -> Result<()> {
            conn.execute("BEGIN TRANSACTION", [])?;

            // Drop triggers first to avoid issues with RENAME
            conn.execute("DROP TRIGGER IF EXISTS update_jobs_modtime", [])?;

            // Drop jobs_old if it exists from a previous failed attempt
            conn.execute("DROP TABLE IF EXISTS jobs_old", [])?;

            // Rename
            conn.execute("ALTER TABLE jobs RENAME TO jobs_old", [])?;

            // Create new table with flexible schema
            conn.execute(
                "CREATE TABLE jobs (
                    id TEXT PRIMARY KEY,
                    company_name TEXT NOT NULL,
                    job_title TEXT NOT NULL,
                    work_model TEXT DEFAULT 'Remote',
                    employment_type TEXT DEFAULT 'Full-time',
                    status TEXT CHECK(status IN ('Drafting', 'Applied', 'Interviewing', 'Offer', 'Rejected')) DEFAULT 'Drafting',
                    raw_jd TEXT NOT NULL,
                    requirements TEXT,
                    core_responsibilities TEXT,
                    custom_instruction TEXT,
                    reference_name TEXT,
                    reference_email TEXT,
                    social_link TEXT,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            // Dynamically identify common columns for data migration
            let old_columns: Vec<String> = conn
                .prepare("PRAGMA table_info(jobs_old)")?
                .query_map([], |row| row.get(1))?
                .collect::<Result<Vec<_>, _>>()?;

            let target_columns = [
                "id",
                "company_name",
                "job_title",
                "work_model",
                "employment_type",
                "status",
                "raw_jd",
                "requirements",
                "core_responsibilities",
                "custom_instruction",
                "reference_name",
                "reference_email",
                "social_link",
                "created_at",
                "updated_at",
            ];

            let common_columns: Vec<&str> = target_columns
                .iter()
                .filter(|&&c| old_columns.contains(&c.to_string()))
                .cloned()
                .collect();

            let cols_str = common_columns.join(", ");
            let insert_sql = format!(
                "INSERT INTO jobs ({}) SELECT {} FROM jobs_old",
                cols_str, cols_str
            );

            conn.execute(&insert_sql, [])?;

            // Drop old table
            conn.execute("DROP TABLE jobs_old", [])?;

            // Re-create the trigger for the new table
            conn.execute(
                "CREATE TRIGGER update_jobs_modtime 
                AFTER UPDATE ON jobs 
                BEGIN UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
                [],
            )?;

            conn.execute("COMMIT", [])?;
            Ok(())
        })();

        if let Err(e) = migration_result {
            println!("Migration failed, attempting rollback: {}", e);
            let _ = conn.execute("ROLLBACK", []);
            // If jobs_old exists and jobs doesn't, try to restore
            let jobs_exist: i32 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='jobs'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            let jobs_old_exist: i32 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='jobs_old'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            if jobs_exist == 0 && jobs_old_exist == 1 {
                let _ = conn.execute("ALTER TABLE jobs_old RENAME TO jobs", []);
            }
            return Err(e);
        }

        conn.execute("PRAGMA foreign_keys=ON", [])?;
    }

    // 2. Add missing columns to 'jobs' table (handles cases where migration wasn't triggered)
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(jobs)")?
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.contains(&"reference_name".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN reference_name TEXT", [])?;
    }
    if !columns.contains(&"reference_email".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN reference_email TEXT", [])?;
    }
    if !columns.contains(&"social_link".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN social_link TEXT", [])?;
    }
    if !columns.contains(&"custom_instruction".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN custom_instruction TEXT", [])?;
    }
    if !columns.contains(&"requirements".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN requirements TEXT", [])?;
    }
    if !columns.contains(&"core_responsibilities".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN core_responsibilities TEXT", [])?;
    }
    if !columns.contains(&"job_url".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN job_url TEXT", [])?;
    }
    if !columns.contains(&"salary".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN salary TEXT", [])?;
    }
    if !columns.contains(&"applied_date".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN applied_date TEXT", [])?;
    }
    if !columns.contains(&"interview_date".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN interview_date TEXT", [])?;
    }
    if !columns.contains(&"offer_date".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN offer_date TEXT", [])?;
    }
    if !columns.contains(&"rejected_date".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN rejected_date TEXT", [])?;
    }
    if !columns.contains(&"joining_date".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN joining_date TEXT", [])?;
    }

    // 3. Fix potential broken foreign keys in tailored_resumes (pointing to jobs_old)
    // This can happen if a previous migration renamed 'jobs' while foreign_keys was ON.
    let tailored_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='tailored_resumes'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if tailored_sql.contains("jobs_old") {
        println!("Fixing broken foreign key in 'tailored_resumes' table...");

        conn.execute("PRAGMA foreign_keys=OFF", [])?;

        let fix_result = (|| -> Result<()> {
            conn.execute("BEGIN TRANSACTION", [])?;

            conn.execute("DROP TRIGGER IF EXISTS update_tailored_resumes_modtime", [])?;
            conn.execute(
                "ALTER TABLE tailored_resumes RENAME TO tailored_resumes_old",
                [],
            )?;

            conn.execute(
                "CREATE TABLE tailored_resumes (
                    id TEXT PRIMARY KEY,
                    job_id TEXT NOT NULL,
                    base_resume_id TEXT NOT NULL,
                    final_latex_content TEXT NOT NULL,
                    is_active BOOLEAN DEFAULT 1,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (job_id) REFERENCES jobs(id),
                    FOREIGN KEY (base_resume_id) REFERENCES base_resumes(id)
                )",
                [],
            )?;

            conn.execute(
                "INSERT INTO tailored_resumes (
                    id, job_id, base_resume_id, final_latex_content, 
                    is_active, created_at, updated_at
                ) SELECT 
                    id, job_id, base_resume_id, final_latex_content, 
                    is_active, created_at, updated_at 
                FROM tailored_resumes_old",
                [],
            )?;

            conn.execute("DROP TABLE tailored_resumes_old", [])?;

            conn.execute(
                "CREATE TRIGGER update_tailored_resumes_modtime 
                AFTER UPDATE ON tailored_resumes 
                BEGIN UPDATE tailored_resumes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;",
                [],
            )?;

            conn.execute("COMMIT", [])?;
            Ok(())
        })();

        if let Err(e) = fix_result {
            println!("Failed to fix tailored_resumes: {}", e);
            let _ = conn.execute("ROLLBACK", []);
        }

        conn.execute("PRAGMA foreign_keys=ON", [])?;
    }

    // 4. Final cleanup: Drop jobs_old if it somehow still exists
    let jobs_old_exists: i32 = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='jobs_old'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if jobs_old_exists > 0 {
        println!("Cleaning up orphaned 'jobs_old' table...");
        let _ = conn.execute("DROP TABLE jobs_old", []);
    }

    // 5. Add base_resume_id and base_cl_id to 'jobs' table
    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='jobs'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if !table_sql.contains("base_resume_id") {
        println!("Adding 'base_resume_id' and 'base_cl_id' to 'jobs' table...");
        let _ = conn.execute("ALTER TABLE jobs ADD COLUMN base_resume_id TEXT REFERENCES base_resumes(id)", []);
        let _ = conn.execute("ALTER TABLE jobs ADD COLUMN base_cl_id TEXT REFERENCES base_cover_letters(id)", []);
    }

    // --- 4. Final Flexible Status Migration ---
    // Remove the rigid status CHECK constraint that blocks 'Joined' and other new statuses.
    let table_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='jobs'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if table_sql.contains("CHECK") || table_sql.contains("status IN") {
        println!("Performing final status constraint removal migration...");
        conn.execute_batch("
            PRAGMA foreign_keys=OFF;
            BEGIN TRANSACTION;
            
            -- Create the new flexible table
            CREATE TABLE jobs_final (
                id TEXT PRIMARY KEY,
                company_name TEXT NOT NULL,
                job_title TEXT NOT NULL,
                work_model TEXT NOT NULL,
                employment_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'Drafting',
                raw_jd TEXT NOT NULL,
                requirements TEXT,
                core_responsibilities TEXT,
                custom_instruction TEXT,
                reference_name TEXT,
                reference_email TEXT,
                social_link TEXT,
                job_url TEXT,
                base_resume_id TEXT REFERENCES base_resumes(id),
                base_cl_id TEXT REFERENCES base_cover_letters(id),
                salary TEXT,
                applied_date TEXT,
                interview_date TEXT,
                offer_date TEXT,
                rejected_date TEXT,
                joining_date TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Copy data using INSERT OR IGNORE and matching columns
            INSERT INTO jobs_final (
                id, company_name, job_title, work_model, employment_type, status, raw_jd, 
                requirements, core_responsibilities, custom_instruction, reference_name, 
                reference_email, social_link, job_url, base_resume_id, base_cl_id,
                salary, applied_date, interview_date, offer_date, rejected_date, joining_date,
                created_at, updated_at
            ) 
            SELECT 
                id, company_name, job_title, 
                COALESCE(work_model, 'Remote'), 
                COALESCE(employment_type, 'Full-time'), 
                status, raw_jd, 
                requirements, core_responsibilities, custom_instruction, reference_name, 
                reference_email, social_link, job_url, base_resume_id, base_cl_id,
                salary, applied_date, interview_date, offer_date, rejected_date, joining_date,
                created_at, updated_at
            FROM jobs;

            DROP TABLE jobs;
            ALTER TABLE jobs_final RENAME TO jobs;
            
            -- Re-create the trigger for the new table
            CREATE TRIGGER IF NOT EXISTS update_jobs_modtime_final 
            AFTER UPDATE ON jobs 
            BEGIN UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

            COMMIT;
            PRAGMA foreign_keys=ON;
        ")?;
    }

    Ok(conn)
}
