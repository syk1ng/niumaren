use rusqlite::{Connection, Result, params};
use std::sync::Mutex;
use std::path::PathBuf;
use crate::models::{Personnel, Schedule, EmailLog, Setting};

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

    // ─── Personnel CRUD ────────────────────────────────────────────

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

    // ─── Settings ───────────────────────────────────────────────────

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

    // ─── Schedule ───────────────────────────────────────────────────

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

    // ─── Email Log ──────────────────────────────────────────────────

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
