const { test } = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const vm = require('node:vm');
const source = readFileSync('static/js/analytics.js', 'utf8');
function boot({ choice, path = '/', production = true } = {}) {
  const cookies = new Map(choice ? [['pretiola_privacy', choice]] : []);
  const element = (dataset = {}) => ({ dataset, hidden: true, listeners: {}, focus() {},
    addEventListener(event, fn) { this.listeners[event] = fn; }, click() { this.listeners.click(); } });
  const buttons = Object.fromEntries(['none', 'analytics', 'all'].map(c => [c, element({ consent: c })]));
  const panel = element({ production: String(production) });
  panel.querySelectorAll = () => Object.values(buttons);
  const control = element(), close = element(), title = element(), scripts = [];
  let reloads = 0;
  const document = {
    getElementById: id => ({ 'privacy-choices': panel, 'privacy-close': close, 'privacy-choices-title': title })[id],
    querySelectorAll: () => [control], createElement: () => ({}),
    head: { appendChild: script => scripts.push(script) }, title: 'Pretiola',
    referrer: 'https://example.org/private?email=test@example.org#secret',
    get cookie() { return [...cookies].map(([k, v]) => k + '=' + v).join('; '); },
    set cookie(value) { const [pair] = value.split(';'); const [key, val] = pair.split('=');
      if (value.includes('Max-Age=0')) cookies.delete(key); else cookies.set(key, val); }
  };
  const window = { location: { origin: 'https://pretiola.org', hostname: 'pretiola.org', protocol: 'https:',
    pathname: path, search: '?email=private@example.org', hash: '#secret', reload: () => reloads++ } };
  vm.runInNewContext(source, { window, document, URL });
  return { window, panel, buttons, cookies, scripts, control, close, reloads: () => reloads,
    calls: () => window.dataLayer.map(args => Array.from(args)) };
}
test('no Google script, config or page view before consent or after rejecting', () => {
  const app = boot();
  assert.equal(app.panel.hidden, false);
  assert.equal(app.scripts.length, 0);
  assert.equal(app.calls().some(c => c[0] === 'config' || c[0] === 'event'), false);
  app.buttons.none.click();
  assert.equal(app.cookies.get('pretiola_privacy'), 'none');
  assert.equal(app.scripts.length, 0);
  assert.equal(app.panel.hidden, true);
  app.control.click();
  assert.equal(app.panel.hidden, false);
});
test('analytics-only loads original property once, denies ads and sanitizes URLs', () => {
  const app = boot(); app.buttons.analytics.click(); app.buttons.analytics.click();
  assert.equal(app.scripts.length, 1);
  assert.equal(app.scripts[0].src, 'https://www.googletagmanager.com/gtag/js?id=G-504XXPYDKL');
  const config = app.calls().find(c => c[0] === 'config');
  assert.equal(config[2].allow_google_signals, false);
  assert.equal(config[2].allow_ad_personalization_signals, false);
  assert.equal(config[2].page_location, 'https://pretiola.org/');
  assert.equal(config[2].page_referrer, 'https://example.org');
  const consent = app.calls().find(c => c[0] === 'consent' && c[1] === 'update')[2];
  assert.equal(consent.analytics_storage, 'granted');
  for (const key of ['ad_storage', 'ad_user_data', 'ad_personalization']) assert.equal(consent[key], 'denied');
});
test('advertising features require separate all consent; withdrawal disables and clears', () => {
  const app = boot({ choice: 'all' });
  assert.equal(app.calls().find(c => c[0] === 'config')[2].allow_google_signals, true);
  app.cookies.set('_ga', 'test'); app.cookies.set('_ga_504XXPYDKL', 'test'); app.cookies.set('pretiola_form', 'keep');
  app.buttons.none.click();
  assert.equal(app.window['ga-disable-G-504XXPYDKL'], true);
  assert.equal(app.cookies.has('_ga'), false);
  assert.equal(app.cookies.has('_ga_504XXPYDKL'), false);
  assert.equal(app.cookies.get('pretiola_form'), 'keep');
  assert.equal(app.cookies.get('pretiola_privacy'), 'none');
  assert.equal(app.reloads(), 1);
});
test('local previews, inquiry forms and receipt pages never load Google', () => {
  for (const options of [{ production: false }, { path: '/contribute.html' }, { path: '/contact.html' },
    { path: '/ministry.html' }, { path: '/inquiry/private-receipt' }, { path: '/unknown' }]) {
    const app = boot({ ...options, choice: 'all' });
    assert.equal(app.scripts.length, 0, JSON.stringify(options));
    assert.equal(app.calls().some(c => c[0] === 'config' || c[0] === 'event'), false);
  }
});
