# Local validation record — 19 September 2026

The first implementation pass is committed locally on `refinement-local`. No push, production deployment, infrastructure change or external email test was performed. The candidate site runs at http://localhost:8091/ from the compiled release binary and binds to loopback by default.

## Passed

- 17 Rust tests: rendering/routing/HEAD/sitemap, atomic record and notification creation, saved-record reopen, durable rate limits, duplicate retries, token tampering/expiry/extreme timestamp rejection, notification retry/backoff, expiry cascade, HTML submission/receipt redirects, cross-site rejection, escaped preserved input, request-size rejection and spoofed forwarding headers.
- `cargo fmt --check`, Clippy with warnings denied, locked optimized release build.
- CSS, image and favicon builds; npm audit reports zero known vulnerabilities after dependency updates.
- Graphical browser: contributor introduction submitted and confirmed. Ministry inquiry submitted with page JavaScript explicitly disabled using native controls; confirmed saved. Browser JavaScript and viewport overrides restored after testing.
- Interactive Lynx: both ministry and contributor forms filled through keyboard navigation, cookies accepted, successful native POST and confirmation page. No JS challenge or hidden thank-you content required.
- A Lynx link-spacing issue and a browser Origin/referrer-policy mismatch were found and fixed during review. Form redirects now use the configured absolute origin to avoid Lynx's relative-redirect warning.
- All five public pages inspected at 320, 375, 768 and 1280 CSS pixels: no document overflow; one H1 each; no broken loaded images; no script tags. Desktop homepage, mobile contributor introduction and 320px ministry fields visually inspected.
- Local link/anchor/asset crawl: nine unique internal URLs checked successfully; legacy section anchors retained. Public partial/draft routes rejected by integration tests.
- Principal color checks: gold labels on paper 5.00:1, muted body on paper 5.70:1, button text 9.32:1, input border on field background 3.82:1. This is not a complete accessibility certification.
- Local database backup created with SQLite's online backup API; integrity check passed. All records, jobs, signing key and quota rows matched the source at the time of backup. A second isolated capture-only application opened the restored database and served the form and signed confirmation successfully; the test instance was then stopped.
- Main local service restarted from the optimized binary with all four synthetic review inquiries still present. No external notifications were sent.
- Production startup was tested without an actual `/data` mount, with the confirmation flag set: startup correctly refused. Merely setting an environment flag cannot enable an ephemeral production database.

## Not yet verified / release gates

- Fly account access is unavailable locally. The repository previously had no volume mount; candidate configuration now requires one. Actual volume attachment, machine count, restart/redeploy persistence and independent off-host backup scheduling/restoration remain unverified.
- Docker is not installed in this environment, so the full container build was not run. Rust release and Node asset builds passed separately. The container build must pass before release.
- Real SMTP account, sender, recipient and inbox receipt remain unconfigured/unverified. Local jobs are explicitly `captured`, not `sent`.
- Dedicated screen-reader review and actual 200% browser zoom remain open. Browser key automation did not support the attempted zoom shortcuts; narrow-width reflow was verified separately and is not substituted for this test.
- Five-reader comprehension exercise and one ministry representative walkthrough require actual participants. No usability results have been invented.
- Real available assignment, coordinator/supervision capacity, current founder facts, attributable dated work record and image reuse permissions need owner confirmation. The preview uses expressions of interest and makes unconfirmed work evidence explicit.
- Draft policies require operator/processor/backup details, retention acceptance and appropriate review. Public analytics was removed; no conversion analytics is running. The restricted CLI can record inquiry stages but does not establish real contribution outcomes.
- User review and sign-off are pending. The local CI change makes future deployments manual, but the remote repository's current push-to-main behavior remains unchanged until approved changes are pushed.

## Spam protection limits

Burst limits bound requests before database writes; persistent quotas cap accepted inquiries by network/email/site and survive restarts. Duplicate retries do not generate new records. Limits and token checks reduce abuse but do not prove a visitor is human. Distributed spam or targeted quota exhaustion can still deny legitimate intake; platform-level traffic controls, monitoring and an accessible email fallback remain necessary production considerations. No recipient auto-response is sent to unverified visitor addresses.

## Review sequence

1. Review homepage tone, visual character and contributor/ministry separation.
2. Try each local form with fictional details and inspect the restricted local record if desired.
3. Resolve the factual/operational decisions above.
4. Finish environment-dependent checks before authorizing any push or production deployment.
