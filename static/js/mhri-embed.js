// Progressive enhancement: the painting and its link remain available without JS.
const painting = document.getElementById('mhri-artwork');
if (painting) {
  const frame = document.createElement('iframe');
  frame.title = 'The Good Shepherd — interactive MHRI painting';
  frame.src = '/static/mhri/index.html';
  frame.setAttribute('allow', 'fullscreen');
  // Touch gestures belong to page scrolling; animation continues inside the frame.
  const mouseInteraction = window.matchMedia('(min-width: 801px) and (hover: hover) and (pointer: fine)');
  const updateInteraction = () => {
    frame.tabIndex = mouseInteraction.matches ? 0 : -1;
    frame.title = mouseInteraction.matches
      ? 'The Good Shepherd — interactive MHRI painting'
      : 'The Good Shepherd — animated MHRI painting';
  };
  updateInteraction();
  mouseInteraction.addEventListener('change', updateInteraction);
  painting.replaceChildren(frame);
}
