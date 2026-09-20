// Keep the iframe visual-only: all scrolling and focus stay in the parent page.
const painting = document.getElementById('mhri-artwork');
if (painting) {
  const frame = document.createElement('iframe');
  frame.title = 'The Good Shepherd — animated MHRI painting';
  frame.src = '/static/mhri/index.html?v=20260920-scroll-clouds';
  frame.tabIndex = -1;
  frame.setAttribute('inert', '');
  const mouseInteraction = window.matchMedia('(min-width: 801px) and (hover: hover) and (pointer: fine)');
  const sendHover = (active, x = 0, y = 0) => {
    frame.contentWindow?.postMessage({type: 'pretiola-art-hover', active, x, y}, window.location.origin);
  };
  painting.addEventListener('pointermove', event => {
    if (!mouseInteraction.matches || event.pointerType !== 'mouse' || event.buttons !== 0) return;
    const rect = painting.getBoundingClientRect();
    sendHover(true, (event.clientX - rect.left) / rect.width * 2 - 1,
      (event.clientY - rect.top) / rect.height * 2 - 1);
  }, {passive: true});
  painting.addEventListener('pointerleave', () => sendHover(false), {passive: true});
  painting.addEventListener('pointerdown', () => sendHover(false), {passive: true});
  mouseInteraction.addEventListener('change', () => sendHover(false));
  let scrollFrame = 0;
  let lastState = '';
  const updateScroll = () => {
    scrollFrame = 0;
    const rect = painting.getBoundingClientRect();
    const visible = !document.hidden && rect.bottom > 0 && rect.top < window.innerHeight;
    const mobile = !mouseInteraction.matches;
    // Normalize page scroll over the painting's visible passage through the viewport.
    const progress = Math.max(0, Math.min(1,
      (window.innerHeight - rect.top) / (window.innerHeight + rect.height)));
    const state = JSON.stringify({type: 'pretiola-art-scroll', mobile, visible,
      progress: mobile && visible ? progress : null});
    if (state !== lastState) {
      frame.contentWindow?.postMessage(JSON.parse(state), window.location.origin);
      lastState = state;
    }
  };
  const scheduleScroll = () => {
    if (!scrollFrame) scrollFrame = requestAnimationFrame(updateScroll);
  };
  window.addEventListener('scroll', scheduleScroll, {passive: true});
  window.addEventListener('resize', scheduleScroll, {passive: true});
  document.addEventListener('visibilitychange', scheduleScroll);
  mouseInteraction.addEventListener('change', scheduleScroll);
  const observer = new IntersectionObserver(scheduleScroll);
  observer.observe(painting);
  window.addEventListener('message', event => {
    if (event.source === frame.contentWindow && event.origin === window.location.origin
      && event.data?.type === 'pretiola-art-ready') {
      lastState = '';
      scheduleScroll();
    }
  });
  frame.addEventListener('load', () => { lastState = ''; scheduleScroll(); });
  painting.replaceChildren(frame);
  scheduleScroll();
}
