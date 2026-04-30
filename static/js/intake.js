(function () {
    'use strict';

    const form = document.getElementById('intake-form');
    if (!form) return;

    const submitBtn = document.getElementById('intake-submit');
    const submitLabel = document.getElementById('intake-submit-label');
    const submitSpinner = document.getElementById('intake-submit-spinner');
    const formError = document.getElementById('intake-form-error');
    const formShell = document.getElementById('intake-shell');
    const successBlock = document.getElementById('intake-success');
    const problemTextarea = document.getElementById('intake-problem');
    const problemCount = document.getElementById('intake-problem-count');

    const FALLBACK_EMAIL = 'contact@pretiola.org';
    const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    const WEB3FORMS_ENDPOINT = 'https://api.web3forms.com/submit';

    const accessKeyMeta = document.querySelector('meta[name="web3forms-access-key"]');
    const accessKey = accessKeyMeta ? (accessKeyMeta.getAttribute('content') || '').trim() : '';

    const fieldSpec = [
        { id: 'intake-ministry-name', errId: 'err-ministry-name', label: 'Ministry name', kind: 'text' },
        { id: 'intake-contact-name', errId: 'err-contact-name', label: 'Contact name', kind: 'text' },
        { id: 'intake-role', errId: 'err-role', label: 'Role', kind: 'text' },
        { id: 'intake-email', errId: 'err-email', label: 'Email', kind: 'email' },
        { id: 'intake-country', errId: 'err-country', label: 'Country', kind: 'text' },
        { id: 'intake-donations-bucket', errId: 'err-donations-bucket', label: 'Donations range', kind: 'select' },
        { id: 'intake-problem', errId: 'err-problem', label: 'Problem description', kind: 'textarea', max: 500 },
        { id: 'intake-consent', errId: 'err-consent', label: 'Consent', kind: 'checkbox' }
    ];

    function setError(field, message) {
        const el = document.getElementById(field.id);
        const errEl = document.getElementById(field.errId);
        if (message) {
            errEl.textContent = message;
            errEl.classList.remove('hidden');
            el.setAttribute('aria-invalid', 'true');
            el.classList.add('border-brand-oxblood');
            el.classList.remove('border-brand-gold/40');
        } else {
            errEl.textContent = '';
            errEl.classList.add('hidden');
            el.removeAttribute('aria-invalid');
            el.classList.remove('border-brand-oxblood');
            el.classList.add('border-brand-gold/40');
        }
    }

    function setRadioError(message) {
        const errEl = document.getElementById('err-canonical-status');
        if (message) { errEl.textContent = message; errEl.classList.remove('hidden'); }
        else { errEl.textContent = ''; errEl.classList.add('hidden'); }
    }

    function setRailsError(message) {
        const errEl = document.getElementById('err-payment-rails');
        if (message) { errEl.textContent = message; errEl.classList.remove('hidden'); }
        else { errEl.textContent = ''; errEl.classList.add('hidden'); }
    }

    function validateField(field) {
        const el = document.getElementById(field.id);
        const value = field.kind === 'checkbox' ? el.checked : (el.value || '').trim();

        if (field.kind === 'checkbox') {
            if (!value) { setError(field, 'Please confirm to proceed.'); return false; }
            setError(field, ''); return true;
        }
        if (!value) { setError(field, `${field.label} is required.`); return false; }
        if (field.kind === 'email' && !EMAIL_RE.test(value)) {
            setError(field, 'Please enter a valid email address.'); return false;
        }
        if (field.max && value.length > field.max) {
            setError(field, `Maximum ${field.max} characters.`); return false;
        }
        setError(field, ''); return true;
    }

    function validateCanonicalStatus() {
        const checked = form.querySelector('input[name="canonical_status"]:checked');
        if (!checked) { setRadioError('Please select your canonical status.'); return false; }
        setRadioError(''); return true;
    }

    function validatePaymentRails() {
        const checked = form.querySelectorAll('input[name="payment_rails"]:checked');
        if (checked.length === 0) { setRailsError('Please select at least one option.'); return false; }
        setRailsError(''); return true;
    }

    function validateAll(silent) {
        let firstInvalid = null;

        for (const field of fieldSpec) {
            if (silent) {
                const el = document.getElementById(field.id);
                const value = field.kind === 'checkbox' ? el.checked : (el.value || '').trim();
                let ok = true;
                if (field.kind === 'checkbox') ok = !!value;
                else if (!value) ok = false;
                else if (field.kind === 'email' && !EMAIL_RE.test(value)) ok = false;
                else if (field.max && value.length > field.max) ok = false;
                if (!ok) return { ok: false };
            } else {
                if (!validateField(field) && !firstInvalid) {
                    firstInvalid = document.getElementById(field.id);
                }
            }
        }

        if (silent) {
            const canon = form.querySelector('input[name="canonical_status"]:checked');
            if (!canon) return { ok: false };
            const rails = form.querySelectorAll('input[name="payment_rails"]:checked');
            if (rails.length === 0) return { ok: false };
            return { ok: true };
        }

        const canonOk = validateCanonicalStatus();
        if (!canonOk && !firstInvalid) firstInvalid = form.querySelector('input[name="canonical_status"]');
        const railsOk = validatePaymentRails();
        if (!railsOk && !firstInvalid) firstInvalid = form.querySelector('input[name="payment_rails"]');

        return { ok: !firstInvalid, firstInvalid };
    }

    function refreshSubmitState() {
        const result = validateAll(true);
        submitBtn.disabled = !result.ok;
    }

    fieldSpec.forEach(field => {
        const el = document.getElementById(field.id);
        el.addEventListener('blur', () => validateField(field));
        el.addEventListener('input', () => {
            if (el.getAttribute('aria-invalid') === 'true') validateField(field);
            refreshSubmitState();
        });
        if (field.kind === 'checkbox' || field.kind === 'select') {
            el.addEventListener('change', () => { validateField(field); refreshSubmitState(); });
        }
    });

    form.querySelectorAll('input[name="canonical_status"]').forEach(r => {
        r.addEventListener('change', () => { validateCanonicalStatus(); refreshSubmitState(); });
    });
    form.querySelectorAll('input[name="payment_rails"]').forEach(r => {
        r.addEventListener('change', () => { validatePaymentRails(); refreshSubmitState(); });
    });

    if (problemTextarea && problemCount) {
        const updateCount = () => {
            const len = problemTextarea.value.length;
            problemCount.textContent = `${len} / 500`;
            problemCount.classList.toggle('text-brand-oxblood', len > 500);
        };
        problemTextarea.addEventListener('input', updateCount);
        updateCount();
    }

    function showFormError(message) {
        formError.textContent = message;
        formError.classList.remove('hidden');
        formError.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }

    function hideFormError() {
        formError.textContent = '';
        formError.classList.add('hidden');
    }

    function setSubmitting(isSubmitting) {
        submitBtn.disabled = isSubmitting;
        submitLabel.textContent = isSubmitting ? 'Submitting…' : 'Submit Intake';
        submitSpinner.classList.toggle('hidden', !isSubmitting);
    }

    function buildPayload() {
        const ministryName = document.getElementById('intake-ministry-name').value.trim();
        const contactName = document.getElementById('intake-contact-name').value.trim();
        const role = document.getElementById('intake-role').value.trim();
        const email = document.getElementById('intake-email').value.trim();
        const country = document.getElementById('intake-country').value.trim();
        const canonical = (form.querySelector('input[name="canonical_status"]:checked') || {}).value || '';
        const bucket = document.getElementById('intake-donations-bucket').value;
        const rails = Array.from(form.querySelectorAll('input[name="payment_rails"]:checked')).map(c => c.value);
        const problem = document.getElementById('intake-problem').value.trim();
        const referral = document.getElementById('intake-referral').value.trim();
        const honeypot = document.getElementById('intake-website').value;

        const messageLines = [
            'New ministry intake submission:',
            '',
            `Ministry: ${ministryName}`,
            `Contact: ${contactName} (${role})`,
            `Email: ${email}`,
            `Country: ${country}`,
            `Canonical status: ${canonical}`,
            `Annual donations (EUR): ${bucket}`,
            `Current rails: ${rails.join(', ')}`,
            `Referral: ${referral || '(none)'}`,
            '',
            'Problem to solve:',
            problem
        ];

        return {
            access_key: accessKey,
            subject: `[Pretiola Intake] ${ministryName}`,
            from_name: 'Pretiola Intake',
            replyto: email,
            ministry_name: ministryName,
            contact_name: contactName,
            role: role,
            email: email,
            country: country,
            canonical_status: canonical,
            donations_bucket: bucket,
            payment_rails: rails.join(', '),
            problem: problem,
            referral: referral,
            message: messageLines.join('\n'),
            // Web3Forms honeypot: bots fill `botcheck`, humans never see it.
            botcheck: honeypot
        };
    }

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        hideFormError();

        const result = validateAll(false);
        if (!result.ok) {
            if (result.firstInvalid) {
                result.firstInvalid.focus();
                result.firstInvalid.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
            return;
        }

        if (!accessKey) {
            showFormError(`The intake form is not yet configured. Please email us directly at ${FALLBACK_EMAIL}.`);
            return;
        }

        setSubmitting(true);

        try {
            const resp = await fetch(WEB3FORMS_ENDPOINT, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
                body: JSON.stringify(buildPayload())
            });

            let body = null;
            try { body = await resp.json(); } catch (_) { /* fall through */ }

            if (resp.ok && body && body.success) {
                form.classList.add('hidden');
                successBlock.classList.remove('hidden');
                successBlock.scrollIntoView({ behavior: 'smooth', block: 'center' });
                if (window.gtag) {
                    window.gtag('event', 'ministry_intake_submitted', { event_category: 'engagement' });
                }
                return;
            }

            const reason = (body && body.message) || `Server returned ${resp.status}`;
            showFormError(`We could not submit your intake (${reason}). You can also email us directly at ${FALLBACK_EMAIL}.`);
        } catch (err) {
            showFormError(`We could not reach the intake endpoint (${err.message}). You can also email us directly at ${FALLBACK_EMAIL}.`);
        } finally {
            setSubmitting(false);
            refreshSubmitState();
        }
    });

    refreshSubmitState();
})();
