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

        -- 3. Jobs Table
        CREATE TABLE IF NOT EXISTS jobs (
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
            job_url TEXT,
            base_resume_id TEXT,
            base_cl_id TEXT,
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
            name TEXT NOT NULL,
            config TEXT NOT NULL,
            is_builtin BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "
        )?;

        // --- MIGRATIONS ---

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

    Ok(conn)
}
