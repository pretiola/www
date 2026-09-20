# Pretiola

A server-rendered Rust/Actix-web site with Tera templates and Tailwind CSS. Essential reading, navigation and forms work without JavaScript, including in Lynx. Node/Sharp builds images and favicons.

## Local preview

Requires current stable Rust and Node.js 22+; Python 3 runs the restricted inquiry tools. From this repository:

```
npm ci
npm run build:css
npm run build:images
npm run build:favicon
PORT=8091 cargo run
```

Open http://localhost:8091/. Restart after Rust or template edits (Tera loads templates at startup); rebuild CSS after styling changes. Local review pages are marked noindex and show a draft notice. Use fictional details only. The default notification mode is `capture`, which does not send messages. No production secrets are needed.

SQLite saves local inquiries in ignored `private/intake.sqlite3`. The worker captures notification jobs, retries real delivery when Telegram or SMTP is explicitly configured, and expires inquiry records after 90 days. Capture is not proof of real delivery.

For a containerized local preview, `docker compose up --build` uses a named development volume. Do not use `docker compose down -v` unless intentionally deleting its local test database.

## Telegram leads (local setup)

Copy `.env.example` to `.env` if it does not already exist. The local `.env` is ignored by Git and excluded from Docker builds. Set:

```dotenv
NOTIFICATION_MODE=telegram
TELEGRAM_BOT_TOKEN=your-bot-token
TELEGRAM_USER_IDS=123456789,987654321
```

Only one bot token is needed. IDs are positive numeric Telegram user IDs, comma-separated (up to ten recipients), not usernames. Each recipient must open this bot and send `/start` before it can message them. Restart the server from the repository directory after editing `.env`; exported shell variables take precedence over the file. Never paste credentials into chat or commit them.

Submit a fresh fictional inquiry through either form and wait up to 30 seconds. Check that Telegram receives the name, contact information, message and other supplied fields, and that its reference matches the confirmation page. Run `python3 scripts/inquiries.py status` to confirm the outbox reports `sent`. Existing `captured` jobs are not resent when changing modes.

SQLite saves the inquiry before confirming success; Telegram is delivery, not the only record. Existing spam limits still apply. Long messages are split. Failed deliveries retry with backoff, up to eight attempts; successful recipient/part receipts prevent ordinary retry duplicates. A crash after Telegram accepts a message but before its receipt is saved can still cause a duplicate. `sent` means Telegram accepted every part for every recipient, not that a person read it. After fixing a failure, use `python3 scripts/inquiries.py retry INQUIRY_ID`. Telegram copies are not removed by the application's 90-day database expiry.

Use `NOTIFICATION_MODE=capture` for tests that must not contact anyone. `MAIL_MODE` remains a legacy fallback only when `NOTIFICATION_MODE` is absent.

## Test

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
npm audit
lynx http://localhost:8091/for_ministries.html
```

Tests cover routing, storage/notification atomicity, restart persistence, duplicates, quotas, token tampering/expiry, cross-site submission rejection, error preservation and native HTML redirects. Browser and Lynx checks complement automated tests. Enable the essential form cookie for interactive Lynx; no JavaScript or CAPTCHA is required.

## Structure

- `src/routes.rs`: explicit public pages and sitemap. Adding a template does not make it public.
- `src/forms.rs`: native forms, cookie-bound tokens, validation/error rendering and receipt redirects.
- `src/inquiries.rs`: schema v1, durable records, duplicate protection and quotas.
- `src/notifications.rs`: local capture / Telegram / STARTTLS SMTP adapters, retry queue and expiry.
- `templates/`: shared layout, homepage, contributor/ministry forms, policies and receipts.
- `static/css/styles.css`: Tailwind source and responsive visual system; rebuild tracked `tailwind.css`.
- `scripts/inquiries.py`: restricted inquiry status, retrieval, stage updates, backup, deletion and failed-notification retry.

No public administration interface or inquiry export endpoint is exposed. Private records and secrets must not enter Git, Docker build contexts, static assets or application logs.

## Production is gated

Keep changes on `refinement-local` until user sign-off; do not push or deploy without authorization. The proposed CI runs checks on pushes/PRs; deployment requires explicit manual dispatch from main and the `production` environment. The existing remote workflow is unchanged until these local commits are approved and pushed.

Fly's root filesystem is ephemeral. The candidate deployment requires a separately provisioned Fly Volume mounted at `/data`, exactly one writer, independently stored backups and restore verification. The app refuses production startup without the real mount, expected database path and explicit storage confirmation. Telegram or SMTP credentials and confirmed recipients are also required. A mounted volume is not replication or protection against all hardware failure.

Read [operations and release gates](docs/operations.md) before any infrastructure or deployment work. Live Fly volumes, live email delivery and production restore have not been verified locally. If multi-machine availability is required, move to a shared database before scaling.

See [refinement checklist](docs/refinement-plan.md) and [local validation record](docs/validation.md) for implementation status, evidence and remaining factual/operational decisions.
