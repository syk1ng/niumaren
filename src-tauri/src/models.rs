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
