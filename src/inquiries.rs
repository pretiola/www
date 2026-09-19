//! Durable intake and bounded abuse controls. No request contents enter logs.
use hmac::{Hmac, Mac};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    sync::Mutex,
    time::Duration,
};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;
pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Inquiry {
    pub token: String,
    pub name: String,
    pub email: String,
    pub organization: String,
    pub country: String,
    pub mode: String,
    pub message: String,
    pub availability: String,
    pub timezone: String,
    pub interests: String,
    pub referral: String,
    pub consent: String,
}
impl Inquiry {
    pub fn normalize(&mut self) {
        for s in [
            &mut self.name,
            &mut self.email,
            &mut self.organization,
            &mut self.country,
            &mut self.mode,
            &mut self.message,
            &mut self.availability,
            &mut self.timezone,
            &mut self.interests,
            &mut self.referral,
            &mut self.consent,
        ] {
            *s = s.trim().to_string();
        }
        self.email = self.email.to_lowercase();
    }
    pub fn validate(&self, kind: &str) -> BTreeMap<String, String> {
        let mut errors = BTreeMap::new();
        let mut field = |key: &str, label: &str, value: &str, required: bool, max: usize| {
            if required && value.is_empty() {
                errors.insert(key.into(), format!("Enter {label}."));
            } else if value.chars().count() > max
                || value
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t')
            {
                errors.insert(key.into(), format!("Use no more than {max} characters for {label}, without control characters."));
            }
        };
        field("name", "your name", &self.name, true, 120);
        field("email", "your email address", &self.email, true, 254);
        field("message", "a short description", &self.message, true, 2000);
        field(
            "organization",
            "your organization",
            &self.organization,
            kind == "ministry",
            160,
        );
        field(
            "country",
            "your country",
            &self.country,
            kind == "ministry",
            120,
        );
        for (key, value) in [
            ("availability", &self.availability),
            ("timezone", &self.timezone),
            ("interests", &self.interests),
            ("referral", &self.referral),
        ] {
            field(key, key, value, false, 200);
        }
        if self.email.parse::<lettre::Address>().is_err()
            || self.email.chars().any(char::is_whitespace)
        {
            errors.insert(
                "email".into(),
                "Enter a valid reply email address, such as name@example.org.".into(),
            );
        }
        if kind == "contributor"
            && !["professional", "mentoring", "learning"].contains(&self.mode.as_str())
        {
            errors.insert(
                "mode".into(),
                "Choose the kind of contribution you are interested in.".into(),
            );
        }
        if self.consent != "yes" {
            errors.insert(
                "consent".into(),
                "Confirm that we may reply about this inquiry.".into(),
            );
        }
        errors
    }
    fn record(&self, kind: &str) -> Self {
        let mut record = self.clone();
        record.token.clear();
        if kind == "ministry" {
            record.mode.clear();
            record.availability.clear();
            record.timezone.clear();
            record.interests.clear();
        } else {
            record.organization.clear();
            record.country.clear();
        }
        record
    }
}

pub struct Store {
    pub(crate) db: Mutex<Connection>,
    secret: Vec<u8>,
    burst: Mutex<HashMap<String, (i64, u32)>>,
}
#[derive(Debug)]
pub enum IntakeError {
    Limited,
    Full,
    Database(rusqlite::Error),
}
impl From<rusqlite::Error> for IntakeError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
impl Store {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
            }
        }
        let db = Connection::open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Self::from_connection(db)
    }
    pub fn memory() -> Self {
        Self::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    fn from_connection(db: Connection) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        db.busy_timeout(Duration::from_secs(3))?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS inquiries (
                id TEXT PRIMARY KEY, kind TEXT NOT NULL, created INTEGER NOT NULL,
                nonce TEXT NOT NULL UNIQUE, fingerprint TEXT NOT NULL, payload TEXT NOT NULL,
                stage TEXT NOT NULL DEFAULT 'received');
            CREATE INDEX IF NOT EXISTS inquiry_dedupe ON inquiries(fingerprint, created);
            CREATE TABLE IF NOT EXISTS outbox (
                inquiry_id TEXT PRIMARY KEY REFERENCES inquiries(id) ON DELETE CASCADE,
                state TEXT NOT NULL DEFAULT 'pending', attempts INTEGER NOT NULL DEFAULT 0,
                next_attempt INTEGER NOT NULL DEFAULT 0, last_error TEXT);
            CREATE TABLE IF NOT EXISTS limits (key TEXT PRIMARY KEY, expires INTEGER NOT NULL, count INTEGER NOT NULL);
            PRAGMA user_version=1;")?;
        let random = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
        db.execute(
            "INSERT OR IGNORE INTO settings(key,value) VALUES('signing_key', ?1)",
            [random.as_bytes()],
        )?;
        let secret = db.query_row(
            "SELECT value FROM settings WHERE key='signing_key'",
            [],
            |r| r.get(0),
        )?;
        Ok(Self {
            db: Mutex::new(db),
            secret,
            burst: Mutex::new(HashMap::new()),
        })
    }
    pub fn sign(&self, value: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).expect("HMAC accepts any key length");
        mac.update(value.as_bytes());
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    fn verify(&self, value: &str, signature: &str) -> bool {
        if signature.len() != 64 || !signature.is_ascii() {
            return false;
        }
        let Ok(bytes) = (0..64)
            .step_by(2)
            .map(|i| u8::from_str_radix(&signature[i..i + 2], 16))
            .collect::<Result<Vec<_>, _>>()
        else {
            return false;
        };
        let mut mac = HmacSha256::new_from_slice(&self.secret).unwrap();
        mac.update(value.as_bytes());
        mac.verify_slice(&bytes).is_ok()
    }
    pub fn token(&self, session: &str, kind: &str, at: i64) -> String {
        let body = format!("{at}.{}", Uuid::new_v4());
        format!(
            "{body}.{}",
            self.sign(&format!("form:{session}:{kind}:{body}"))
        )
    }
    pub fn check_token(&self, token: &str, session: &str, kind: &str, at: i64) -> bool {
        let Some((body, sig)) = token.rsplit_once('.') else {
            return false;
        };
        let Some((stamp, nonce)) = body.split_once('.') else {
            return false;
        };
        let Ok(stamp) = stamp.parse::<i64>() else {
            return false;
        };
        (0..=86400).contains(&(at - stamp))
            && Uuid::parse_str(nonce).is_ok()
            && self.verify(&format!("form:{session}:{kind}:{body}"), sig)
    }
    pub fn receipt(&self, id: &str) -> String {
        format!("{id}.{}", self.sign(&format!("receipt:{id}")))
    }
    pub fn check_receipt(&self, receipt: &str) -> bool {
        receipt.rsplit_once('.').is_some_and(|(id, sig)| {
            Uuid::parse_str(id).is_ok() && self.verify(&format!("receipt:{id}"), sig)
        })
    }
    // In-memory only: limits CPU/DB work without collecting rejected inquiry content.
    pub fn allow_attempt(&self, network: &str, at: i64) -> bool {
        let mut buckets = self.burst.lock().unwrap();
        buckets.retain(|_, (start, _)| at - *start < 600);
        let key = self.sign(&format!("burst:{network}"));
        if buckets.len() >= 10000 && !buckets.contains_key(&key) {
            return false;
        }
        let global = buckets.entry("global".into()).or_insert((at, 0));
        if global.1 >= 200 {
            return false;
        }
        global.1 += 1;
        let bucket = buckets.entry(key).or_insert((at, 0));
        if bucket.1 >= 20 {
            return false;
        }
        bucket.1 += 1;
        true
    }
    pub fn save(
        &self,
        kind: &str,
        form: &Inquiry,
        network: &str,
        at: i64,
    ) -> Result<String, IntakeError> {
        let payload = serde_json::to_string(&form.record(kind)).expect("serializable inquiry");
        let fingerprint = self.sign(&format!("payload:{kind}:{payload}"));
        let nonce = self.sign(&format!("nonce:{}", form.token));
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let previous: Option<String> = tx.query_row(
            "SELECT id FROM inquiries WHERE nonce=?1 OR (fingerprint=?2 AND created>?3) LIMIT 1",
            params![nonce, fingerprint, at-86400], |r| r.get(0)).optional()?;
        if let Some(id) = previous {
            return Ok(id);
        }
        let total: i64 = tx.query_row("SELECT count(*) FROM inquiries", [], |r| r.get(0))?;
        if total >= 20000 {
            return Err(IntakeError::Full);
        }
        tx.execute("DELETE FROM limits WHERE expires<=?1", [at])?;
        let keys = [
            (
                self.sign(&format!("network:{network}:{}", at / 3600)),
                3600,
                5,
            ),
            (
                self.sign(&format!("email:{}:{}", form.email, at / 3600)),
                3600,
                3,
            ),
            (format!("global:{}", at / 86400), 86400, 100),
        ];
        for (key, seconds, cap) in keys {
            let count: i64 = tx
                .query_row("SELECT count FROM limits WHERE key=?1", [&key], |r| {
                    r.get(0)
                })
                .optional()?
                .unwrap_or(0);
            if count >= cap {
                return Err(IntakeError::Limited);
            }
            tx.execute("INSERT INTO limits(key,expires,count) VALUES(?1,?2,1) ON CONFLICT(key) DO UPDATE SET count=count+1", params![key, (at/seconds+1)*seconds])?;
        }
        let id = Uuid::new_v4().to_string();
        tx.execute("INSERT INTO inquiries(id,kind,created,nonce,fingerprint,payload) VALUES(?1,?2,?3,?4,?5,?6)", params![id,kind,at,nonce,fingerprint,payload])?;
        tx.execute("INSERT INTO outbox(inquiry_id) VALUES(?1)", [&id])?;
        tx.commit()?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn form(i: usize) -> Inquiry {
        Inquiry {
            token: format!("token{i}"),
            name: "Test Person".into(),
            email: "test@example.org".into(),
            message: format!("Example inquiry {i}"),
            mode: "professional".into(),
            consent: "yes".into(),
            ..Default::default()
        }
    }
    #[test]
    fn tokens_reject_tampering_expiry_wrong_session_and_route() {
        let s = Store::memory();
        let token = s.token("session", "ministry", 100);
        assert!(s.check_token(&token, "session", "ministry", 101));
        assert!(!s.check_token(&token, "other", "ministry", 101));
        assert!(!s.check_token(&token, "session", "contributor", 101));
        assert!(!s.check_token(&token, "session", "ministry", 86501));
        assert!(!s.check_token(&(token + "a"), "session", "ministry", 101));
    }
    #[test]
    fn save_is_durable_atomic_and_deduplicated() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("private/intake.sqlite3");
        let s = Store::open(&path).unwrap();
        let id = s.save("contributor", &form(0), "network", 100).unwrap();
        assert_eq!(s.save("contributor", &form(0), "network", 101).unwrap(), id);
        let mut same = form(0);
        same.token = "new-token".into();
        assert_eq!(s.save("contributor", &same, "network", 102).unwrap(), id);
        let receipt = s.receipt(&id);
        drop(s);
        let s = Store::open(&path).unwrap();
        assert!(s.check_receipt(&receipt));
        let db = s.db.lock().unwrap();
        assert_eq!(
            db.query_row("SELECT count(*) FROM inquiries", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.query_row("SELECT count(*) FROM outbox", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn rate_limits_survive_restart_and_do_not_reject_duplicate_retry() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("private/intake.sqlite3");
        let s = Store::open(&path).unwrap();
        for i in 0..3 {
            s.save("contributor", &form(i), "network", 100).unwrap();
        }
        drop(s);
        let s = Store::open(&path).unwrap();
        assert!(matches!(
            s.save("contributor", &form(4), "different", 101),
            Err(IntakeError::Limited)
        ));
        assert!(s.save("contributor", &form(0), "network", 101).is_ok());
        assert!(s.save("contributor", &form(4), "network", 3700).is_ok());
    }
    #[test]
    fn failed_outbox_insert_rolls_back_inquiry() {
        let s = Store::memory();
        s.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_job BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(s.save("contributor", &form(0), "network", 100).is_err());
        assert_eq!(
            s.db.lock()
                .unwrap()
                .query_row("SELECT count(*) FROM inquiries", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn burst_is_bounded_and_recovers() {
        let s = Store::memory();
        for _ in 0..20 {
            assert!(s.allow_attempt("one", 100));
        }
        assert!(!s.allow_attempt("one", 101));
        assert!(s.allow_attempt("two", 101));
        assert!(s.allow_attempt("one", 701));
    }
}
