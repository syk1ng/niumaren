use chrono::NaiveDate;

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
        let (is_hol, _, _source, _) = is_holiday(date, cache, 2026, |_, _| Ok(())).await.unwrap();
        assert!(!is_hol);
    }
}
