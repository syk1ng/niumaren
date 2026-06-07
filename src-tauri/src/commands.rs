use tauri::State;
use std::sync::Arc;
use crate::db::Database;
use crate::models::{Personnel, Setting, Schedule, EmailLog};
use chrono::NaiveDate;

// ─── Personnel Commands ─────────────────────────────────────────

#[tauri::command]
pub fn get_personnel(db: State<Arc<Database>>) -> Result<Vec<Personnel>, String> {
    db.get_all_personnel().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_personnel(db: State<Arc<Database>>, name: String, email: String) -> Result<Personnel, String> {
    if !email.contains('@') || !email.contains('.') {
        return Err("邮箱格式不正确".to_string());
    }
    if name.trim().is_empty() {
        return Err("姓名不能为空".to_string());
    }
    db.add_personnel(&name.trim(), &email.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_personnel(db: State<Arc<Database>>, personnel: Personnel) -> Result<(), String> {
    if !personnel.email.contains('@') || !personnel.email.contains('.') {
        return Err("邮箱格式不正确".to_string());
    }
    if personnel.name.trim().is_empty() {
        return Err("姓名不能为空".to_string());
    }
    db.update_personnel(&personnel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_personnel(db: State<Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_personnel(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_personnel(db: State<Arc<Database>>, ids: Vec<i64>) -> Result<(), String> {
    db.reorder_personnel(&ids).map_err(|e| e.to_string())
}

// ─── Settings Commands ──────────────────────────────────────────

#[tauri::command]
pub fn get_settings(db: State<Arc<Database>>) -> Result<Vec<Setting>, String> {
    db.get_all_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(db: State<Arc<Database>>, settings: Vec<Setting>) -> Result<(), String> {
    for s in settings {
        db.set_setting(&s.key, &s.value).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_setting(db: State<Arc<Database>>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

// ─── Email Commands ─────────────────────────────────────────────

#[tauri::command]
pub async fn test_send_email(
    db: State<'_, Arc<Database>>,
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

    let config = crate::email::SmtpConfig {
        host, port, username, password, use_tls, sender_name,
    };

    crate::email::send_email(
        &config,
        &test_email,
        "测试用户",
        "【牛马人】测试邮件",
        "这是一封测试邮件。如果你收到此邮件，说明 SMTP 配置正确。\n\n—— 牛马人值班系统",
    ).await?;

    Ok("测试邮件发送成功！".to_string())
}

// ─── Holiday Commands ───────────────────────────────────────────

#[tauri::command]
pub async fn check_holiday(
    db: State<'_, Arc<Database>>,
    date_str: String,
) -> Result<bool, String> {
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|e| format!("日期格式错误: {}", e))?;
    let cache = db.get_setting("holiday_cache").unwrap_or_default();
    let cache_year: i32 = db.get_setting("holiday_cache_year")
        .unwrap_or_default().parse().unwrap_or(0);

    let (is_hol, _new_cache, _source, _new_year) = crate::holiday::is_holiday(
        date, &cache, cache_year,
        |k, v| {
            db.set_setting(k, v).map_err(|e| format!("设置保存失败: {}", e))
        }
    ).await?;

    Ok(is_hol)
}

// ─── Schedule & Log Commands ────────────────────────────────────

#[tauri::command]
pub fn get_schedules(db: State<Arc<Database>>) -> Result<Vec<Schedule>, String> {
    db.get_schedules().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_email_logs(db: State<Arc<Database>>) -> Result<Vec<EmailLog>, String> {
    db.get_email_logs().map_err(|e| e.to_string())
}
