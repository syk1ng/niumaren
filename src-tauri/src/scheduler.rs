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

    pub async fn check_and_notify(&self) -> Result<(), String> {
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
            |k, v| self.db.set_setting(k, v).map_err(|e| format!("设置保存失败: {}", e))
        ).await.unwrap_or((false, "{}".to_string(), "error".to_string(), 0));

        if is_hol {
            // Still create a schedule record but mark as holiday
            let _ = self.db.create_schedule(0, &date_str);
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
            self.db.create_email_log(schedule_id, &person.email, &subject, "failed", Some("SMTP 未配置")).ok();
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
pub fn next_weekday(from: NaiveDate, target: Weekday) -> NaiveDate {
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
