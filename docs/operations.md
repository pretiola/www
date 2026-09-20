# Intake operations and production gate

## Storage decision

The implementation uses SQLite for a single writer. Locally it lives at `private/intake.sqlite3`, outside static assets and excluded from Git and Docker contexts. Inquiry and outbox writes share one transaction; synchronous commits precede confirmation. Tokens and hashed quota keys retain the same signing key across restarts.

Fly root filesystems are ephemeral. Production requires a real Fly Volume mounted at `/data`, with the database at `/data/pretiola/intake.sqlite3`. The application checks `/proc/self/mountinfo` and refuses production startup without that mount, the expected path and explicit `PERSISTENT_STORAGE_CONFIRMED=true`. A normal container directory named `/data` is insufficient. Configuration alone is not evidence that the deployed volume exists.

The candidate `fly.toml` declares `intake_data`, one minimum running machine and no autostop, so notification retries run when there is no web traffic. Minimum running count does NOT enforce a maximum. Confirm exactly one machine/writer before release and do not scale this SQLite architecture horizontally. Independent Fly volumes are not synchronized. If multi-machine availability is needed, migrate the storage/outbox to a shared managed database before scaling; do not replicate by copying a live file.

A single volume protects against ordinary restart/deployment loss, not host failure. Accept the availability tradeoff explicitly, define recovery point/time targets, and configure independent off-host database backups. Fly snapshots (candidate retention 14 days) are a secondary recovery mechanism, not the primary backup. Backups contain private data and the signing key: restrict access, encrypt transport/storage, and keep them outside the repository and public assets.

On 20 September 2026, the user approved production deployment. The live app now uses one writer in yyz with encrypted volume intake_data (1 GiB) mounted at /data, scheduled Fly snapshots with 14-day retention, Telegram delivery and an HTTP health check. A synthetic SQLite record survived a machine restart and image update before launch. The production form test was saved, delivered to Telegram, included in a consistent backup downloaded to the operator computer, and retained across a production machine restart. See docs/releases/2026-09-20.md.

Outstanding: automatic backups to an independent provider, their monitoring owner and recovery targets remain to be configured. Current protection is persistent storage, scheduled Fly snapshots and a verified local launch backup; do not describe this as automated off-provider backup.

Sources: https://fly.io/docs/volumes/overview/ and https://fly.io/docs/reference/configuration/#the-mounts-section

## Before any production release

1. Obtain user sign-off on the local changes. Push/deploy remains separately gated; the revised workflow requires manual dispatch, an exact confirmation input and the production environment.
2. Inspect actual Fly machines and volumes. Confirm the attached volume in `yyz`, exactly one writer and no ingress bypassing Fly Proxy. `TRUST_FLY_PROXY=true` relies on Fly's controlled `Fly-Client-IP` header; never expose the application directly to untrusted clients in this mode.
3. Provision the approved persistent storage only after infrastructure authorization. Verify `/data` is mounted. Use only an approved synthetic inquiry for the persistence exercise.
4. Create and verify a consistent SQLite backup, restore to an isolated database, and verify the inquiry and notification job. Restart and redeploy the isolated service with the same mounted volume and verify the record still exists. Repeat the actual Fly persistence exercise before enabling public intake.
5. Configure an independently stored backup schedule and monitoring with a named owner. A backup file on the same volume is not sufficient. Test restore, deletion replay and expiry before relying on the backup.
6. Configure one delivery provider using secrets, not committed files: NOTIFICATION_MODE=telegram with TELEGRAM_BOT_TOKEN and TELEGRAM_USER_IDS, or NOTIFICATION_MODE=smtp with SMTP_HOST, SMTP_USERNAME, SMTP_PASSWORD, MAIL_FROM and INTAKE_EMAIL. Confirm all recipients; Telegram users must start the bot first. SMTP uses mandatory STARTTLS. Set PUBLIC_ORIGIN to the exact public origin. Verify an approved test inquiry reaches the intended Telegram chats or inbox; provider acceptance alone is insufficient.
7. Review retention (implemented default: 90 days for inquiries and linked jobs), authorized operators, snapshot/backup expiry, content facts and the draft policies. Export agreed engagements to an approved separate record before automatic inquiry expiry if longer retention is necessary.
8. Only then set PERSISTENT_STORAGE_CONFIRMED=true. This flag records verification; it does not provision a volume or replace the mount check.

## Restricted operator workflow

Run on the trusted host; there is no public inquiry/admin endpoint. The Python standard library tool is included in the container. Access to the machine/database is administrative access.

```
python3 scripts/inquiries.py status
python3 scripts/inquiries.py list
python3 scripts/inquiries.py show INQUIRY_ID
python3 scripts/inquiries.py stage INQUIRY_ID conversation
python3 scripts/inquiries.py retry INQUIRY_ID
python3 scripts/inquiries.py backup private/backups/intake-TIMESTAMP.sqlite3
python3 scripts/inquiries.py check-backup private/backups/intake-TIMESTAMP.sqlite3
python3 scripts/inquiries.py delete INQUIRY_ID
```

Use `--db /data/pretiola/intake.sqlite3` before the subcommand in production. `show` produces private JSON suitable for restricted export; do not put its output in ordinary logs. Backups use SQLite's online backup API, not a raw file copy that omits WAL data. Restore with the application stopped to a new private directory, verify integrity, preserve 0600 permissions, and point a capture-only isolated instance at it. Never point a restored test instance at live SMTP or Telegram. Reapply documented deletion requests and expire records older than 90 days before restored service resumes. Backup retention/deletion must be approved and enforced separately; deleting the live row does not erase existing snapshots.

SMTP notifications contain only route and reference. Telegram delivery sends full inquiry details to configured numeric user IDs; see the README for setup. Set NOTIFICATION_MODE=telegram with TELEGRAM_BOT_TOKEN and TELEGRAM_USER_IDS as secrets to use Telegram instead of SMTP. Each recipient must start the bot first. Telegram copies require separate deletion; database expiry does not delete them. Local capture marks jobs `captured`, never `sent`. Production retries start after one minute with exponential backoff and stop after eight attempts; inspect failed jobs and requeue explicitly after repair. Telegram tracks successful recipient/part deliveries to avoid ordinary retry duplicates. A crash between provider acceptance and recording its result can produce a duplicate notification. No new inquiry is created by notification retry.

## Abuse controls and limits

- 64 KiB URL-encoded request cap, per-field lengths and server-side validation; no attachments.
- HMAC-signed, route-specific 24-hour form tokens bound to an HttpOnly SameSite cookie; Origin/Fetch-Site checks when supplied. No JavaScript CAPTCHA and no misleading hidden success state. Tokens establish a legitimate form session, not humanity.
- At most 20 submission attempts per network key in ten minutes, and 200 globally; bounded in-memory buckets. IPv6 addresses grouped by /64. No arbitrary forwarded-header trust.
- Saved inquiries limited to 5/network/hour, 3/normalized email/hour and 100/site/day, stored transactionally in SQLite. Network and email rate keys are HMAC hashes; raw IPs are not stored. Fixed windows can permit adjacent-window bursts; request throttling adds a separate bound.
- Duplicate form tokens and identical recent payloads reuse the existing receipt before consuming accepted-submission quota. No autoresponse to visitor-supplied email addresses (avoids email bombing).
- Hard cap of 20,000 live inquiry records and automatic 90-day expiry. Sitewide caps intentionally trade availability under attack for bounded resource use. Review quotas as legitimate volume grows.
- Distributed/human spam can still pass these controls. Before public release, add platform-level request protection/monitoring for volumetric attacks; do not silently add an inaccessible browser challenge. Monitor 429 responses, capacity and notification failures. An email-specific cap can also be exhausted by someone who knows the address; a contact fallback remains available.

## Rollback

Keep the durable volume and backup independent from application rollback. Do not roll back to Web3Forms intake or an app image that writes to ephemeral disk as a shortcut. For an urgent fault, temporarily present the contact fallback while keeping data intact. Review future schema migrations for compatibility before rolling back a binary. Initial schema is version 1 and created idempotently.
