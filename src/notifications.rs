use crate::inquiries::{now, Store};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};
use rusqlite::params;
use std::time::Duration;

pub enum Delivery {
    Capture,
    Smtp {
        transport: Box<SmtpTransport>,
        from: Mailbox,
        to: Mailbox,
    },
}
impl Delivery {
    pub fn from_env(production: bool) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        match std::env::var("MAIL_MODE").as_deref().unwrap_or("capture") {
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
            _ => Err("Production requires MAIL_MODE=smtp and verified mail configuration".into()),
        }
    }
    fn send(&self, id: &str, kind: &str) -> Result<&'static str, ()> {
        match self {
            Self::Capture => Ok("captured"),
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
impl Store {
    pub fn deliver_due(&self, delivery: &Delivery, at: i64) -> Result<usize, rusqlite::Error> {
        self.process_due(at, |id, kind| delivery.send(id, kind))
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
                    Some("Notification transport failed; inspect mail configuration."),
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
