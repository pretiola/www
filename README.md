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

Open http://localhost:8091/. Restart after Rust or template edits (Tera loads templates at startup); rebuild CSS after styling changes. Local review pages are marked noindex and show a draft notice. Use fictional details only. The default mail mode is `capture`, which does not send email. No production secrets are needed.

SQLite saves local inquiries in ignored `private/intake.sqlite3`. The worker captures notification jobs, retries real delivery only when SMTP is explicitly configured, and expires inquiry records after 90 days. Capture is not proof of real mail delivery.

For a containerized local preview, `docker compose up --build` uses a named development volume. Do not use `docker compose down -v` unless intentionally deleting its local test database.

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
- `src/notifications.rs`: local capture / STARTTLS SMTP adapter, retry queue and expiry.
- `templates/`: shared layout, homepage, contributor/ministry forms, policies and receipts.
- `static/css/styles.css`: Tailwind source and responsive visual system; rebuild tracked `tailwind.css`.
- `scripts/inquiries.py`: restricted inquiry status, retrieval, stage updates, backup, deletion and failed-notification retry.

No public administration interface or inquiry export endpoint is exposed. Private records and secrets must not enter Git, Docker build contexts, static assets or application logs.

## Production is gated

Keep changes on `refinement-local` until user sign-off; do not push or deploy without authorization. The proposed CI runs checks on pushes/PRs; deployment requires explicit manual dispatch from main and the `production` environment. The existing remote workflow is unchanged until these local commits are approved and pushed.

Fly's root filesystem is ephemeral. The candidate deployment requires a separately provisioned Fly Volume mounted at `/data`, exactly one writer, independently stored backups and restore verification. The app refuses production startup without the real mount, expected database path and explicit storage confirmation. SMTP credentials and a confirmed recipient are also required. A mounted volume is not replication or protection against all hardware failure.

Read [operations and release gates](docs/operations.md) before any infrastructure or deployment work. Live Fly volumes, live email delivery and production restore have not been verified locally. If multi-machine availability is required, move to a shared database before scaling.

See [refinement checklist](docs/refinement-plan.md) and [local validation record](docs/validation.md) for implementation status, evidence and remaining factual/operational decisions.
