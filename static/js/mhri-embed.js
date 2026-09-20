// Keep the iframe visual-only: all scrolling and focus stay in the parent page.
const painting = document.getElementById('mhri-artwork');
if (painting) {
  const frame = document.createElement('iframe');
  frame.title = 'The Good Shepherd — animated MHRI painting';
  frame.src = '/static/mhri/index.html?v=20260920-scroll';
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
  painting.replaceChildren(frame);
}
