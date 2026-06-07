# 牛马人（NiuMaRen）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows desktop app that manages weekend duty rotation, sends notification emails 2 days ahead via SMTP, and skips Chinese holidays.

**Architecture:** Tauri 2.x desktop app with React/TypeScript frontend and Rust backend. SQLite for persistence, lettre for SMTP, tokio for scheduling, timor.tech API for holidays. System tray resident with auto-start.

**Tech Stack:** Tauri 2.x, React 18, TypeScript, Tailwind CSS, SQLite (rusqlite), lettre, tokio, reqwest, chrono

---

## File Structure Map

```
niumaren/
├── src-tauri/
│   ├── Cargo.toml              # Rust deps
│   ├── tauri.conf.json         # Tauri config (window, tray, plugins)
│   ├── build.rs
│   ├── icons/                  # App icons
│   └── src/
│       ├── main.rs             # Entry: tray setup, plugin init, scheduler start
│       ├── models.rs           # Serde structs: Personnel, Schedule, EmailLog, Settings
│       ├── db.rs               # SQLite init, migrations, query functions
│       ├── commands.rs         # #[tauri::command] fns — all IPC handlers
│       ├── email.rs            # SMTP sender using lettre
│       ├── holiday.rs          # Holiday API client + cache
│       └── scheduler.rs        # Tokio interval timer, duty check logic
├── src/
│   ├── main.tsx                # React DOM entry
│   ├── App.tsx                 # Tab layout + status bar
│   ├── App.css                 # Tailwind base styles
│   ├── types/
│   │   └── index.ts            # TS interfaces matching Rust models
│   ├── hooks/
│   │   └── useTauriInvoke.ts   # Generic invoke<T> wrapper with error handling
│   └── components/
│       ├── PersonnelTab.tsx    # CRUD table for duty personnel
│       ├── EmailConfigTab.tsx  # SMTP settings form + test send
│       ├── TemplateTab.tsx     # Email template editor with variable preview
│       ├── ScheduleTab.tsx     # Duty schedule table view
│       └── LogTab.tsx          # Email send log with status filter
├── index.html                  # Vite entry HTML
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.js
└── postcss.config.js
```

---

### Task 1: Scaffold Tauri + React Project

**Files:**
- Create: entire project skeleton via `create-tauri-app`
- Create: `src/types/index.ts`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Create the Tauri project**

```bash
cd D:/Develope/niumaren
npm create tauri-app@latest . -- --template react-ts
```

Expected: Interactive prompts. Choose:
- Project name: `niumaren`
- Frontend: `React` with `TypeScript`
- Package manager: `npm`

If the interactive mode doesn't work, use:
```bash
cd D:/Develope/niumaren
npm create tauri-app@latest niumaren-temp -- --template react-ts --manager npm
# Then move contents into D:/Develope/niumaren
```

- [ ] **Step 2: Install frontend dependencies**

```bash
cd D:/Develope/niumaren
npm install
npm install -D tailwindcss @tailwindcss/vite postcss
npm install lucide-react
```

- [ ] **Step 3: Configure Tailwind CSS**

Write `tailwind.config.js`:
```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

Write `postcss.config.js`:
```js
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
};
```

- [ ] **Step 4: Write TypeScript types**

Write `src/types/index.ts`:
```typescript
export interface Personnel {
  id: number;
  name: string;
  email: string;
  sort_order: number;
  active: number; // SQLite uses 0/1
  created_at: string;
}

export interface Schedule {
  id: number;
  person_id: number;
  person_name?: string; // joined field
  duty_date: string; // YYYY-MM-DD
  is_holiday: number;
  notified: number;
  notified_at: string | null;
  created_at: string;
}

export interface EmailLog {
  id: number;
  schedule_id: number;
  recipient: string;
  subject: string;
  status: 'success' | 'failed';
  error_msg: string | null;
  sent_at: string;
}

export interface SmtpSettings {
  smtp_host: string;
  smtp_port: number;
  smtp_username: string;
  smtp_password: string;
  smtp_use_tls: boolean;
  sender_name: string;
}

export interface EmailTemplate {
  subject_template: string;
  body_template: string;
}

export interface DutyNotification {
  person: Personnel;
  duty_date: string;
  day_of_week: string;
  next_person: Personnel | null;
  next_date: string | null;
}
```

- [ ] **Step 5: Add Rust dependencies**

Edit `src-tauri/Cargo.toml`, add to `[dependencies]`:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
lettre = { version = "0.11", features = ["tokio1-rustls-tls", "builder"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 6: Configure tauri.conf.json**

Edit `src-tauri/tauri.conf.json`, ensure:
```json
{
  "productName": "牛马人",
  "identifier": "com.niumaren.app",
  "app": {
    "windows": [
      {
        "title": "牛马人 · 值班助手",
        "width": 900,
        "height": 640,
        "resizable": true,
        "visible": true
      }
    ],
    "withGlobalTauri": true
  }
}
```

- [ ] **Step 7: Verify scaffold builds**

```bash
cd D:/Develope/niumaren
npm run tauri dev
```

Expected: Window opens with default React template. Close it.

- [ ] **Step 8: Commit scaffold**

```bash
cd D:/Develope/niumaren
git add -A
git commit -m "feat: scaffold Tauri + React + TS project with deps

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Rust Data Models + Database Init

**Files:**
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: Write models.rs**

Write `src-tauri/src/models.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personnel {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub sort_order: i32,
    pub active: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Option<i64>,
    pub person_id: i64,
    pub person_name: Option<String>,
    pub duty_date: String,
    pub is_holiday: i32,
    pub notified: i32,
    pub notified_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    pub id: Option<i64>,
    pub schedule_id: i64,
    pub recipient: String,
    pub subject: String,
    pub status: String,
    pub error_msg: Option<String>,
    pub sent_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayInfo {
    pub date: String,
    pub name: String,
    pub is_holiday: bool,
}
```

- [ ] **Step 2: Write db.rs — connection and migrations**

Write `src-tauri/src/db.rs`:
```rust
use rusqlite::{Connection, Result, params};
use std::sync::Mutex;
use std::path::PathBuf;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("niumaren.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS personnel (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );

            CREATE TABLE IF NOT EXISTS schedule (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                person_id INTEGER NOT NULL,
                duty_date TEXT NOT NULL,
                is_holiday INTEGER NOT NULL DEFAULT 0,
                notified INTEGER NOT NULL DEFAULT 0,
                notified_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                FOREIGN KEY (person_id) REFERENCES personnel(id)
            );

            CREATE TABLE IF NOT EXISTS email_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                schedule_id INTEGER NOT NULL,
                recipient TEXT NOT NULL,
                subject TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                error_msg TEXT,
                sent_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                FOREIGN KEY (schedule_id) REFERENCES schedule(id)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;

        // Seed default settings if empty
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'last_person_index'",
            [], |r| r.get(0)
        ).unwrap_or(0);

        if count == 0 {
            conn.execute_batch(
                "INSERT OR IGNORE INTO settings (key, value) VALUES
                    ('last_person_index', '-1'),
                    ('email_subject_template', '【值班通知】{日期} {星期} 值班提醒'),
                    ('email_body_template', 'Hi {姓名}：\n\n本周末 {日期}（{星期}）由你值班，请注意以下事项：\n\n📍 值班时间：{日期} 9:00-18:00\n📞 请保持电话畅通\n💻 处理线上问题和紧急需求\n\n下一位值班：{下一位姓名}（{下一位日期}）\n\n祝工作愉快！\n牛马人值班系统'),
                    ('smtp_host', ''),
                    ('smtp_port', '465'),
                    ('smtp_username', ''),
                    ('smtp_password', ''),
                    ('smtp_use_tls', 'true'),
                    ('sender_name', '值班系统'),
                    ('auto_start', 'true'),
                    ('holiday_cache', '{}'),
                    ('holiday_cache_year', '0');"
            )?;
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Compilation succeeds, warnings only.

- [ ] **Step 4: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/models.rs src-tauri/src/db.rs src-tauri/Cargo.toml src/types/index.ts
git commit -m "feat: add data models and SQLite database init

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: Personnel CRUD Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add personnel query functions to db.rs**

Append to `src-tauri/src/db.rs`:
```rust
use crate::models::Personnel;

impl Database {
    pub fn get_all_personnel(&self) -> Result<Vec<Personnel>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, sort_order, active, created_at
             FROM personnel ORDER BY sort_order ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Personnel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                email: row.get(2)?,
                sort_order: row.get(3)?,
                active: row.get(4)?,
                created_at: Some(row.get(5)?),
            })
        })?;
        rows.collect()
    }

    pub fn add_personnel(&self, name: &str, email: &str) -> Result<Personnel> {
        let conn = self.conn.lock().unwrap();
        let max_order: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM personnel",
            [], |r| r.get(0)
        )?;
        conn.execute(
            "INSERT INTO personnel (name, email, sort_order) VALUES (?1, ?2, ?3)",
            params![name, email, max_order + 1]
        )?;
        let id = conn.last_insert_rowid();
        Ok(Personnel {
            id: Some(id),
            name: name.to_string(),
            email: email.to_string(),
            sort_order: max_order + 1,
            active: 1,
            created_at: None,
        })
    }

    pub fn update_personnel(&self, p: &Personnel) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE personnel SET name=?1, email=?2, sort_order=?3, active=?4 WHERE id=?5",
            params![p.name, p.email, p.sort_order, p.active, p.id]
        )?;
        Ok(())
    }

    pub fn delete_personnel(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM personnel WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn reorder_personnel(&self, ids: &[i64]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE personnel SET sort_order=?1 WHERE id=?2",
                params![i as i32, id]
            )?;
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Write IPC commands in commands.rs**

Write `src-tauri/src/commands.rs`:
```rust
use tauri::State;
use crate::db::Database;
use crate::models::Personnel;

#[tauri::command]
pub fn get_personnel(db: State<Database>) -> Result<Vec<Personnel>, String> {
    db.get_all_personnel().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_personnel(db: State<Database>, name: String, email: String) -> Result<Personnel, String> {
    // Validate email format
    if !email.contains('@') || !email.contains('.') {
        return Err("邮箱格式不正确".to_string());
    }
    if name.trim().is_empty() {
        return Err("姓名不能为空".to_string());
    }
    db.add_personnel(&name.trim(), &email.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_personnel(db: State<Database>, personnel: Personnel) -> Result<(), String> {
    if !personnel.email.contains('@') || !personnel.email.contains('.') {
        return Err("邮箱格式不正确".to_string());
    }
    if personnel.name.trim().is_empty() {
        return Err("姓名不能为空".to_string());
    }
    db.update_personnel(&personnel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_personnel(db: State<Database>, id: i64) -> Result<(), String> {
    db.delete_personnel(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_personnel(db: State<Database>, ids: Vec<i64>) -> Result<(), String> {
    db.reorder_personnel(&ids).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Wire up main.rs — register commands and DB state**

Write `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod db;
mod commands;

use db::Database;
use std::path::PathBuf;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            let database = Database::new(app_dir)
                .expect("Failed to initialize database");
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_personnel,
            commands::add_personnel,
            commands::update_personnel,
            commands::delete_personnel,
            commands::reorder_personnel,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Compilation success.

- [ ] **Step 5: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/
git commit -m "feat: add personnel CRUD commands and main entry

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: Settings Commands

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Add settings query functions to db.rs**

Append to `impl Database` in `src-tauri/src/db.rs`:
```rust
use crate::models::Setting;

impl Database {
    pub fn get_setting(&self, key: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT value FROM settings WHERE key=?1",
            params![key],
            |r| r.get(0)
        )
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value]
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<Vec<Setting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })?;
        rows.collect()
    }
}
```

- [ ] **Step 2: Add settings IPC commands**

Append to `src-tauri/src/commands.rs`:
```rust
use crate::models::Setting;

#[tauri::command]
pub fn get_settings(db: State<Database>) -> Result<Vec<Setting>, String> {
    db.get_all_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(db: State<Database>, settings: Vec<Setting>) -> Result<(), String> {
    for s in settings {
        db.set_setting(&s.key, &s.value).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_setting(db: State<Database>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Register new commands in main.rs**

Edit `src-tauri/src/main.rs`, update `invoke_handler`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::get_personnel,
    commands::add_personnel,
    commands::update_personnel,
    commands::delete_personnel,
    commands::reorder_personnel,
    commands::get_settings,
    commands::save_settings,
    commands::save_setting,
])
```

- [ ] **Step 4: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/
git commit -m "feat: add settings CRUD commands

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: Email Sending Module

**Files:**
- Create: `src-tauri/src/email.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Write email.rs**

Write `src-tauri/src/email.rs`:
```rust
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use lettre::message::header::ContentType;

pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub sender_name: String,
}

pub async fn send_email(
    config: &SmtpConfig,
    to_email: &str,
    to_name: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let from_email = &config.username;
    let from_display = format!("{} <{}>", config.sender_name, from_email);
    let to_display = format!("{} <{}>", to_name, to_email);

    let email = Message::builder()
        .from(from_display.parse().map_err(|e| format!("发件人格式错误: {}", e))?)
        .to(to_display.parse().map_err(|e| format!("收件人格式错误: {}", e))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("邮件构建失败: {}", e))?;

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let mailer = if config.use_tls {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| format!("SMTP 配置错误: {}", e))?
            .port(config.port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| format!("SMTP 配置错误: {}", e))?
            .port(config.port)
            .credentials(creds)
            .build()
    };

    mailer.send(email).await.map_err(|e| format!("邮件发送失败: {}", e))?;
    Ok(())
}

pub fn replace_template_vars(
    template: &str,
    name: &str,
    email: &str,
    date: &str,
    day_of_week: &str,
    next_name: Option<&str>,
    next_date: Option<&str>,
) -> String {
    let mut result = template
        .replace("{姓名}", name)
        .replace("{邮箱}", email)
        .replace("{日期}", date)
        .replace("{星期}", day_of_week);
    result = result.replace("{下一位姓名}", next_name.unwrap_or("无"));
    result = result.replace("{下一位日期}", next_date.unwrap_or("无"));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_template_vars() {
        let template = "Hi {姓名}，{日期} {星期} 值班，下一位是{下一位姓名}";
        let result = replace_template_vars(
            template, "张三", "zhang@test.com",
            "2026-06-14", "星期日",
            Some("李四"), Some("2026-06-20")
        );
        assert_eq!(result, "Hi 张三，2026-06-14 星期日 值班，下一位是李四");
    }

    #[test]
    fn test_replace_without_next_person() {
        let template = "下一位：{下一位姓名}";
        let result = replace_template_vars(
            template, "张三", "z@t.com",
            "2026-06-14", "周六", None, None
        );
        assert_eq!(result, "下一位：无");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd D:/Develope/niumaren
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: 2 tests pass.

- [ ] **Step 3: Add test-send IPC command**

Append to `src-tauri/src/commands.rs`:
```rust
use crate::email;

#[tauri::command]
pub async fn test_send_email(
    db: State<'_, Database>,
    test_email: String,
) -> Result<String, String> {
    let host = db.get_setting("smtp_host").map_err(|e| e.to_string())?;
    let port: u16 = db.get_setting("smtp_port")
        .map_err(|e| e.to_string())?
        .parse().unwrap_or(465);
    let username = db.get_setting("smtp_username").map_err(|e| e.to_string())?;
    let password = db.get_setting("smtp_password").map_err(|e| e.to_string())?;
    let use_tls = db.get_setting("smtp_use_tls")
        .map_err(|e| e.to_string())?
        .parse().unwrap_or(true);
    let sender_name = db.get_setting("sender_name").map_err(|e| e.to_string())?;

    let config = email::SmtpConfig {
        host, port, username, password, use_tls, sender_name,
    };

    email::send_email(
        &config,
        &test_email,
        "测试用户",
        "【牛马人】测试邮件",
        "这是一封测试邮件。如果你收到此邮件，说明 SMTP 配置正确。\n\n—— 牛马人值班系统",
    ).await?;

    Ok("测试邮件发送成功！".to_string())
}
```

- [ ] **Step 4: Register command and module in main.rs**

Update `src-tauri/src/main.rs` — add `mod email;` and register `commands::test_send_email` in `invoke_handler`.

- [ ] **Step 5: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/
git commit -m "feat: add SMTP email module with template variable replacement

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 6: Holiday API Module

**Files:**
- Create: `src-tauri/src/holiday.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Write holiday.rs**

Write `src-tauri/src/holiday.rs`:
```rust
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HolidayApiResponse {
    #[serde(rename = "type")]
    type_field: Option<HolidayType>,
}

#[derive(Debug, Deserialize)]
struct HolidayType {
    #[serde(rename = "type")]
    type_val: Option<i32>,
    name: Option<String>,
}

/// Check if a date is a Chinese public holiday.
/// Uses timor.tech free API with local cache.
pub async fn is_holiday(
    date: NaiveDate,
    cached_holidays: &str,
    cache_year: i32,
    set_setting: impl Fn(&str, &str) -> Result<(), String>,
) -> Result<(bool, String, String, i32), String> {
    let date_str = date.format("%Y-%m-%d").to_string();
    let year = date.format("%Y").to_string().parse::<i32>().unwrap();

    // Use cache if available for this year
    if cache_year == year && !cached_holidays.is_empty() && cached_holidays != "{}" {
        let holidays: serde_json::Value = serde_json::from_str(cached_holidays)
            .map_err(|e| format!("缓存解析失败: {}", e))?;
        let is_hol = holidays.get(&date_str)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        return Ok((is_hol, cached_holidays.to_string(), "cache".to_string(), cache_year));
    }

    // Fetch from API
    let url = format!("https://timor.tech/api/holiday/year/{}", year);
    let response = reqwest::get(&url).await
        .map_err(|e| format!("节假日 API 请求失败: {}", e))?;

    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("节假日 API 解析失败: {}", e))?;

    // Build holiday set from API response
    let mut holidays = serde_json::Map::new();
    if let Some(days) = json.get("holiday").and_then(|h| h.as_object()) {
        for (day, info) in days {
            let is_off = info.get("holiday").and_then(|v| v.as_bool()).unwrap_or(false);
            holidays.insert(day.clone(), serde_json::Value::Bool(is_off));
        }
    }

    let holidays_str = serde_json::Value::Object(holidays).to_string();

    // Update cache
    set_setting("holiday_cache", &holidays_str)?;
    set_setting("holiday_cache_year", &year.to_string())?;

    let is_hol = serde_json::from_str::<serde_json::Value>(&holidays_str)
        .ok()
        .and_then(|v| v.get(&date_str).and_then(|b| b.as_bool()))
        .unwrap_or(false);

    Ok((is_hol, holidays_str, "api".to_string(), year))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_holiday_from_cache() {
        let cache = r#"{"2026-01-01":true,"2026-06-13":false}"#;
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let (is_hol, _, source, _) = is_holiday(date, cache, 2026, |_, _| Ok(())).await.unwrap();
        assert!(is_hol);
        assert_eq!(source, "cache");
    }

    #[tokio::test]
    async fn test_non_holiday_from_cache() {
        let cache = r#"{"2026-01-01":true}"#;
        let date = NaiveDate::from_ymd_opt(2026, 6, 13).unwrap();
        let (is_hol, _, source, _) = is_holiday(date, cache, 2026, |_, _| Ok(())).await.unwrap();
        assert!(!is_hol);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd D:/Develope/niumaren
cargo test --manifest-path src-tauri/Cargo.toml -- holiday
```

Expected: 2 tests pass.

- [ ] **Step 3: Add check-holiday IPC command**

Append to `src-tauri/src/commands.rs`:
```rust
use chrono::NaiveDate;

#[tauri::command]
pub async fn check_holiday(
    db: State<'_, Database>,
    date_str: String,
) -> Result<bool, String> {
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|e| format!("日期格式错误: {}", e))?;
    let cache = db.get_setting("holiday_cache").unwrap_or_default();
    let cache_year: i32 = db.get_setting("holiday_cache_year")
        .unwrap_or_default().parse().unwrap_or(0);

    // We need a closure to pass to is_holiday; use the db directly
    let (is_hol, new_cache, _source, new_year) = crate::holiday::is_holiday(
        date, &cache, cache_year,
        |k, v| {
            let db_ref = &db;
            db_ref.set_setting(k, v)
        }
    ).await?;

    Ok(is_hol)
}
```

- [ ] **Step 4: Register in main.rs**

Add `mod holiday;` and register `commands::check_holiday` in `invoke_handler`.

- [ ] **Step 5: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 6: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/
git commit -m "feat: add holiday API module with local cache

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 7: Scheduler (Duty Check + Notification Trigger)

**Files:**
- Create: `src-tauri/src/scheduler.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add schedule query methods to db.rs**

Append to `impl Database` in `src-tauri/src/db.rs`:
```rust
use crate::models::Schedule;

impl Database {
    pub fn create_schedule(&self, person_id: i64, duty_date: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO schedule (person_id, duty_date) VALUES (?1, ?2)",
            params![person_id, duty_date]
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn mark_notified(&self, schedule_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE schedule SET notified=1, notified_at=datetime('now','localtime') WHERE id=?1",
            params![schedule_id]
        )?;
        Ok(())
    }

    pub fn get_schedules(&self) -> Result<Vec<Schedule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT s.id, s.person_id, p.name, s.duty_date, s.is_holiday, s.notified,
                    s.notified_at, s.created_at
             FROM schedule s
             LEFT JOIN personnel p ON s.person_id = p.id
             ORDER BY s.duty_date DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Schedule {
                id: Some(row.get(0)?),
                person_id: row.get(1)?,
                person_name: row.get(2)?,
                duty_date: row.get(3)?,
                is_holiday: row.get(4)?,
                notified: row.get(5)?,
                notified_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }
}
```

Also add email_log methods:
```rust
use crate::models::EmailLog;

impl Database {
    pub fn create_email_log(
        &self, schedule_id: i64, recipient: &str, subject: &str, status: &str, error_msg: Option<&str>
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO email_log (schedule_id, recipient, subject, status, error_msg)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![schedule_id, recipient, subject, status, error_msg]
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_email_logs(&self) -> Result<Vec<EmailLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, schedule_id, recipient, subject, status, error_msg, sent_at
             FROM email_log ORDER BY sent_at DESC LIMIT 100"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EmailLog {
                id: Some(row.get(0)?),
                schedule_id: row.get(1)?,
                recipient: row.get(2)?,
                subject: row.get(3)?,
                status: row.get(4)?,
                error_msg: row.get(5)?,
                sent_at: Some(row.get(6)?),
            })
        })?;
        rows.collect()
    }

    pub fn resend_email_log(&self, log_id: i64) -> Result<EmailLog> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, schedule_id, recipient, subject, status, error_msg, sent_at
             FROM email_log WHERE id=?1",
            params![log_id],
            |row| Ok(EmailLog {
                id: Some(row.get(0)?),
                schedule_id: row.get(1)?,
                recipient: row.get(2)?,
                subject: row.get(3)?,
                status: row.get(4)?,
                error_msg: row.get(5)?,
                sent_at: Some(row.get(6)?),
            })
        )
    }
}
```

- [ ] **Step 2: Write scheduler.rs**

Write `src-tauri/src/scheduler.rs`:
```rust
use chrono::{Local, NaiveDate, Datelike, Duration, Weekday};
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};
use crate::db::Database;
use crate::email;
use crate::holiday;

pub struct Scheduler {
    pub db: Arc<Database>,
}

impl Scheduler {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn start(&self) {
        let mut ticker = interval(TokioDuration::from_secs(3600)); // every hour
        loop {
            ticker.tick().await;
            if let Err(e) = self.check_and_notify().await {
                eprintln!("Scheduler error: {}", e);
            }
        }
    }

    async fn check_and_notify(&self) -> Result<(), String> {
        let today = Local::now().date_naive();

        // Check Saturday notification (Thursday = Saturday - 2)
        let saturday = next_weekday(today, Weekday::Sat);
        let thursday = saturday - Duration::days(2);
        if today == thursday {
            self.process_duty_notification(saturday).await?;
        }

        // Check Sunday notification (Friday = Sunday - 2)
        let sunday = next_weekday(today, Weekday::Sun);
        let friday = sunday - Duration::days(2);
        if today == friday {
            self.process_duty_notification(sunday).await?;
        }

        Ok(())
    }

    async fn process_duty_notification(&self, duty_date: NaiveDate) -> Result<(), String> {
        let date_str = duty_date.format("%Y-%m-%d").to_string();

        // Check if already scheduled for this date
        let existing = self.db.get_schedules().unwrap_or_default();
        if existing.iter().any(|s| s.duty_date == date_str && s.notified == 1) {
            return Ok(()); // already notified
        }

        // Check holiday
        let cache = self.db.get_setting("holiday_cache").unwrap_or_default();
        let cache_year: i32 = self.db.get_setting("holiday_cache_year")
            .unwrap_or_default().parse().unwrap_or(0);
        let (is_hol, _, _, _) = holiday::is_holiday(
            duty_date, &cache, cache_year,
            |k, v| self.db.set_setting(k, v)
        ).await.unwrap_or((false, "{}".to_string(), "error".to_string(), 0));

        if is_hol {
            return Ok(()); // skip holiday
        }

        // Get next duty person
        let personnel = self.db.get_all_personnel().unwrap_or_default();
        let active: Vec<_> = personnel.iter().filter(|p| p.active == 1).collect();
        if active.is_empty() {
            return Err("没有可用的值班人员".to_string());
        }

        let last_idx_str = self.db.get_setting("last_person_index").unwrap_or("-1".to_string());
        let last_idx: i32 = last_idx_str.parse().unwrap_or(-1);
        let next_idx = ((last_idx + 1) % active.len() as i32) as usize;
        let person = &active[next_idx];

        // Get next-next person for template
        let next_next_idx = ((next_idx + 1) % active.len()) as usize;
        let next_person = active.get(next_next_idx);

        // Next duty date (next weekend day after this one)
        let next_duty = if duty_date.weekday() == Weekday::Sat {
            duty_date + Duration::days(1) // Sunday
        } else {
            next_weekday(duty_date + Duration::days(1), Weekday::Sat) // next Saturday
        };
        let next_date_str = next_duty.format("%Y-%m-%d").to_string();

        // Build email
        let subject_tpl = self.db.get_setting("email_subject_template")
            .unwrap_or_else(|_| "【值班通知】{日期} {星期}".to_string());
        let body_tpl = self.db.get_setting("email_body_template")
            .unwrap_or_else(|_| "Hi {姓名}，{日期} {星期} 你值班。".to_string());

        let day_of_week = match duty_date.weekday() {
            Weekday::Sat => "星期六",
            Weekday::Sun => "星期日",
            _ => "",
        };

        let subject = email::replace_template_vars(
            &subject_tpl, &person.name, &person.email, &date_str, day_of_week,
            next_person.map(|p| p.name.as_str()), Some(&next_date_str)
        );
        let body = email::replace_template_vars(
            &body_tpl, &person.name, &person.email, &date_str, day_of_week,
            next_person.map(|p| p.name.as_str()), Some(&next_date_str)
        );

        // Create schedule record
        let schedule_id = self.db.create_schedule(
            person.id.unwrap(), &date_str
        ).map_err(|e| format!("创建排班记录失败: {}", e))?;

        // Send email
        let host = self.db.get_setting("smtp_host").unwrap_or_default();
        if host.is_empty() {
            return Err("SMTP 未配置".to_string());
        }

        let config = email::SmtpConfig {
            host,
            port: self.db.get_setting("smtp_port").unwrap_or("465".into()).parse().unwrap_or(465),
            username: self.db.get_setting("smtp_username").unwrap_or_default(),
            password: self.db.get_setting("smtp_password").unwrap_or_default(),
            use_tls: self.db.get_setting("smtp_use_tls").unwrap_or("true".into()).parse().unwrap_or(true),
            sender_name: self.db.get_setting("sender_name").unwrap_or("值班系统".into()),
        };

        match email::send_email(&config, &person.email, &person.name, &subject, &body).await {
            Ok(_) => {
                self.db.mark_notified(schedule_id).ok();
                self.db.set_setting("last_person_index", &(next_idx as i32).to_string()).ok();
                self.db.create_email_log(
                    schedule_id, &person.email, &subject, "success", None
                ).ok();
            }
            Err(e) => {
                self.db.create_email_log(
                    schedule_id, &person.email, &subject, "failed", Some(&e)
                ).ok();
                return Err(e);
            }
        }

        Ok(())
    }
}

/// Get the next occurrence of a given weekday after (or on) a date
fn next_weekday(from: NaiveDate, target: Weekday) -> NaiveDate {
    let current = from.weekday().num_days_from_monday();
    let target_num = target.num_days_from_monday();
    let days_until = if target_num >= current {
        target_num - current
    } else {
        7 - current + target_num
    };
    from + Duration::days(days_until as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_weekday() {
        // Thursday (Jun 4, 2026 is a Thursday)
        let thu = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let sat = next_weekday(thu, Weekday::Sat);
        assert_eq!(sat, NaiveDate::from_ymd_opt(2026, 6, 6).unwrap());

        let sun = next_weekday(thu, Weekday::Sun);
        assert_eq!(sun, NaiveDate::from_ymd_opt(2026, 6, 7).unwrap());
    }

    #[test]
    fn test_next_weekday_same_day() {
        let sat = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(); // Saturday
        assert_eq!(next_weekday(sat, Weekday::Sat), sat);
    }
}
```

- [ ] **Step 3: Run scheduler tests**

```bash
cd D:/Develope/niumaren
cargo test --manifest-path src-tauri/Cargo.toml -- scheduler
```

Expected: 2 tests pass.

- [ ] **Step 4: Add get_schedules and get_email_logs IPC commands**

Append to `src-tauri/src/commands.rs`:
```rust
use crate::models::{Schedule, EmailLog};

#[tauri::command]
pub fn get_schedules(db: State<Database>) -> Result<Vec<Schedule>, String> {
    db.get_schedules().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_email_logs(db: State<Database>) -> Result<Vec<EmailLog>, String> {
    db.get_email_logs().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn manual_send(db: State<'_, Database>) -> Result<String, String> {
    let today = chrono::Local::now().date_naive();
    let saturday = crate::scheduler::next_weekday(today, chrono::Weekday::Sat);
    crate::scheduler::Scheduler::new(std::sync::Arc::new(db.inner().clone()))
        .check_and_notify().await?;
    Ok("手动发送完成".to_string())
}
```

Wait, this won't work because State doesn't give us the inner Database. Let me fix this approach — pass the DB reference differently.

Actually, let me simplify: instead of trying to reuse the scheduler from commands, let's add a dedicated manual-check command that duplicates the logic.

- [ ] **Step 4 (revised): Add schedule/log query commands only**

Append to `src-tauri/src/commands.rs`:
```rust
use crate::models::{Schedule, EmailLog};

#[tauri::command]
pub fn get_schedules(db: State<Database>) -> Result<Vec<Schedule>, String> {
    db.get_schedules().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_email_logs(db: State<Database>) -> Result<Vec<EmailLog>, String> {
    db.get_email_logs().map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Update main.rs to start scheduler**

Update `src-tauri/src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod db;
mod commands;
mod email;
mod holiday;
mod scheduler;

use db::Database;
use scheduler::Scheduler;
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            let database = Database::new(app_dir)
                .expect("Failed to initialize database");
            let db = Arc::new(database);
            app.manage(db.clone());

            // Start scheduler in background
            let scheduler = Scheduler::new(db);
            tauri::async_runtime::spawn(async move {
                scheduler.start().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_personnel,
            commands::add_personnel,
            commands::update_personnel,
            commands::delete_personnel,
            commands::reorder_personnel,
            commands::get_settings,
            commands::save_settings,
            commands::save_setting,
            commands::test_send_email,
            commands::check_holiday,
            commands::get_schedules,
            commands::get_email_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compilation success.

- [ ] **Step 7: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/
git commit -m "feat: add scheduler with duty rotation and notification logic

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 8: System Tray Integration

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add tray setup in main.rs**

Update `src-tauri/src/main.rs` — replace the `fn main()` with:
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            let database = Database::new(app_dir)
                .expect("Failed to initialize database");
            let db = Arc::new(database);
            app.manage(db.clone());

            // Build tray menu
            let show = MenuItem::with_id(app, "show", "打开主面板", true, None::<&str>)?;
            let schedule = MenuItem::with_id(app, "schedule", "查看排班表", true, None::<&str>)?;
            let manual = MenuItem::with_id(app, "manual", "手动发送通知", true, None::<&str>)?;
            let separator = MenuItem::with_id(app, "sep", "──────────", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &schedule, &manual, &separator, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("牛马人 · 值班助手")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up, ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            // Hide window on close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        w.hide().ok();
                    }
                });
            }

            // Start scheduler in background
            let scheduler = Scheduler::new(db.clone());
            tauri::async_runtime::spawn(async move {
                scheduler.start().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_personnel,
            commands::add_personnel,
            commands::update_personnel,
            commands::delete_personnel,
            commands::reorder_personnel,
            commands::get_settings,
            commands::save_settings,
            commands::save_setting,
            commands::test_send_email,
            commands::check_holiday,
            commands::get_schedules,
            commands::get_email_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Create tray icon**

Create a simple 32x32 PNG icon for the tray. Use any tool to create `src-tauri/icons/tray-icon.png` with a simple 🐂 emoji-style icon (a brown circle with text would work).

For now, we can copy the app icon:
```bash
cd D:/Develope/niumaren
cp src-tauri/icons/32x32.png src-tauri/icons/tray-icon.png
```

- [ ] **Step 3: Set tray icon in tauri.conf.json**

Add to `src-tauri/tauri.conf.json`:
```json
{
  "app": {
    "trayIcon": {
      "iconPath": "icons/tray-icon.png",
      "iconAsTemplate": true
    }
  }
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

- [ ] **Step 5: Commit**

```bash
cd D:/Develope/niumaren
git add src-tauri/src/main.rs src-tauri/tauri.conf.json src-tauri/icons/
git commit -m "feat: add system tray with context menu

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 9: Frontend — Main Layout + IPC Hook

**Files:**
- Create: `src/hooks/useTauriInvoke.ts`
- Modify: `src/App.tsx`
- Modify: `src/App.css` (or `src/index.css`)

- [ ] **Step 1: Write the IPC hook**

Write `src/hooks/useTauriInvoke.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';

export function useTauriInvoke<T>(command: string) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const call = useCallback(async (args?: Record<string, unknown>) => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<T>(command, args);
      setData(result);
      return result;
    } catch (e) {
      const msg = typeof e === 'string' ? e : (e as Error).message;
      setError(msg);
      return null;
    } finally {
      setLoading(false);
    }
  }, [command]);

  return { data, loading, error, call };
}
```

- [ ] **Step 2: Write the main App layout**

Write `src/App.tsx`:
```tsx
import { useState } from 'react';
import PersonnelTab from './components/PersonnelTab';
import EmailConfigTab from './components/EmailConfigTab';
import TemplateTab from './components/TemplateTab';
import ScheduleTab from './components/ScheduleTab';
import LogTab from './components/LogTab';
import { Users, Mail, FileText, Calendar, List } from 'lucide-react';

type Tab = 'personnel' | 'config' | 'template' | 'schedule' | 'log';

const TABS: { key: Tab; label: string; icon: React.ReactNode }[] = [
  { key: 'personnel', label: '人员管理', icon: <Users size={16} /> },
  { key: 'config', label: '邮件配置', icon: <Mail size={16} /> },
  { key: 'template', label: '邮件模板', icon: <FileText size={16} /> },
  { key: 'schedule', label: '排班表', icon: <Calendar size={16} /> },
  { key: 'log', label: '发送日志', icon: <List size={16} /> },
];

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('personnel');

  const renderTab = () => {
    switch (activeTab) {
      case 'personnel': return <PersonnelTab />;
      case 'config': return <EmailConfigTab />;
      case 'template': return <TemplateTab />;
      case 'schedule': return <ScheduleTab />;
      case 'log': return <LogTab />;
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 flex flex-col">
      {/* Title Bar */}
      <div className="bg-gray-900 border-b border-gray-800 px-4 py-2 flex items-center gap-3">
        <span className="text-xl">🐂</span>
        <h1 className="text-lg font-bold">牛马人 · 值班助手</h1>
      </div>

      {/* Tab Bar */}
      <div className="flex border-b border-gray-800 bg-gray-900">
        {TABS.map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex items-center gap-2 px-4 py-2.5 text-sm transition-colors border-b-2 ${
              activeTab === tab.key
                ? 'border-amber-500 text-amber-400 bg-gray-900'
                : 'border-transparent text-gray-400 hover:text-gray-200 hover:bg-gray-800'
            }`}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto p-4">
        {renderTab()}
      </div>

      {/* Status Bar */}
      <div className="bg-gray-900 border-t border-gray-800 px-4 py-1.5 text-xs text-gray-500 flex items-center gap-4">
        <span className="flex items-center gap-1">
          <span className="w-2 h-2 rounded-full bg-green-500" />
          运行中
        </span>
        <span>牛马人 v0.1.0</span>
      </div>
    </div>
  );
}

export default App;
```

- [ ] **Step 3: Update global CSS**

Write `src/index.css`:
```css
@import "tailwindcss";

body {
  margin: 0;
  padding: 0;
  background: #030712;
  color: #f3f4f6;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: #1f2937;
}
::-webkit-scrollbar-thumb {
  background: #4b5563;
  border-radius: 3px;
}

input, textarea, select {
  background: #1f2937;
  border: 1px solid #374151;
  color: #e5e7eb;
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 14px;
}

input:focus, textarea:focus, select:focus {
  outline: none;
  border-color: #f59e0b;
  box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.2);
}

button {
  cursor: pointer;
}

.btn-primary {
  background: #d97706;
  color: white;
  border: none;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  transition: background 0.15s;
}
.btn-primary:hover {
  background: #b45309;
}
.btn-danger {
  background: #dc2626;
  color: white;
  border: none;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 14px;
}
.btn-danger:hover {
  background: #b91c1c;
}
.btn-secondary {
  background: #374151;
  color: #e5e7eb;
  border: 1px solid #4b5563;
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 14px;
}
.btn-secondary:hover {
  background: #4b5563;
}
```

- [ ] **Step 4: Create placeholder components (stubs)**

Create each component file with a minimal placeholder:

`src/components/PersonnelTab.tsx`:
```tsx
export default function PersonnelTab() {
  return <div className="text-gray-400">人员管理 — 即将实现</div>;
}
```
(Repeat for EmailConfigTab, TemplateTab, ScheduleTab, LogTab)

- [ ] **Step 5: Verify frontend builds**

```bash
cd D:/Develope/niumaren
npm run build
```

Expected: Vite builds without errors.

- [ ] **Step 6: Commit**

```bash
cd D:/Develope/niumaren
git add src/ src/index.css
git commit -m "feat: add main layout with tabs and IPC hook

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 10: PersonnelTab Component

**Files:**
- Modify: `src/components/PersonnelTab.tsx`

- [ ] **Step 1: Write the full PersonnelTab component**

Write `src/components/PersonnelTab.tsx`:
```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Personnel } from '../types';
import { Plus, Trash2, GripVertical, Check, X } from 'lucide-react';

export default function PersonnelTab() {
  const [personnel, setPersonnel] = useState<Personnel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const [newEmail, setNewEmail] = useState('');
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editName, setEditName] = useState('');
  const [editEmail, setEditEmail] = useState('');

  const loadPersonnel = useCallback(async () => {
    try {
      const data = await invoke<Personnel[]>('get_personnel');
      setPersonnel(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadPersonnel(); }, [loadPersonnel]);

  const addPerson = async () => {
    if (!newName.trim() || !newEmail.trim()) return;
    try {
      await invoke('add_personnel', { name: newName.trim(), email: newEmail.trim() });
      setNewName('');
      setNewEmail('');
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const updatePerson = async (p: Personnel) => {
    try {
      await invoke('update_personnel', { personnel: p });
      setEditingId(null);
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const deletePerson = async (id: number) => {
    try {
      await invoke('delete_personnel', { id });
      await loadPersonnel();
    } catch (e) {
      setError(String(e));
    }
  };

  const toggleActive = async (p: Personnel) => {
    await updatePerson({ ...p, active: p.active === 1 ? 0 : 1 });
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      {error && (
        <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {error}
          <button onClick={() => setError(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      {/* Add Form */}
      <div className="flex gap-3 mb-6 items-end">
        <div className="flex-1">
          <label className="text-xs text-gray-400 mb-1 block">姓名</label>
          <input
            value={newName}
            onChange={e => setNewName(e.target.value)}
            placeholder="张三"
            className="w-full"
            onKeyDown={e => e.key === 'Enter' && addPerson()}
          />
        </div>
        <div className="flex-1">
          <label className="text-xs text-gray-400 mb-1 block">邮箱</label>
          <input
            value={newEmail}
            onChange={e => setNewEmail(e.target.value)}
            placeholder="zhangsan@qq.com"
            className="w-full"
            onKeyDown={e => e.key === 'Enter' && addPerson()}
          />
        </div>
        <button onClick={addPerson} className="btn-primary flex items-center gap-1">
          <Plus size={14} /> 添加
        </button>
      </div>

      {/* Personnel Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="py-2 px-2 w-10">#</th>
              <th className="py-2 px-2">姓名</th>
              <th className="py-2 px-2">邮箱</th>
              <th className="py-2 px-2 w-20">状态</th>
              <th className="py-2 px-2 w-28">操作</th>
            </tr>
          </thead>
          <tbody>
            {personnel.map((p, idx) => (
              <tr key={p.id} className="border-b border-gray-800 hover:bg-gray-900/50">
                <td className="py-2 px-2 text-gray-500">{idx + 1}</td>
                <td className="py-2 px-2">
                  {editingId === p.id ? (
                    <input
                      value={editName}
                      onChange={e => setEditName(e.target.value)}
                      className="w-full text-sm"
                    />
                  ) : p.name}
                </td>
                <td className="py-2 px-2">
                  {editingId === p.id ? (
                    <input
                      value={editEmail}
                      onChange={e => setEditEmail(e.target.value)}
                      className="w-full text-sm"
                    />
                  ) : p.email}
                </td>
                <td className="py-2 px-2">
                  <button
                    onClick={() => toggleActive(p)}
                    className={`text-xs px-2 py-0.5 rounded ${
                      p.active === 1 ? 'bg-green-900/50 text-green-400' : 'bg-gray-800 text-gray-500'
                    }`}
                  >
                    {p.active === 1 ? '启用' : '禁用'}
                  </button>
                </td>
                <td className="py-2 px-2">
                  <div className="flex gap-1">
                    {editingId === p.id ? (
                      <>
                        <button
                          onClick={() => updatePerson({
                            ...p, name: editName, email: editEmail
                          })}
                          className="text-green-400 hover:text-green-300"
                        >
                          <Check size={16} />
                        </button>
                        <button
                          onClick={() => setEditingId(null)}
                          className="text-gray-400 hover:text-gray-300"
                        >
                          <X size={16} />
                        </button>
                      </>
                    ) : (
                      <>
                        <button
                          onClick={() => {
                            setEditingId(p.id!);
                            setEditName(p.name);
                            setEditEmail(p.email);
                          }}
                          className="text-amber-400 hover:text-amber-300 text-xs mr-2"
                        >
                          编辑
                        </button>
                        <button
                          onClick={() => deletePerson(p.id!)}
                          className="text-red-400 hover:text-red-300"
                        >
                          <Trash2 size={14} />
                        </button>
                      </>
                    )}
                  </div>
                </td>
              </tr>
            ))}
            {personnel.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  还没有值班人员，点击上方添加
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd D:/Develope/niumaren
npm run build
```

- [ ] **Step 3: Commit**

```bash
cd D:/Develope/niumaren
git add src/components/PersonnelTab.tsx
git commit -m "feat: add PersonnelTab with full CRUD UI

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 11: EmailConfigTab Component

**Files:**
- Modify: `src/components/EmailConfigTab.tsx`

- [ ] **Step 1: Write the EmailConfigTab component**

Write `src/components/EmailConfigTab.tsx`:
```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SmtpSettings } from '../types';
import { Save, Send, Loader2 } from 'lucide-react';

export default function EmailConfigTab() {
  const [settings, setSettings] = useState<SmtpSettings>({
    smtp_host: '', smtp_port: 465, smtp_username: '',
    smtp_password: '', smtp_use_tls: true, sender_name: '值班系统',
  });
  const [testEmail, setTestEmail] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [msg, setMsg] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      const data = await invoke<{ key: string; value: string }[]>('get_settings');
      const map: Record<string, string> = {};
      data.forEach(s => { map[s.key] = s.value; });
      setSettings({
        smtp_host: map.smtp_host || '',
        smtp_port: parseInt(map.smtp_port || '465'),
        smtp_username: map.smtp_username || '',
        smtp_password: map.smtp_password || '',
        smtp_use_tls: map.smtp_use_tls !== 'false',
        sender_name: map.sender_name || '值班系统',
      });
      setTestEmail(map.smtp_username || '');
    } catch (e) {
      setMsg({ type: 'error', text: `加载失败: ${e}` });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadSettings(); }, [loadSettings]);

  const save = async () => {
    setSaving(true);
    try {
      const entries = Object.entries(settings).map(([key, value]) => ({
        key, value: String(value),
      }));
      await invoke('save_settings', { settings: entries });
      setMsg({ type: 'success', text: '保存成功' });
    } catch (e) {
      setMsg({ type: 'error', text: `保存失败: ${e}` });
    } finally {
      setSaving(false);
    }
  };

  const testSend = async () => {
    if (!testEmail.trim()) return;
    setTesting(true);
    try {
      const result = await invoke<string>('test_send_email', { testEmail: testEmail.trim() });
      setMsg({ type: 'success', text: result });
    } catch (e) {
      setMsg({ type: 'error', text: `测试失败: ${e}` });
    } finally {
      setTesting(false);
    }
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div className="max-w-lg">
      {msg && (
        <div className={`px-4 py-2 rounded-lg mb-4 text-sm ${
          msg.type === 'success'
            ? 'bg-green-900/50 border border-green-700 text-green-200'
            : 'bg-red-900/50 border border-red-700 text-red-200'
        }`}>
          {msg.text}
          <button onClick={() => setMsg(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      <div className="space-y-4">
        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 服务器地址</label>
          <input
            value={settings.smtp_host}
            onChange={e => setSettings({ ...settings, smtp_host: e.target.value })}
            placeholder="smtp.qq.com"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 端口</label>
          <input
            type="number"
            value={settings.smtp_port}
            onChange={e => setSettings({ ...settings, smtp_port: parseInt(e.target.value) || 465 })}
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">发件邮箱地址</label>
          <input
            value={settings.smtp_username}
            onChange={e => setSettings({ ...settings, smtp_username: e.target.value })}
            placeholder="your@qq.com"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">SMTP 授权码</label>
          <input
            type="password"
            value={settings.smtp_password}
            onChange={e => setSettings({ ...settings, smtp_password: e.target.value })}
            placeholder="输入邮箱授权码（非登录密码）"
            className="w-full"
          />
        </div>

        <div>
          <label className="text-xs text-gray-400 mb-1 block">发件人显示名称</label>
          <input
            value={settings.sender_name}
            onChange={e => setSettings({ ...settings, sender_name: e.target.value })}
            className="w-full"
          />
        </div>

        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={settings.smtp_use_tls}
              onChange={e => setSettings({ ...settings, smtp_use_tls: e.target.checked })}
              className="w-4 h-4"
            />
            <span className="text-sm">使用 TLS/SSL 加密</span>
          </label>
        </div>

        <button onClick={save} disabled={saving} className="btn-primary flex items-center gap-2">
          {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
          保存配置
        </button>

        <hr className="border-gray-700 my-6" />

        <h3 className="text-sm font-semibold text-gray-300">测试发送</h3>
        <div className="flex gap-3 items-end">
          <div className="flex-1">
            <label className="text-xs text-gray-400 mb-1 block">发送测试邮件到</label>
            <input
              value={testEmail}
              onChange={e => setTestEmail(e.target.value)}
              placeholder="test@example.com"
              className="w-full"
              onKeyDown={e => e.key === 'Enter' && testSend()}
            />
          </div>
          <button onClick={testSend} disabled={testing} className="btn-secondary flex items-center gap-2">
            {testing ? <Loader2 size={14} className="animate-spin" /> : <Send size={14} />}
            发送测试
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd D:/Develope/niumaren
npm run build
```

- [ ] **Step 3: Commit**

```bash
cd D:/Develope/niumaren
git add src/components/EmailConfigTab.tsx
git commit -m "feat: add EmailConfigTab with SMTP settings and test send

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 12: TemplateTab Component

**Files:**
- Modify: `src/components/TemplateTab.tsx`

- [ ] **Step 1: Write the TemplateTab component**

Write `src/components/TemplateTab.tsx`:
```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, Eye, Loader2 } from 'lucide-react';

const VARIABLES = [
  { v: '{姓名}', d: '值班人员姓名' },
  { v: '{邮箱}', d: '值班人员邮箱' },
  { v: '{日期}', d: '值班日期（YYYY-MM-DD）' },
  { v: '{星期}', d: '星期几' },
  { v: '{下一位姓名}', d: '下一个值班人姓名' },
  { v: '{下一位日期}', d: '下一个值班日期' },
];

export default function TemplateTab() {
  const [subject, setSubject] = useState('');
  const [body, setBody] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [preview, setPreview] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const data = await invoke<{ key: string; value: string }[]>('get_settings');
      const map: Record<string, string> = {};
      data.forEach(s => { map[s.key] = s.value; });
      setSubject(map.email_subject_template || '【值班通知】{日期} {星期}');
      setBody(map.email_body_template || 'Hi {姓名}，{日期} {星期} 你值班。');
    } catch (e) {
      setMsg(`加载失败: ${e}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const save = async () => {
    setSaving(true);
    try {
      await invoke('save_setting', { key: 'email_subject_template', value: subject });
      await invoke('save_setting', { key: 'email_body_template', value: body });
      setMsg('模板已保存');
    } catch (e) {
      setMsg(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const insertVar = (v: string) => {
    const ta = document.querySelector('textarea') as HTMLTextAreaElement;
    if (!ta) return;
    const start = ta.selectionStart;
    const end = ta.selectionEnd;
    const newBody = body.substring(0, start) + v + body.substring(end);
    setBody(newBody);
    setTimeout(() => {
      ta.selectionStart = ta.selectionEnd = start + v.length;
      ta.focus();
    }, 0);
  };

  const previewVars = (text: string) => text
    .replace(/\{姓名\}/g, '张三')
    .replace(/\{邮箱\}/g, 'zhangsan@qq.com')
    .replace(/\{日期\}/g, '2026-06-14')
    .replace(/\{星期\}/g, '星期日')
    .replace(/\{下一位姓名\}/g, '李四')
    .replace(/\{下一位日期\}/g, '2026-06-20');

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      {msg && (
        <div className="bg-green-900/50 border border-green-700 text-green-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {msg}
          <button onClick={() => setMsg(null)} className="ml-3 underline">关闭</button>
        </div>
      )}

      <div className="flex gap-6">
        {/* Editor */}
        <div className="flex-1 space-y-4">
          <div>
            <label className="text-xs text-gray-400 mb-1 block">邮件标题模板</label>
            <input
              value={subject}
              onChange={e => setSubject(e.target.value)}
              className="w-full"
              placeholder="【值班通知】{日期} {星期}"
            />
          </div>

          <div>
            <label className="text-xs text-gray-400 mb-1 block">邮件正文模板</label>
            <textarea
              value={body}
              onChange={e => setBody(e.target.value)}
              className="w-full font-mono text-sm"
              rows={14}
              style={{ background: '#1f2937', border: '1px solid #374151', color: '#e5e7eb',
                       borderRadius: '6px', padding: '10px', resize: 'vertical' }}
            />
          </div>

          <div className="flex gap-2">
            <button onClick={save} disabled={saving} className="btn-primary flex items-center gap-2">
              {saving ? <Loader2 size={14} className="animate-spin" /> : <Save size={14} />}
              保存模板
            </button>
            <button
              onClick={() => setPreview(!preview)}
              className="btn-secondary flex items-center gap-2"
            >
              <Eye size={14} />
              {preview ? '关闭预览' : '预览'}
            </button>
          </div>
        </div>

        {/* Variable Reference + Preview */}
        <div className="w-72 space-y-4">
          <div className="bg-gray-900 rounded-lg p-4 border border-gray-800">
            <h3 className="text-sm font-semibold mb-2 text-gray-300">可用变量</h3>
            <div className="space-y-1.5">
              {VARIABLES.map(v => (
                <button
                  key={v.v}
                  onClick={() => insertVar(v.v)}
                  className="flex items-center justify-between w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm"
                >
                  <code className="text-amber-400 text-xs">{v.v}</code>
                  <span className="text-gray-500 text-xs">{v.d}</span>
                </button>
              ))}
            </div>
          </div>

          {preview && (
            <div className="bg-gray-900 rounded-lg p-4 border border-gray-700">
              <h3 className="text-sm font-semibold mb-2 text-gray-300">预览效果</h3>
              <div className="text-xs text-gray-400 mb-2">
                标题：<span className="text-gray-200">{previewVars(subject)}</span>
              </div>
              <div className="text-xs text-gray-200 whitespace-pre-wrap bg-gray-950 p-3 rounded border border-gray-800">
                {previewVars(body)}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd D:/Develope/niumaren
npm run build
```

- [ ] **Step 3: Commit**

```bash
cd D:/Develope/niumaren
git add src/components/TemplateTab.tsx
git commit -m "feat: add TemplateTab with variable insertion and preview

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 13: ScheduleTab Component

**Files:**
- Modify: `src/components/ScheduleTab.tsx`

- [ ] **Step 1: Write the ScheduleTab component**

Write `src/components/ScheduleTab.tsx`:
```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Schedule } from '../types';
import { RefreshCw } from 'lucide-react';

export default function ScheduleTab() {
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<Schedule[]>('get_schedules');
      setSchedules(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const getStatusBadge = (s: Schedule) => {
    if (s.is_holiday === 1) {
      return <span className="text-xs bg-gray-800 text-gray-500 px-2 py-0.5 rounded">已跳过（节假日）</span>;
    }
    if (s.notified === 1) {
      return <span className="text-xs bg-green-900/50 text-green-400 px-2 py-0.5 rounded">✅ 已通知</span>;
    }
    return <span className="text-xs bg-amber-900/50 text-amber-400 px-2 py-0.5 rounded">⏳ 待发送</span>;
  };

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold text-gray-300">排班记录</h2>
        <button onClick={load} className="btn-secondary flex items-center gap-1 text-xs">
          <RefreshCw size={12} /> 刷新
        </button>
      </div>

      {error && (
        <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {error}
        </div>
      )}

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="py-2 px-3">日期</th>
              <th className="py-2 px-3">星期</th>
              <th className="py-2 px-3">值班人</th>
              <th className="py-2 px-3">状态</th>
              <th className="py-2 px-3">通知时间</th>
            </tr>
          </thead>
          <tbody>
            {schedules.map(s => {
              const date = new Date(s.duty_date);
              const weekdays = ['日', '一', '二', '三', '四', '五', '六'];
              const weekday = weekdays[date.getDay()];
              return (
                <tr key={s.id} className={`border-b border-gray-800 ${s.is_holiday ? 'opacity-40' : ''}`}>
                  <td className="py-2 px-3">{s.duty_date}</td>
                  <td className="py-2 px-3">
                    <span className={date.getDay() === 0 || date.getDay() === 6 ? 'text-amber-400' : ''}>
                      周{weekday}
                    </span>
                  </td>
                  <td className="py-2 px-3">{s.person_name || '-'}</td>
                  <td className="py-2 px-3">{getStatusBadge(s)}</td>
                  <td className="py-2 px-3 text-gray-500 text-xs">
                    {s.notified_at || '-'}
                  </td>
                </tr>
              );
            })}
            {schedules.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  暂无排班记录。应用运行后将自动生成排班。
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd D:/Develope/niumaren
npm run build
```

- [ ] **Step 3: Commit**

```bash
cd D:/Develope/niumaren
git add src/components/ScheduleTab.tsx
git commit -m "feat: add ScheduleTab with duty history view

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 14: LogTab Component

**Files:**
- Modify: `src/components/LogTab.tsx`

- [ ] **Step 1: Write the LogTab component**

Write `src/components/LogTab.tsx`:
```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { EmailLog } from '../types';
import { RefreshCw, Filter } from 'lucide-react';

type FilterStatus = 'all' | 'success' | 'failed';

export default function LogTab() {
  const [logs, setLogs] = useState<EmailLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterStatus>('all');

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<EmailLog[]>('get_email_logs');
      setLogs(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const filtered = filter === 'all'
    ? logs
    : logs.filter(l => l.status === filter);

  if (loading) return <div className="text-gray-400">加载中...</div>;

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold text-gray-300">邮件发送记录</h2>
        <div className="flex gap-2">
          <div className="flex rounded-lg overflow-hidden border border-gray-700 text-xs">
            {(['all', 'success', 'failed'] as FilterStatus[]).map(f => (
              <button
                key={f}
                onClick={() => setFilter(f)}
                className={`px-3 py-1 ${
                  filter === f ? 'bg-amber-600 text-white' : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                {f === 'all' ? '全部' : f === 'success' ? '成功' : '失败'}
              </button>
            ))}
          </div>
          <button onClick={load} className="btn-secondary flex items-center gap-1 text-xs">
            <RefreshCw size={12} /> 刷新
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-900/50 border border-red-700 text-red-200 px-4 py-2 rounded-lg mb-4 text-sm">
          {error}
        </div>
      )}

      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="py-2 px-3">时间</th>
              <th className="py-2 px-3">收件人</th>
              <th className="py-2 px-3">邮件标题</th>
              <th className="py-2 px-3">状态</th>
              <th className="py-2 px-3">错误信息</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(log => (
              <tr key={log.id} className={`border-b border-gray-800 ${
                log.status === 'failed' ? 'bg-red-900/10' : ''
              }`}>
                <td className="py-2 px-3 text-gray-500 text-xs">{log.sent_at}</td>
                <td className="py-2 px-3">{log.recipient}</td>
                <td className="py-2 px-3 text-gray-300 max-w-60 truncate">{log.subject}</td>
                <td className="py-2 px-3">
                  {log.status === 'success' ? (
                    <span className="text-green-400 text-xs">✅ 成功</span>
                  ) : (
                    <span className="text-red-400 text-xs">❌ 失败</span>
                  )}
                </td>
                <td className="py-2 px-3 text-red-400 text-xs max-w-40 truncate">
                  {log.error_msg || '-'}
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-8 text-gray-500">
                  暂无发送记录
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd D:/Develope/niumaren
npm run build
```

- [ ] **Step 3: Commit**

```bash
cd D:/Develope/niumaren
git add src/components/LogTab.tsx
git commit -m "feat: add LogTab with email send history and status filter

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 15: Integration — Fix Compilation Issues & Full Build

**Files:**
- Modify: `src-tauri/src/main.rs` (Arc<Database> handling)
- Modify: `src-tauri/src/commands.rs` (State type fixes)
- Modify: `src-tauri/src/scheduler.rs` (pub export)

- [ ] **Step 1: Fix Rust State management**

The `State` in Tauri wraps `Arc` internally. Update `src-tauri/src/main.rs`:
```rust
use std::sync::Arc;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            let database = Database::new(app_dir)
                .expect("Failed to initialize database");
            let db = Arc::new(database);
            app.manage(db.clone());

            // ... tray setup ...

            // Start scheduler
            let scheduler_db = db.clone();
            tauri::async_runtime::spawn(async move {
                let scheduler = Scheduler::new(scheduler_db);
                scheduler.start().await;
            });

            Ok(())
        })
        // ...
}
```

- [ ] **Step 2: Make scheduler::next_weekday public**

In `src-tauri/src/scheduler.rs`, change `fn next_weekday` to `pub fn next_weekday`.

- [ ] **Step 3: Fix imports in commands.rs**

Ensure `src-tauri/src/commands.rs` has:
```rust
use tauri::State;
use std::sync::Arc;
use crate::db::Database;
use crate::models::{Personnel, Setting, Schedule, EmailLog};
```

Where `State<Arc<Database>>` is used:
```rust
#[tauri::command]
pub fn get_personnel(db: State<Arc<Database>>) -> Result<Vec<Personnel>, String> {
    db.get_all_personnel().map_err(|e| e.to_string())
}
// ... all other commands similarly use State<Arc<Database>>
```

- [ ] **Step 4: Full build check**

```bash
cd D:/Develope/niumaren
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Compilation succeeds with all modules integrated.

- [ ] **Step 5: Frontend build**

```bash
cd D:/Develope/niumaren
npm run build
```

Expected: Vite builds without errors.

- [ ] **Step 6: Commit**

```bash
cd D:/Develope/niumaren
git add -A
git commit -m "fix: integrate Rust modules, fix State<Arc<Database>> types

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 16: Final Verification — Dev Run & Smoke Test

- [ ] **Step 1: Launch the app in dev mode**

```bash
cd D:/Develope/niumaren
npm run tauri dev
```

- [ ] **Step 2: Smoke test checklist**

| Check | Action | Expected |
|-------|--------|----------|
| Window opens | App starts | Main window visible with 5 tabs |
| Personnel tab | Add a test person | Row appears in table |
| Edit person | Click edit, change name, save | Name updates |
| Delete person | Click trash icon | Row removed |
| Email config | Fill SMTP settings, save | "保存成功" message |
| Email template | Edit subject/body, save | "模板已保存" message |
| Template preview | Click preview | Variable-replaced text shown |
| Schedule tab | Displays schedule records | Table loads (empty ok) |
| Log tab | Displays email logs | Table loads with filter |
| Tray icon | Check system tray | 🐂 icon visible |
| Tray menu | Right-click tray icon | Context menu shows |
| Close to tray | Click window X | Window hides, app keeps running |

- [ ] **Step 3: Fix any issues found during smoke test**

- [ ] **Step 4: Final commit**

```bash
cd D:/Develope/niumaren
git add -A
git commit -m "chore: final integration fixes after smoke test

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Implementation Order Summary

```
Task 1  → Scaffold project
Task 2  → Rust models + DB init
Task 3  → Personnel CRUD commands
Task 4  → Settings commands
Task 5  → Email module
Task 6  → Holiday module
Task 7  → Scheduler
Task 8  → System tray
Task 9  → Frontend layout + IPC hook
Task 10 → PersonnelTab UI
Task 11 → EmailConfigTab UI
Task 12 → TemplateTab UI
Task 13 → ScheduleTab UI
Task 14 → LogTab UI
Task 15 → Integration fixes
Task 16 → Smoke test & polish
```
