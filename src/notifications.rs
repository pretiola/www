use crate::inquiries::{now, Store};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};
use rusqlite::params;
use std::time::Duration;

pub enum Delivery {
    Capture,
    Telegram {
        token: String,
        recipients: Vec<i64>,
    },
    Smtp {
        transport: Box<SmtpTransport>,
        from: Mailbox,
        to: Mailbox,
    },
}
impl Delivery {
    pub fn from_env(production: bool) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match std::env::var("NOTIFICATION_MODE")
            .or_else(|_| std::env::var("MAIL_MODE"))
            .as_deref()
            .unwrap_or("capture")
        {
            "telegram" => {
                let token = std::env::var("TELEGRAM_BOT_TOKEN")?;
                if token.trim().is_empty()
                    || !token
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || ":_-".contains(c))
                {
                    return Err("Invalid TELEGRAM_BOT_TOKEN".into());
                }
                let recipients = telegram_recipients(&std::env::var("TELEGRAM_USER_IDS")?)?;
                Ok(Self::Telegram { token, recipients })
            }
            "capture" if !production => Ok(Self::Capture),
            "smtp" => {
                let host = std::env::var("SMTP_HOST")?;
                let from = std::env::var("MAIL_FROM")?.parse()?;
                let to = std::env::var("INTAKE_EMAIL")?.parse()?;
                let username = std::env::var("SMTP_USERNAME")?;
                let password = std::env::var("SMTP_PASSWORD")?;
                let transport = SmtpTransport::starttls_relay(&host)?
                    .credentials(Credentials::new(username, password))
                    .timeout(Some(Duration::from_secs(15)))
                    .build();
                Ok(Self::Smtp {
                    transport: Box::new(transport),
                    from,
                    to,
                })
            }
            _ => Err(
                "Choose a configured smtp or telegram notification mode; capture is local only"
                    .into(),
            ),
        }
    }
    fn send(&self, id: &str, kind: &str) -> Result<&'static str, ()> {
        match self {
            Self::Capture => Ok("captured"),
            Self::Telegram { .. } => Err(()),
            Self::Smtp {
                transport,
                from,
                to,
            } => {
                // Keep private inquiry content out of notification emails. Retrieve through the operator CLI.
                let message = Message::builder().from(from.clone()).to(to.clone())
                    .message_id(Some(format!("{id}@pretiola.org")))
                    .subject(format!("Pretiola: new {kind} inquiry"))
                    .body(format!("A new {kind} inquiry has been saved.\nReference: {id}\n\nRetrieve it using the restricted inquiry tools. This notification contains no submitted personal details.\n"))
                    .map_err(|_| ())?;
                transport.send(&message).map_err(|_| ())?;
                Ok("sent")
            }
        }
    }
}
fn telegram_recipients(raw: &str) -> Result<Vec<i64>, &'static str> {
    let mut ids = Vec::new();
    for item in raw.split(',') {
        let id = item
            .trim()
            .parse::<i64>()
            .map_err(|_| "TELEGRAM_USER_IDS must be comma-separated positive numeric user IDs")?;
        if id <= 0 {
            return Err("TELEGRAM_USER_IDS must contain positive user IDs");
        }
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    if ids.is_empty() || ids.len() > 10 {
        return Err("Configure between one and ten Telegram recipients");
    }
    Ok(ids)
}
fn telegram_parts(id: &str, kind: &str, f: &crate::inquiries::Inquiry) -> Vec<String> {
    let mut body = String::new();
    for (label, value) in [
        ("Name", &f.name),
        ("Email", &f.email),
        ("Organization", &f.organization),
        ("Country", &f.country),
        ("Contribution", &f.mode),
        ("Message", &f.message),
        ("Availability", &f.availability),
        ("Time zone", &f.timezone),
        ("Interests", &f.interests),
        ("Referral", &f.referral),
    ] {
        if !value.is_empty() {
            body.push_str(&format!("{label}: {value}\n\n"));
        }
    }
    let chars: Vec<char> = body.chars().collect();
    chars
        .chunks(1600)
        .enumerate()
        .map(|(i, c)| {
            format!(
                "Pretiola: {kind} inquiry\nReference: {id}\nPart {}\n\n{}",
                i + 1,
                c.iter().collect::<String>()
            )
        })
        .collect()
}
fn telegram_send(token: &str, recipient: i64, text: &str) -> Result<(), ()> {
    // Never log reqwest errors: their URLs contain the bot credential.
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ())?;
    let response = client.post(format!("https://api.telegram.org/bot{token}/sendMessage"))
        .json(&serde_json::json!({"chat_id": recipient, "text": text, "link_preview_options": {"is_disabled": true}, "protect_content": true}))
        .send().map_err(|_| ())?;
    if !response.status().is_success() {
        return Err(());
    }
    let reply: serde_json::Value = response.json().map_err(|_| ())?;
    if reply["ok"] == true {
        Ok(())
    } else {
        Err(())
    }
}
impl Store {
    pub fn deliver_due(&self, delivery: &Delivery, at: i64) -> Result<usize, rusqlite::Error> {
        if let Delivery::Telegram { token, recipients } = delivery {
            self.db.lock().unwrap().execute_batch("CREATE TABLE IF NOT EXISTS telegram_receipts (inquiry_id TEXT NOT NULL REFERENCES inquiries(id) ON DELETE CASCADE, recipient INTEGER NOT NULL, part INTEGER NOT NULL, PRIMARY KEY(inquiry_id,recipient,part))")?;
            self.process_due(at, |id, kind| {
                self.deliver_telegram(id, kind, recipients, |recipient, text| {
                    telegram_send(token, recipient, text)
                })
            })
        } else {
            self.process_due(at, |id, kind| delivery.send(id, kind))
        }
    }
    fn deliver_telegram(
        &self,
        id: &str,
        kind: &str,
        recipients: &[i64],
        mut send: impl FnMut(i64, &str) -> Result<(), ()>,
    ) -> Result<&'static str, ()> {
        let payload: String = self
            .db
            .lock()
            .unwrap()
            .query_row("SELECT payload FROM inquiries WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .map_err(|_| ())?;
        let inquiry: crate::inquiries::Inquiry = serde_json::from_str(&payload).map_err(|_| ())?;
        let parts = telegram_parts(id, kind, &inquiry);
        let mut failed = false;
        for recipient in recipients {
            for (part, text) in parts.iter().enumerate() {
                let done: bool = self.db.lock().unwrap().query_row("SELECT EXISTS(SELECT 1 FROM telegram_receipts WHERE inquiry_id=?1 AND recipient=?2 AND part=?3)", params![id,recipient,part as i64], |r| r.get(0)).map_err(|_| ())?;
                if done {
                    continue;
                }
                if send(*recipient, text).is_err() {
                    failed = true;
                    break;
                }
                self.db
                    .lock()
                    .unwrap()
                    .execute(
                        "INSERT OR IGNORE INTO telegram_receipts VALUES(?1,?2,?3)",
                        params![id, recipient, part as i64],
                    )
                    .map_err(|_| ())?;
            }
        }
        if failed {
            Err(())
        } else {
            Ok("sent")
        }
    }
    fn process_due(
        &self,
        at: i64,
        mut send: impl FnMut(&str, &str) -> Result<&'static str, ()>,
    ) -> Result<usize, rusqlite::Error> {
        let jobs: Vec<(String, String, i64)> = {
            let db = self.db.lock().unwrap();
            let mut stmt = db.prepare("SELECT o.inquiry_id,i.kind,o.attempts FROM outbox o JOIN inquiries i ON i.id=o.inquiry_id WHERE o.state='pending' AND o.next_attempt<=?1 LIMIT 10")?;
            let rows = stmt
                .query_map([at], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<Result<_, _>>()?;
            rows
        };
        for (id, kind, attempts) in &jobs {
            // One worker per database deployment. A crash after delivery can retry the same Message-ID.
            let (state, error) = match send(id, kind) {
                Ok(state) => (state, None),
                Err(_) => (
                    if *attempts >= 7 { "failed" } else { "pending" },
                    Some("Notification transport failed; inspect notification configuration."),
                ),
            };
            self.db.lock().unwrap().execute("UPDATE outbox SET state=?1,attempts=attempts+1,next_attempt=?2,last_error=?3 WHERE inquiry_id=?4", params![state,at+60*(1_i64<<(*attempts).min(10)),error,id])?;
            if error.is_some() {
                log::warn!("Inquiry notification failed; inspect restricted outbox status");
            }
        }
        Ok(jobs.len())
    }
    pub fn expire_records(&self, at: i64) -> Result<(), rusqlite::Error> {
        let db = self.db.lock().unwrap();
        db.execute(
            "DELETE FROM inquiries WHERE created < ?1",
            [at - 90 * 86400],
        )?;
        db.execute("DELETE FROM limits WHERE expires <= ?1", [at])?;
        Ok(())
    }
}
pub fn start_worker(store: actix_web::web::Data<Store>, delivery: Delivery) {
    actix_web::rt::spawn(async move {
        let delivery = std::sync::Arc::new(delivery);
        loop {
            let store = store.clone();
            let delivery = delivery.clone();
            let result = actix_web::web::block(move || {
                store.expire_records(now())?;
                store.deliver_due(&delivery, now())
            })
            .await;
            if !matches!(result, Ok(Ok(_))) {
                log::error!("Inquiry worker unavailable; inspect local storage");
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inquiries::Inquiry;
    #[test]
    fn telegram_ids_are_validated_and_deduplicated() {
        assert_eq!(telegram_recipients("123, 456,123").unwrap(), vec![123, 456]);
        for bad in ["", "0", "-123", "abc", "123,"] {
            assert!(telegram_recipients(bad).is_err());
        }
    }
    #[test]
    fn telegram_splits_unicode_without_losing_content_or_exposing_tokens() {
        let f = Inquiry {
            name: "Test".into(),
            email: "test@example.invalid".into(),
            message: "🥨".repeat(2000),
            token: "SECRET-TOKEN".into(),
            ..Default::default()
        };
        let parts = telegram_parts("reference", "contributor", &f);
        assert!(parts.len() > 1);
        assert!(parts.iter().all(|p| p.encode_utf16().count() <= 4096));
        let joined = parts.join("");
        assert_eq!(joined.matches('🥨').count(), 2000);
        assert!(joined.contains("test@example.invalid"));
        assert!(!joined.contains("SECRET-TOKEN"));
    }
    #[test]
    fn telegram_retries_only_undelivered_parts_and_recipients() {
        let s = Store::memory();
        let f = Inquiry {
            name: "Test".into(),
            message: "a".repeat(2000),
            token: "one".into(),
            ..Default::default()
        };
        let id = s.save("contributor", &f, "network", 100).unwrap();
        // Initialize receipt schema without processing jobs.
        s.deliver_due(
            &Delivery::Telegram {
                token: "unused".into(),
                recipients: vec![1, 2],
            },
            -1,
        )
        .unwrap();
        let mut calls = Vec::new();
        assert!(s
            .deliver_telegram(&id, "contributor", &[1, 2], |recipient, text| {
                calls.push((recipient, text.to_owned()));
                if recipient == 1 && text.contains("Part 2") {
                    Err(())
                } else {
                    Ok(())
                }
            })
            .is_err());
        assert_eq!(calls.len(), 4);
        calls.clear();
        assert_eq!(
            s.deliver_telegram(&id, "contributor", &[1, 2], |recipient, text| {
                calls.push((recipient, text.to_owned()));
                Ok(())
            }),
            Ok("sent")
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, 1);
        assert!(calls[0].1.contains("Part 2"));
        s.expire_records(100 + 91 * 86400).unwrap();
        assert_eq!(
            s.db.lock()
                .unwrap()
                .query_row("SELECT count(*) FROM telegram_receipts", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn failures_retry_without_losing_or_duplicating_inquiries() {
        let s = Store::memory();
        let f = Inquiry {
            token: "one".into(),
            ..Default::default()
        };
        s.save("ministry", &f, "network", 100).unwrap();
        assert_eq!(s.process_due(100, |_, _| Err(())).unwrap(), 1);
        assert_eq!(
            s.process_due(101, |_, _| panic!("must respect backoff"))
                .unwrap(),
            0
        );
        assert_eq!(s.process_due(160, |_, _| Ok("sent")).unwrap(), 1);
        assert_eq!(
            s.process_due(1000, |_, _| panic!("must not resend completed job"))
                .unwrap(),
            0
        );
        let db = s.db.lock().unwrap();
        assert_eq!(
            db.query_row("SELECT count(*) FROM inquiries", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.query_row("SELECT state FROM outbox", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "sent"
        );
    }
    #[test]
    fn capture_is_explicitly_not_email_delivery() {
        let s = Store::memory();
        s.save(
            "ministry",
            &Inquiry {
                token: "one".into(),
                ..Default::default()
            },
            "n",
            100,
        )
        .unwrap();
        s.deliver_due(&Delivery::Capture, 100).unwrap();
        assert_eq!(
            s.db.lock()
                .unwrap()
                .query_row("SELECT state FROM outbox", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "captured"
        );
        s.expire_records(100 + 91 * 86400).unwrap();
        assert_eq!(
            s.db.lock()
                .unwrap()
                .query_row("SELECT count(*) FROM outbox", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
