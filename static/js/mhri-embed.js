// Progressive enhancement: the painting and its link remain available without JS.
const painting = document.getElementById('mhri-artwork');
if (painting) {
  const frame = document.createElement('iframe');
  frame.title = 'The Good Shepherd — interactive MHRI painting';
  frame.src = '/static/mhri/index.html';
  frame.setAttribute('allow', 'fullscreen');
  painting.replaceChildren(frame);
}
