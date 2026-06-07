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
