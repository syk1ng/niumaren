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
