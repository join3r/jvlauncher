use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type alias for database connection pool
pub type DbPool = Pool<SqliteConnectionManager>;

/// Represents the type of application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    App,
    Webapp,
    Tui,
}

impl AppType {
    pub fn as_str(&self) -> &str {
        match self {
            AppType::App => "app",
            AppType::Webapp => "webapp",
            AppType::Tui => "tui",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "webapp" => AppType::Webapp,
            "tui" => AppType::Tui,
            _ => AppType::App,
        }
    }
}

/// Represents window position and size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Represents an application in the launcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: i64,
    pub app_type: AppType,
    pub name: String,
    pub icon_path: Option<String>,
    pub position: i32,
    pub shortcut: Option<String>,
    pub global_shortcut: Option<String>,
    // Details specific to app type
    pub binary_path: Option<String>,
    pub cli_params: Option<String>,
    pub url: Option<String>,
    pub session_data_path: Option<String>,
    pub show_nav_controls: Option<bool>,
}

/// Data for creating a new app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewApp {
    pub app_type: AppType,
    pub name: String,
    pub icon_path: Option<String>,
    pub shortcut: Option<String>,
    pub global_shortcut: Option<String>,
    pub binary_path: Option<String>,
    pub cli_params: Option<String>,
    pub url: Option<String>,
    pub show_nav_controls: Option<bool>,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub global_shortcut: String,
    pub theme: String,
    pub grid_cols: i32,
    pub grid_rows: i32,
    pub start_at_login: bool,
    pub terminal_command: Option<String>,
    pub hide_app_names: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            global_shortcut: "CommandOrControl+Shift+Space".to_string(),
            theme: "system".to_string(),
            grid_cols: 4,
            grid_rows: 3,
            start_at_login: false,
            terminal_command: None,
            hide_app_names: false,
        }
    }
}

/// Initialize the database with schema
pub fn init_database(db_path: PathBuf) -> Result<DbPool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(manager)?;
    
    let conn = pool.get()?;
    create_schema(&conn)?;
    initialize_settings(&conn)?;
    
    Ok(pool)
}

/// Create database schema
fn create_schema(conn: &Connection) -> Result<()> {
    // Apps table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_type TEXT NOT NULL,
            name TEXT NOT NULL,
            icon_path TEXT,
            position INTEGER NOT NULL,
            shortcut TEXT,
            global_shortcut TEXT
        )",
        [],
    )?;

    // Add global_shortcut column to existing apps table if it doesn't exist
    let _ = conn.execute("ALTER TABLE apps ADD COLUMN global_shortcut TEXT", []);

    // App details table (for native apps and TUI)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_details (
            app_id INTEGER PRIMARY KEY,
            binary_path TEXT NOT NULL,
            cli_params TEXT,
            FOREIGN KEY(app_id) REFERENCES apps(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Webapp details table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS webapp_details (
            app_id INTEGER PRIMARY KEY,
            url TEXT NOT NULL,
            session_data_path TEXT NOT NULL,
            window_x INTEGER,
            window_y INTEGER,
            window_width INTEGER,
            window_height INTEGER,
            FOREIGN KEY(app_id) REFERENCES apps(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Add columns to existing webapp_details table if they don't exist
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN window_x INTEGER", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN window_y INTEGER", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN window_width INTEGER", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN window_height INTEGER", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN show_nav_controls INTEGER DEFAULT 0", []);

    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

/// Initialize default settings if they don't exist
fn initialize_settings(conn: &Connection) -> Result<()> {
    let default_settings = Settings::default();

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('global_shortcut', ?1)",
        params![default_settings.global_shortcut],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', ?1)",
        params![default_settings.theme],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('grid_cols', ?1)",
        params![default_settings.grid_cols.to_string()],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('grid_rows', ?1)",
        params![default_settings.grid_rows.to_string()],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('start_at_login', ?1)",
        params![if default_settings.start_at_login { "true" } else { "false" }],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('hide_app_names', ?1)",
        params![if default_settings.hide_app_names { "true" } else { "false" }],
    )?;

    Ok(())
}

/// Get all apps from the database
pub fn get_all_apps(pool: &DbPool) -> Result<Vec<App>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.app_type, a.name, a.icon_path, a.position, a.shortcut, a.global_shortcut,
                ad.binary_path, ad.cli_params,
                wd.url, wd.session_data_path, wd.show_nav_controls
         FROM apps a
         LEFT JOIN app_details ad ON a.id = ad.app_id
         LEFT JOIN webapp_details wd ON a.id = wd.app_id
         ORDER BY a.position"
    )?;

    let apps = stmt.query_map([], |row| {
        let show_nav_controls: Option<i32> = row.get(11).ok();
        Ok(App {
            id: row.get(0)?,
            app_type: AppType::from_str(&row.get::<_, String>(1)?),
            name: row.get(2)?,
            icon_path: row.get(3)?,
            position: row.get(4)?,
            shortcut: row.get(5)?,
            global_shortcut: row.get(6)?,
            binary_path: row.get(7)?,
            cli_params: row.get(8)?,
            url: row.get(9)?,
            session_data_path: row.get(10)?,
            show_nav_controls: show_nav_controls.map(|v| v != 0),
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(apps)
}

/// Create a new app
pub fn create_app(pool: &DbPool, new_app: NewApp, session_dir: Option<PathBuf>) -> Result<i64> {
    let conn = pool.get()?;
    
    // Get the next position
    let position: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM apps",
        [],
        |row| row.get(0),
    )?;

    // Insert into apps table
    conn.execute(
        "INSERT INTO apps (app_type, name, icon_path, position, shortcut, global_shortcut)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_app.app_type.as_str(),
            new_app.name,
            new_app.icon_path,
            position,
            new_app.shortcut,
            new_app.global_shortcut,
        ],
    )?;

    let app_id = conn.last_insert_rowid();

    // Insert type-specific details
    match new_app.app_type {
        AppType::App | AppType::Tui => {
            if let Some(binary_path) = new_app.binary_path {
                conn.execute(
                    "INSERT INTO app_details (app_id, binary_path, cli_params)
                     VALUES (?1, ?2, ?3)",
                    params![app_id, binary_path, new_app.cli_params],
                )?;
            }
        }
        AppType::Webapp => {
            if let Some(url) = new_app.url {
                let session_path = session_dir
                    .unwrap_or_else(|| PathBuf::from(format!("webapp_{}", app_id)))
                    .to_string_lossy()
                    .to_string();

                let show_nav_controls = new_app.show_nav_controls.unwrap_or(false);

                conn.execute(
                    "INSERT INTO webapp_details (app_id, url, session_data_path, show_nav_controls)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![app_id, url, session_path, if show_nav_controls { 1 } else { 0 }],
                )?;
            }
        }
    }

    Ok(app_id)
}

/// Update an existing app
pub fn update_app(pool: &DbPool, app: App) -> Result<()> {
    let conn = pool.get()?;

    conn.execute(
        "UPDATE apps SET name = ?1, icon_path = ?2, shortcut = ?3, global_shortcut = ?4
         WHERE id = ?5",
        params![app.name, app.icon_path, app.shortcut, app.global_shortcut, app.id],
    )?;

    // Update type-specific details
    match app.app_type {
        AppType::App | AppType::Tui => {
            conn.execute(
                "UPDATE app_details SET binary_path = ?1, cli_params = ?2
                 WHERE app_id = ?3",
                params![app.binary_path, app.cli_params, app.id],
            )?;
        }
        AppType::Webapp => {
            let show_nav_controls = app.show_nav_controls.unwrap_or(false);
            conn.execute(
                "UPDATE webapp_details SET url = ?1, show_nav_controls = ?2
                 WHERE app_id = ?3",
                params![app.url, if show_nav_controls { 1 } else { 0 }, app.id],
            )?;
        }
    }

    Ok(())
}

/// Delete an app
pub fn delete_app(pool: &DbPool, app_id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM apps WHERE id = ?1", params![app_id])?;
    Ok(())
}

/// Reorder apps by updating their positions
pub fn reorder_apps(pool: &DbPool, app_ids: Vec<i64>) -> Result<()> {
    let conn = pool.get()?;
    
    for (position, app_id) in app_ids.iter().enumerate() {
        conn.execute(
            "UPDATE apps SET position = ?1 WHERE id = ?2",
            params![position as i32, app_id],
        )?;
    }
    
    Ok(())
}

/// Get all settings
pub fn get_settings(pool: &DbPool) -> Result<Settings> {
    let conn = pool.get()?;

    let global_shortcut: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'global_shortcut'",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "CommandOrControl+Shift+Space".to_string());

    let theme: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'theme'",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "system".to_string());

    let grid_cols: i32 = conn.query_row(
        "SELECT value FROM settings WHERE key = 'grid_cols'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "4".to_string())
    .parse()
    .unwrap_or(4);

    let grid_rows: i32 = conn.query_row(
        "SELECT value FROM settings WHERE key = 'grid_rows'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "3".to_string())
    .parse()
    .unwrap_or(3);

    let start_at_login: bool = conn.query_row(
        "SELECT value FROM settings WHERE key = 'start_at_login'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "false".to_string()) == "true";

    let terminal_command: Option<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'terminal_command'",
        [],
        |row| row.get(0),
    ).ok();

    let hide_app_names: bool = conn.query_row(
        "SELECT value FROM settings WHERE key = 'hide_app_names'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "false".to_string()) == "true";

    Ok(Settings {
        global_shortcut,
        theme,
        grid_cols,
        grid_rows,
        start_at_login,
        terminal_command,
        hide_app_names,
    })
}

/// Update a single setting
pub fn update_setting(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    let conn = pool.get()?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;

    Ok(())
}

/// Save window state for a webapp
pub fn save_window_state(pool: &DbPool, app_id: i64, state: &WindowState) -> Result<()> {
    let conn = pool.get()?;

    conn.execute(
        "UPDATE webapp_details SET window_x = ?1, window_y = ?2, window_width = ?3, window_height = ?4
         WHERE app_id = ?5",
        params![state.x, state.y, state.width, state.height, app_id],
    )?;

    Ok(())
}

/// Load window state for a webapp
pub fn load_window_state(pool: &DbPool, app_id: i64) -> Result<Option<WindowState>> {
    let conn = pool.get()?;

    let result = conn.query_row(
        "SELECT window_x, window_y, window_width, window_height FROM webapp_details WHERE app_id = ?1",
        params![app_id],
        |row| {
            let x: Option<i32> = row.get(0)?;
            let y: Option<i32> = row.get(1)?;
            let width: Option<i32> = row.get(2)?;
            let height: Option<i32> = row.get(3)?;

            // Only return WindowState if all values are present
            match (x, y, width, height) {
                (Some(x), Some(y), Some(w), Some(h)) => Ok(Some(WindowState { x, y, width: w, height: h })),
                _ => Ok(None),
            }
        },
    );

    match result {
        Ok(state) => Ok(state),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_state_serialization() {
        let state = WindowState {
            x: 100,
            y: 200,
            width: 1200,
            height: 800,
        };

        assert_eq!(state.x, 100);
        assert_eq!(state.y, 200);
        assert_eq!(state.width, 1200);
        assert_eq!(state.height, 800);
    }

    #[test]
    fn test_window_state_clone() {
        let state1 = WindowState {
            x: 100,
            y: 200,
            width: 1200,
            height: 800,
        };

        let state2 = state1.clone();

        assert_eq!(state1.x, state2.x);
        assert_eq!(state1.y, state2.y);
        assert_eq!(state1.width, state2.width);
        assert_eq!(state1.height, state2.height);
    }
}

