const menu = document.querySelector('.mobile-navigation');
if (menu) {
  const toggle = menu.querySelector('summary');
  // Handle one toggle per activation; retain native disclosure when JS is absent.
  toggle.addEventListener('click', event => {
    event.preventDefault();
    menu.open = !menu.open;
  });
  menu.addEventListener('click', event => {
    if (event.target.closest('a')) menu.open = false;
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && menu.open) {
      menu.open = false;
      menu.querySelector('summary').focus();
    }
  });
  document.addEventListener('click', event => {
    if (!menu.contains(event.target)) menu.open = false;
  });
}
