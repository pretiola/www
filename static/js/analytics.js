(() => {
  const id = 'G-504XXPYDKL';
  const key = 'pretiola_privacy';
  const panel = document.getElementById('privacy-choices');
  if (!panel) return;
  const controls = document.querySelectorAll('[data-privacy-settings]');
  const close = document.getElementById('privacy-close');
  const maxAge = 180 * 86400;
  let choice = document.cookie.split('; ').find(v => v.startsWith(key + '='))?.split('=')[1];
  if (!['none', 'analytics', 'all'].includes(choice)) choice = null;
  const eligible = panel.dataset.production === 'true'
    && ['/', '/privacy.html', '/terms.html'].includes(window.location.pathname);
  let loaded = false;
  window.dataLayer = window.dataLayer || [];
  window.gtag = function () { window.dataLayer.push(arguments); };
  const consent = value => ({
    analytics_storage: value === 'analytics' || value === 'all' ? 'granted' : 'denied',
    ad_storage: value === 'all' ? 'granted' : 'denied',
    ad_user_data: value === 'all' ? 'granted' : 'denied',
    ad_personalization: value === 'all' ? 'granted' : 'denied'
  });
  window.gtag('consent', 'default', consent('none'));
  window.gtag('set', 'ads_data_redaction', true);
  const clearCookies = () => {
    const parts = window.location.hostname.split('.');
    const domains = ['', ...parts.map((_, i) => '.' + parts.slice(i).join('.'))];
    for (const cookie of document.cookie.split(';')) {
      const name = cookie.trim().split('=')[0];
      if (!/^(_ga(?:_|$)|_gid$|_gat(?:_|$)|_gcl_)/.test(name)) continue;
      for (const domain of domains) {
        document.cookie = name + '=; Max-Age=0; Path=/' + (domain ? '; Domain=' + domain : '');
      }
    }
  };
  const start = value => {
    if (!eligible || value === 'none' || !value || loaded) return;
    loaded = true;
    window['ga-disable-' + id] = false;
    window.gtag('consent', 'update', consent(value));
    window.gtag('js', new Date());
    const options = {
      send_page_view: false,
      allow_google_signals: value === 'all',
      allow_ad_personalization_signals: value === 'all',
      cookie_expires: maxAge,
      cookie_update: false,
      page_location: window.location.origin + window.location.pathname,
      page_referrer: ''
    };
    try { options.page_referrer = document.referrer ? new URL(document.referrer).origin : ''; } catch {}
    window.gtag('config', id, options);
    window.gtag('event', 'page_view', {
      send_to: id, page_location: options.page_location,
      page_referrer: options.page_referrer, page_title: document.title
    });
    const script = document.createElement('script');
    script.async = true;
    script.src = 'https://www.googletagmanager.com/gtag/js?id=' + id;
    document.head.appendChild(script);
  };
  const show = () => {
    panel.hidden = false;
    close.hidden = !choice;
    document.getElementById('privacy-choices-title').focus();
  };
  controls.forEach(button => {
    button.hidden = false;
    button.addEventListener('click', show);
  });
  close.addEventListener('click', () => {
    panel.hidden = true;
    controls[0]?.focus({ preventScroll: true });
  });
  panel.querySelectorAll('[data-consent]').forEach(button => button.addEventListener('click', () => {
    const next = button.dataset.consent;
    document.cookie = key + '=' + next + '; Max-Age=' + maxAge + '; Path=/; SameSite=Lax'
      + (window.location.protocol === 'https:' ? '; Secure' : '');
    if (loaded && next !== choice) {
      // Stop collection before leaving this page; the next load starts with the new choice.
      window['ga-disable-' + id] = true;
      window.gtag('consent', 'update', consent(next));
      clearCookies();
      window.location.reload();
      return;
    }
    choice = next;
    if (choice === 'none') clearCookies();
    start(choice);
    panel.hidden = true;
    controls[0]?.focus({ preventScroll: true });
  }));
  if (!choice) panel.hidden = false;
  if (choice === 'none') clearCookies();
  start(choice);
})();
