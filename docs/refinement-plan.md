# Pretiola local refinement plan

Source: [Website Planning Guide](https://docs.google.com/document/d/1etif9ywoyxdXRJl0mYY8JxP4Sl2_tEt9n42qHTDRm4k/edit?tab=t.0), transitions T01–T13, plus the repository review of 19 September 2026.

## Working agreement

- Keep Rust, Actix-web, Tera and Tailwind. Preserve Catholic identity and the personal, warm visual character.
- Reading, navigation, validation and submission must work in Lynx and without JavaScript. JavaScript may enhance the experience.
- Implement in order on local branch `refinement-local`. Each numbered stage ends with a focused commit and recorded verification. Split a stage into smaller commits when that makes review easier.
- Preview the real application locally at http://localhost:8091/. Restart after template/server changes and rebuild CSS/assets when needed.
- No pushes, pull requests, production deployment, production infrastructure changes or real external test messages until explicitly authorized. Spam prevention and persistent Fly storage are explicit user priorities. Local commits are authorized. A push to main currently triggers deployment; release authorization must cover that consequence.
- Never commit credentials, submission data, local databases or private inquiry exports. Use synthetic test inquiries and a local notification sink.
- Mark work Proposed, Implemented, Verified or Accepted; do not equate a commit with acceptance. Record commit, test evidence, outstanding issues and user feedback against each stage.
- Do not invent opportunities, endorsements, founder statements, results or operating commitments. Prepare drafts and keep unresolved factual claims visible in this checklist.

## Ordered implementation and commit checklist

### 1. Establish the local baseline and close routing leaks — T12

- [x] Document reproducible startup, asset builds and test commands; preserve the existing passing baseline.
- [x] Inspect the generated CSS difference from initial setup before including it in any commit.
- [x] Correct robots.txt to the Pretiola sitemap; make `/` the canonical homepage and redirect `/index.html`.
- [x] Use explicit public routes/sitemap entries so partials and draft templates cannot become public pages automatically.
- [x] Triage the dependency audit findings from setup and apply compatible fixes where supported; separate any necessary breaking upgrade.
- Validation: approved pages/assets respond, old homepage redirects, partials/drafts return 404, sitemap excludes duplicates and private templates, existing tests updated for intentional route changes.
- Commit theme: `fix: establish canonical public routes and local baseline` (dependency fixes separately if needed).

### 2. Own the inquiry record before relying on email — T06, T07, T13

- [x] Add a first-party submission endpoint with server-side validation, bounded inputs and typed contributor/ministry records.
- [x] Decide and document durable storage and migrations before implementation. Evaluate a local SQLite database for development against the eventual Fly.io topology; do not assume an ephemeral container filesystem is durable or that one attached volume supports multiple machines.
- [x] Save inquiry and notification job atomically, then confirm receipt. Return an error rather than success if saving fails.
- [x] Add submission identifiers, duplicate protection and accessible abuse controls that require no JavaScript challenge.
- [x] Provide restricted retrieval/export and a deletion process; a public admin dashboard is not required.
- [x] Document backup and restore procedures and exercise them against synthetic records.
- Validation: valid submission survives process restart; malformed/oversized input is rejected; storage failure never shows success; resubmission does not duplicate the inquiry; records cannot be fetched through public routes; restored test data is readable.
- Commit theme: `feat: persist inquiries through the Pretiola server`.

### 3. Make notifications recoverable — T06, T07, T13

- [x] Add a notification adapter and local capture sink; keep email delivery separate from the saved record.
- [x] Record pending/sent/failed notification state, bounded retry behavior and actionable failures. Do not log inquiry bodies or credentials.
- [x] Define the eventual sender/recipient configuration and delivery acceptance test, without enabling live delivery locally.
- [x] Remove obsolete Web3Forms configuration after its final caller is replaced in stages 4–5.
- Validation: simulated provider outage preserves inquiries, retry succeeds, ordinary retries do not create new inquiry records, local notifications identify the correct inquiry route. Document that email delivery may need duplicate-tolerant handling.
- Commit theme: `feat: add recoverable inquiry notifications`.

### 4. Simplify ministry intake and make it native HTML — T07, T11

- [x] Move intake to `/for_ministries.html`, preserving a useful destination at the old homepage anchor.
- [x] Require organization, contact name, reply email, country and practical need. Move financial/canonical/payment questions to optional discovery or remove them from first contact.
- [x] Use native POST, meaningful labels, server-rendered errors with preserved input, and a real confirmation page after successful persistence.
- [x] Keep submit usable for validation; avoid the CSS-hidden thank-you and honeypot behavior exposed in Lynx. Make email fallback available before failure.
- [x] Remove the three-day reply promise unless its operational owner confirms it.
- Validation: complete an actual synthetic submission interactively in Lynx and a graphical browser; test invalid email, missing fields, storage failure, refresh/back and keyboard navigation.
- Commit theme: `feat: make ministry inquiries work without JavaScript`.

### 5. Add the contributor journey — T05, T06

- [x] Build `/contribute.html` with scoped professional work, mentoring/review and supervised learning clearly distinguished by real availability.
- [x] Add a separate introduction form: name, reply email, mode and experience required; availability, time zone, interests and referral optional.
- [x] Connect to the owned submission flow with route-specific validation and notification content.
- [x] Explain commitment, support and how to pause. Until a real assignment is confirmed, use honest expressions of interest rather than an invented vacancy or guaranteed placement.
- Validation: contributors can submit without ministry/finance fields; errors retain input; duplicate submissions are handled; saved record and local notification identify the contributor route; Lynx completion works.
- Commit theme: `feat: add a distinct contributor introduction journey`.

### 6. Rework the homepage and shared navigation — T01–T04

- [x] Draft the contributor-first invitation and practical work categories from the guide.
- [x] Order the page around invitation, practical work, participation process, attributable example, founder/faith and next action.
- [x] Use consistent About, How we help, Our work, contributor and ministry destinations throughout shared navigation/footer.
- [x] Preserve useful old anchors, repair the unfinished footer sentence, and expose contact/privacy/terms on every page.
- Validation: both audiences reach the appropriate form within two intentional clicks; links and anchors resolve; purpose and next action are legible in a mobile and Lynx reading pass.
- Commit theme: `feat: orient the homepage around practical participation`.
- Review checkpoint: local walkthrough of both complete journeys before further visual polishing.

### 7. Ground identity and work evidence — T08

- [x] Refine the founder/faith section while distinguishing independent Catholic inspiration from official endorsement.
- [ ] Replace broad partner-program claims with Pretiola's attributable contribution, date, deliverable, observed result and remaining uncertainty.
- [ ] Confirm biography, founder wording, partner permission and asset reuse before treating drafts as final.
- Validation: every material claim has an owner/source or remains explicitly unresolved; partner outcomes and Pretiola's work are distinguishable; no fabricated testimonials, logos, counts or vacancies.
- Commit theme: `content: clarify identity and attributable work`.

### 8. Align policies and measurement with actual behavior — T09, T10, T13

- [ ] Inventory actual form fields, processors, storage, access, retention, deletion, hosting and analytics. Update the privacy notice and policies to match the implementation.
- [x] Replace irrelevant financial/advertising boilerplate with accurate website/inquiry scope; flag legal wording requiring qualified review.
- [x] Decide analytics behavior and keep local development out of production measurement. If retained, use consent behavior appropriate to the agreed setup and send no personal information.
- [ ] Distinguish contributor path selection, form start and accepted submission. Never count clicks or email opens as completed contributions.
- [x] Define a restricted operational register for inquiry → conversation → match → commitment → accepted deliverable, with clear denominators and observation windows.
- Validation: inspect requests/storage against disclosures; success events follow persistence; form content never enters analytics; policy links are available at point of collection. Legal/operational acceptance remains separate from technical verification.
- Commit theme: `content: align policies and measurement with owned intake`.

### 9. Refine presentation and accessibility — T11, T12

- [x] Improve typography, spacing, section rhythm, contrast and responsive layout while retaining the existing character.
- [x] Correct heading hierarchy, link names, focus visibility, anchor offsets and mobile navigation state/Escape behavior. Keep navigation available when JavaScript is off.
- [x] Update page titles, descriptions, canonical and social metadata to the agreed positioning; use approved imagery only.
- Validation: keyboard, screen-reader-oriented checks, Lynx, JavaScript-off browser, 320/375/768px widths, desktop and 200% zoom; check clipped controls, text contrast and image alternatives. Record limitations rather than claiming blanket accessibility compliance.
- Commit theme: `style: refine accessible layouts and visual hierarchy`.
- Review checkpoint: local visual walkthrough with user feedback captured as explicit follow-up items.

### 10. Regressions, sign-off and release readiness — T01–T13

- [ ] Run appropriate Rust tests, asset build and production container build. Exercise storage migration, restoration and notification failure recovery with synthetic data.
- [ ] Review the complete change set and commit history; resolve remaining user feedback in focused local commits.
- [ ] Run the guide's formative comprehension exercise with five representative readers and one ministry representative when available; do not substitute automated checks for human results.
- [ ] Verify one real email receipt only after authorized test environment, recipient and sender are available. Until then, label live delivery unverified.
- [x] Present the working local site, completed checklist, remaining factual/operational decisions and deployment/rollback requirements for explicit sign-off.
- [x] Stop before pushing. User approval of the local changes does not silently authorize a production rollout unless that is included in the instruction.
- Commit theme: `test: verify complete inquiry and navigation journeys`, followed by focused feedback fixes.

## Decisions to resolve while independent work continues

1. Primary contributor audience and approved positioning (D1).
2. First available assignment, contribution mode, owner, time boundary and supervision capacity (D2).
3. Inquiry recipient, backup owner and realistic response promise (D3).
4. Approved founder statement, biography, work evidence and imagery permissions (D4).
5. Data retention, authorized readers, production storage/backup topology, mail delivery service, analytics choice and policy review (D5).

These decisions gate the affected final claims or production configuration, not unrelated local development. The guide's later operational proof—one supervised contribution completed and accepted—cannot be marked done by shipping website code.

## Progress record — 19 September 2026

Local implementation is ready for review, not production release. See [validation record](validation.md) and [storage/operations gate](operations.md). All work is committed on `refinement-local`; nothing has been pushed.

| Stage | Status | Evidence / remaining work |
| --- | --- | --- |
| 1 | Verified locally | Explicit routes, canonical redirect, sitemap; npm audit clean |
| 2 | Verified locally | Atomic SQLite records/jobs, quotas, duplicates, reopen and restored-instance checks |
| 3 | Verified locally | Capture mode and retry/backoff tests; real SMTP receipt pending |
| 4 | Verified locally | Simplified native ministry form, successful interactive Lynx and JS-disabled browser submissions |
| 5 | Verified locally | Separate contributor flow, successful Lynx/browser submissions; real assignment still needs confirmation |
| 6 | Implemented; local review ready | New homepage, consistent navigation/footer, working anchors; human comprehension exercise pending |
| 7 | Draft pending factual acceptance | No invented first-person quote or outcomes; work record, current biography and image permissions need owner confirmation |
| 8 | Partially verified | Policies match local architecture; analytics removed, restricted stage register implemented. Production processors/backups/operator details and policy review pending. Public conversion instrumentation deferred while tracking is disabled |
| 9 | Verified within recorded scope | Desktop/mobile screenshots, layout widths, labels, keyboard controls and contrast checked; dedicated screen-reader and 200% zoom review remain open |
| 10 | Local technical verification complete within available tools | 17 Rust tests, clean Clippy, release build, assets, browser/Lynx and local restore passed. Docker engine unavailable; live Fly persistence, real mail, representative readers and user sign-off remain open |

### Implementation choices for review

- No JavaScript CAPTCHA: native cookie-bound signed tokens, bounded requests, burst throttling, durable per-network/per-email/global quotas and duplicate protection. Determined/distributed spam still needs operational monitoring and platform request protection.
- SQLite is a single-writer candidate, not a promise of high availability. Production requires a real `/data` mount, confirmed backup/restore and a deliberate one-machine deployment; use a shared database before horizontal scaling.
- Default inquiry retention is 90 days; confirm this alongside backup retention before release.
- Navigation stays visible on small screens, removing the need for a JavaScript hamburger control.
- Tracking is disabled, so the private inquiry register is the current measurement source. Conversion funnel instrumentation needs a separately agreed analytics decision.
- Existing imagery is reused; ownership/reuse permission remains a factual publication check.
