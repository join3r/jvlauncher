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
    Agent,
}

impl AppType {
    pub fn as_str(&self) -> &str {
        match self {
            AppType::App => "app",
            AppType::Webapp => "webapp",
            AppType::Tui => "tui",
            AppType::Agent => "agent",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "webapp" => AppType::Webapp,
            "tui" => AppType::Tui,
            "agent" => AppType::Agent,
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
    pub open_external_links: Option<bool>,
    pub enable_oauth: Option<bool>,
    pub auto_close_timeout: Option<i32>,
    pub always_on_top: Option<bool>,
    pub hide_on_shortcut: Option<bool>,
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
    pub open_external_links: Option<bool>,
    pub enable_oauth: Option<bool>,
    pub auto_close_timeout: Option<i32>,
    pub always_on_top: Option<bool>,
    pub hide_on_shortcut: Option<bool>,
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
    pub separate_agent_apps: bool,
}

/// AI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    pub enabled: bool,
    pub endpoint_url: String,
    pub api_key: String,
    pub default_model: Option<String>,
    pub max_concurrent_agents: i32,
}

impl Default for AISettings {
    fn default() -> Self {
        AISettings {
            enabled: false,
            endpoint_url: "http://192.168.1.113:1234".to_string(),
            api_key: String::new(),
            default_model: None,
            max_concurrent_agents: 1,
        }
    }
}

/// AI Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub created: Option<i64>,
}

/// Agent app configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentApp {
    pub app_id: i64,
    pub model: Option<String>,
    pub prompt: String,
    pub tool_notification: bool,
    pub tool_website_scrape: bool,
    pub tool_run_command: bool,
    pub website_url: Option<String>,
    pub website_scrape_mode: Option<String>, // "text" or "visual"
    pub command: Option<String>,
}

/// AI Queue item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIQueueItem {
    pub id: i64,
    pub status: String, // pending, processing, completed, failed
    pub message: String,
    pub response: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub agent_name: Option<String>,
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub text: String,
    pub created_at: i64,
    pub dismissed: bool,
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
            separate_agent_apps: false,
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
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN open_external_links INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN enable_oauth INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN auto_close_timeout INTEGER", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN always_on_top INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE webapp_details ADD COLUMN hide_on_shortcut INTEGER DEFAULT 0", []);

    // Add always_on_top and hide_on_shortcut columns to app_details table if they don't exist
    let _ = conn.execute("ALTER TABLE app_details ADD COLUMN always_on_top INTEGER DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE app_details ADD COLUMN hide_on_shortcut INTEGER DEFAULT 0", []);

    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // AI models table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_models (
            id TEXT PRIMARY KEY,
            created INTEGER
        )",
        [],
    )?;

    // Agent apps table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_apps (
            app_id INTEGER PRIMARY KEY,
            model TEXT,
            prompt TEXT NOT NULL,
            tool_notification INTEGER DEFAULT 0,
            tool_website_scrape INTEGER DEFAULT 0,
            tool_run_command INTEGER DEFAULT 0,
            website_url TEXT,
            website_scrape_mode TEXT DEFAULT 'text',
            command TEXT,
            FOREIGN KEY(app_id) REFERENCES apps(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Add website_scrape_mode column if it doesn't exist (migration)
    let _ = conn.execute(
        "ALTER TABLE agent_apps ADD COLUMN website_scrape_mode TEXT DEFAULT 'text'",
        [],
    );

    // AI queue table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            status TEXT NOT NULL,
            message TEXT NOT NULL,
            response TEXT,
            created_at INTEGER NOT NULL,
            completed_at INTEGER,
            agent_name TEXT
        )",
        [],
    )?;

    // Add agent_name column if it doesn't exist (migration)
    let _ = conn.execute(
        "ALTER TABLE ai_queue ADD COLUMN agent_name TEXT",
        [],
    );

    // Notifications table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            dismissed INTEGER DEFAULT 0
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

    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('separate_agent_apps', ?1)",
        params![if default_settings.separate_agent_apps { "true" } else { "false" }],
    )?;

    // Initialize AI settings
    let default_ai_settings = AISettings::default();
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_enabled', ?1)",
        params![if default_ai_settings.enabled { "true" } else { "false" }],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_endpoint_url', ?1)",
        params![default_ai_settings.endpoint_url],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_api_key', ?1)",
        params![default_ai_settings.api_key],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('ai_max_concurrent_agents', ?1)",
        params![default_ai_settings.max_concurrent_agents.to_string()],
    )?;

    Ok(())
}

/// Get all apps from the database
pub fn get_all_apps(pool: &DbPool) -> Result<Vec<App>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.app_type, a.name, a.icon_path, a.position, a.shortcut, a.global_shortcut,
                ad.binary_path, ad.cli_params, ad.always_on_top as ad_always_on_top, ad.hide_on_shortcut as ad_hide_on_shortcut,
                wd.url, wd.session_data_path, wd.show_nav_controls, wd.open_external_links, wd.enable_oauth, wd.auto_close_timeout, wd.always_on_top as wd_always_on_top, wd.hide_on_shortcut as wd_hide_on_shortcut
         FROM apps a
         LEFT JOIN app_details ad ON a.id = ad.app_id
         LEFT JOIN webapp_details wd ON a.id = wd.app_id
         ORDER BY a.position"
    )?;

    let apps = stmt.query_map([], |row| {
        let show_nav_controls: Option<i32> = row.get(13).ok();
        let open_external_links: Option<i32> = row.get(14).ok();
        let enable_oauth: Option<i32> = row.get(15).ok();
        let auto_close_timeout: Option<i32> = row.get(16).ok();
        let ad_always_on_top: Option<i32> = row.get(9).ok();
        let wd_always_on_top: Option<i32> = row.get(17).ok();
        let ad_hide_on_shortcut: Option<i32> = row.get(10).ok();
        let wd_hide_on_shortcut: Option<i32> = row.get(18).ok();
        // Use webapp always_on_top if available, otherwise use app_details always_on_top
        let always_on_top = wd_always_on_top.or(ad_always_on_top).map(|v| v != 0);
        // Use webapp hide_on_shortcut if available, otherwise use app_details hide_on_shortcut
        let hide_on_shortcut = wd_hide_on_shortcut.or(ad_hide_on_shortcut).map(|v| v != 0);

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
            url: row.get(11)?,
            session_data_path: row.get(12)?,
            show_nav_controls: show_nav_controls.map(|v| v != 0),
            open_external_links: open_external_links.map(|v| v != 0),
            enable_oauth: enable_oauth.map(|v| v != 0),
            auto_close_timeout,
            always_on_top,
            hide_on_shortcut,
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
                let always_on_top = new_app.always_on_top.unwrap_or(false);
                let hide_on_shortcut = new_app.hide_on_shortcut.unwrap_or(false);
                conn.execute(
                    "INSERT INTO app_details (app_id, binary_path, cli_params, always_on_top, hide_on_shortcut)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![app_id, binary_path, new_app.cli_params, if always_on_top { 1 } else { 0 }, if hide_on_shortcut { 1 } else { 0 }],
                )?;
            }
        }
        AppType::Agent => {
            // Agent apps don't need app_details, they use agent_apps table
            // Agent configuration will be saved separately via save_agent_app
        }
        AppType::Webapp => {
            if let Some(url) = new_app.url {
                let session_path = session_dir
                    .unwrap_or_else(|| PathBuf::from(format!("webapp_{}", app_id)))
                    .to_string_lossy()
                    .to_string();

                let show_nav_controls = new_app.show_nav_controls.unwrap_or(false);
                let open_external_links = new_app.open_external_links.unwrap_or(false);
                let enable_oauth = new_app.enable_oauth.unwrap_or(false);
                let always_on_top = new_app.always_on_top.unwrap_or(false);
                let hide_on_shortcut = new_app.hide_on_shortcut.unwrap_or(false);

                conn.execute(
                    "INSERT INTO webapp_details (app_id, url, session_data_path, show_nav_controls, open_external_links, enable_oauth, auto_close_timeout, always_on_top, hide_on_shortcut)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        app_id,
                        url,
                        session_path,
                        if show_nav_controls { 1 } else { 0 },
                        if open_external_links { 1 } else { 0 },
                        if enable_oauth { 1 } else { 0 },
                        new_app.auto_close_timeout,
                        if always_on_top { 1 } else { 0 },
                        if hide_on_shortcut { 1 } else { 0 }
                    ],
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
            let always_on_top = app.always_on_top.unwrap_or(false);
            let hide_on_shortcut = app.hide_on_shortcut.unwrap_or(false);
            conn.execute(
                "UPDATE app_details SET binary_path = ?1, cli_params = ?2, always_on_top = ?3, hide_on_shortcut = ?4
                 WHERE app_id = ?5",
                params![app.binary_path, app.cli_params, if always_on_top { 1 } else { 0 }, if hide_on_shortcut { 1 } else { 0 }, app.id],
            )?;
        }
        AppType::Agent => {
            // Agent apps don't need app_details updates
            // Agent configuration is updated separately via save_agent_app
        }
        AppType::Webapp => {
            let show_nav_controls = app.show_nav_controls.unwrap_or(false);
            let open_external_links = app.open_external_links.unwrap_or(false);
            let enable_oauth = app.enable_oauth.unwrap_or(false);
            let always_on_top = app.always_on_top.unwrap_or(false);
            let hide_on_shortcut = app.hide_on_shortcut.unwrap_or(false);
            conn.execute(
                "UPDATE webapp_details SET url = ?1, show_nav_controls = ?2, open_external_links = ?3, enable_oauth = ?4, auto_close_timeout = ?5, always_on_top = ?6, hide_on_shortcut = ?7
                 WHERE app_id = ?8",
                params![
                    app.url,
                    if show_nav_controls { 1 } else { 0 },
                    if open_external_links { 1 } else { 0 },
                    if enable_oauth { 1 } else { 0 },
                    app.auto_close_timeout,
                    if always_on_top { 1 } else { 0 },
                    if hide_on_shortcut { 1 } else { 0 },
                    app.id
                ],
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

    let separate_agent_apps: bool = conn.query_row(
        "SELECT value FROM settings WHERE key = 'separate_agent_apps'",
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
        separate_agent_apps,
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

/// Get AI settings
pub fn get_ai_settings(pool: &DbPool) -> Result<AISettings> {
    let conn = pool.get()?;
    
    let enabled: bool = conn.query_row(
        "SELECT value FROM settings WHERE key = 'ai_enabled'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "false".to_string()) == "true";

    let endpoint_url: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'ai_endpoint_url'",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| "http://192.168.1.113:1234".to_string());

    let api_key: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'ai_api_key'",
        [],
        |row| row.get(0),
    ).unwrap_or_else(|_| String::new());

    let default_model: Option<String> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'ai_default_model'",
        [],
        |row| row.get(0),
    ).ok();

    let max_concurrent_agents: i32 = conn.query_row(
        "SELECT value FROM settings WHERE key = 'ai_max_concurrent_agents'",
        [],
        |row| row.get::<_, String>(0),
    ).unwrap_or_else(|_| "1".to_string())
    .parse()
    .unwrap_or(1);

    Ok(AISettings {
        enabled,
        endpoint_url,
        api_key,
        default_model,
        max_concurrent_agents,
    })
}

/// Update AI setting
pub fn update_ai_setting(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![format!("ai_{}", key), value],
    )?;
    Ok(())
}

/// Set default model
pub fn set_default_model(pool: &DbPool, model_id: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_default_model', ?1)",
        params![model_id],
    )?;
    Ok(())
}

/// Get all AI models
pub fn get_models(pool: &DbPool) -> Result<Vec<AIModel>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, created FROM ai_models ORDER BY id")?;
    
    let models = stmt.query_map([], |row| {
        Ok(AIModel {
            id: row.get(0)?,
            created: row.get(1)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(models)
}

/// Save AI models
pub fn save_models(pool: &DbPool, models: Vec<AIModel>) -> Result<()> {
    let conn = pool.get()?;

    // Clear existing models
    conn.execute("DELETE FROM ai_models", [])?;

    // Insert new models
    for model in models {
        conn.execute(
            "INSERT INTO ai_models (id, created) VALUES (?1, ?2)",
            params![model.id, model.created],
        )?;
    }

    Ok(())
}

/// Get agent app configuration
pub fn get_agent_app(pool: &DbPool, app_id: i64) -> Result<Option<AgentApp>> {
    let conn = pool.get()?;
    
    let result = conn.query_row(
        "SELECT app_id, model, prompt, tool_notification, tool_website_scrape, tool_run_command, website_url, website_scrape_mode, command
         FROM agent_apps WHERE app_id = ?1",
        params![app_id],
        |row| {
            Ok(AgentApp {
                app_id: row.get(0)?,
                model: row.get(1)?,
                prompt: row.get(2)?,
                tool_notification: row.get::<_, i32>(3)? != 0,
                tool_website_scrape: row.get::<_, i32>(4)? != 0,
                tool_run_command: row.get::<_, i32>(5)? != 0,
                website_url: row.get(6)?,
                website_scrape_mode: row.get(7)?,
                command: row.get(8)?,
            })
        },
    );
    
    match result {
        Ok(agent) => Ok(Some(agent)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Save agent app configuration
pub fn save_agent_app(pool: &DbPool, agent: &AgentApp) -> Result<()> {
    let conn = pool.get()?;
    
    conn.execute(
        "INSERT OR REPLACE INTO agent_apps (app_id, model, prompt, tool_notification, tool_website_scrape, tool_run_command, website_url, website_scrape_mode, command)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            agent.app_id,
            agent.model,
            agent.prompt,
            if agent.tool_notification { 1 } else { 0 },
            if agent.tool_website_scrape { 1 } else { 0 },
            if agent.tool_run_command { 1 } else { 0 },
            agent.website_url,
            agent.website_scrape_mode,
            agent.command,
        ],
    )?;
    
    Ok(())
}

/// Add item to AI queue
pub fn add_queue_item(pool: &DbPool, message: &str, agent_name: Option<&str>) -> Result<i64> {
    let conn = pool.get()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT INTO ai_queue (status, message, created_at, agent_name) VALUES ('pending', ?1, ?2, ?3)",
        params![message, timestamp, agent_name],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Update queue item status
pub fn update_queue_item_status(pool: &DbPool, id: i64, status: &str, response: Option<&str>) -> Result<()> {
    let conn = pool.get()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    if let Some(resp) = response {
        conn.execute(
            "UPDATE ai_queue SET status = ?1, response = ?2, completed_at = ?3 WHERE id = ?4",
            params![status, resp, timestamp, id],
        )?;
    } else {
        conn.execute(
            "UPDATE ai_queue SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
    }
    
    Ok(())
}

/// Get AI queue items
pub fn get_queue_items(pool: &DbPool) -> Result<Vec<AIQueueItem>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, status, message, response, created_at, completed_at, agent_name FROM ai_queue ORDER BY created_at DESC LIMIT 100"
    )?;

    let items = stmt.query_map([], |row| {
        Ok(AIQueueItem {
            id: row.get(0)?,
            status: row.get(1)?,
            message: row.get(2)?,
            response: row.get(3)?,
            created_at: row.get(4)?,
            completed_at: row.get(5)?,
            agent_name: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

/// Get queue item by ID
pub fn get_queue_item(pool: &DbPool, id: i64) -> Result<Option<AIQueueItem>> {
    let conn = pool.get()?;

    let result = conn.query_row(
        "SELECT id, status, message, response, created_at, completed_at, agent_name FROM ai_queue WHERE id = ?1",
        params![id],
        |row| {
            Ok(AIQueueItem {
                id: row.get(0)?,
                status: row.get(1)?,
                message: row.get(2)?,
                response: row.get(3)?,
                created_at: row.get(4)?,
                completed_at: row.get(5)?,
                agent_name: row.get(6)?,
            })
        },
    );

    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Clear finished queue items (completed and failed)
pub fn clear_finished_queue_items(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM ai_queue WHERE status IN ('completed', 'failed')",
        [],
    )?;
    Ok(())
}

/// Create notification
pub fn create_notification(pool: &DbPool, text: &str) -> Result<i64> {
    let conn = pool.get()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    conn.execute(
        "INSERT INTO notifications (text, created_at, dismissed) VALUES (?1, ?2, 0)",
        params![text, timestamp],
    )?;
    
    Ok(conn.last_insert_rowid())
}

/// Get notifications
pub fn get_notifications(pool: &DbPool, include_dismissed: bool) -> Result<Vec<Notification>> {
    let conn = pool.get()?;
    let query = if include_dismissed {
        "SELECT id, text, created_at, dismissed FROM notifications ORDER BY created_at DESC"
    } else {
        "SELECT id, text, created_at, dismissed FROM notifications WHERE dismissed = 0 ORDER BY created_at DESC"
    };
    
    let mut stmt = conn.prepare(query)?;
    
    let notifications = stmt.query_map([], |row| {
        Ok(Notification {
            id: row.get(0)?,
            text: row.get(1)?,
            created_at: row.get(2)?,
            dismissed: row.get::<_, i32>(3)? != 0,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    
    Ok(notifications)
}

/// Dismiss notification
pub fn dismiss_notification(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE notifications SET dismissed = 1 WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Dismiss all notifications
pub fn dismiss_all_notifications(pool: &DbPool) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("UPDATE notifications SET dismissed = 1", [])?;
    Ok(())
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

