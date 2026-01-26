document.addEventListener('DOMContentLoaded', async () => {
  const usernameInput = document.getElementById('username');
  const passwordInput = document.getElementById('password');
  const addBtn = document.getElementById('addBtn');
  const fillBtn = document.getElementById('fillBtn');

  // Get current tab URL
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) return;

  const url = new URL(tab.url);
  const origin = url.origin; // or url.href if you truly want full URL

  // Ask background/Rust for credentials for this URL
  browser.runtime.sendMessage({
    action: 'get-credentials',
    origin
  }).then((cred) => {
    if (cred) {
      usernameInput.value = cred.username || '';
      passwordInput.value = cred.password || '';
    }
  });

  // Store / update credentials
  addBtn.addEventListener('click', () => {
    const username = usernameInput.value;
    const password = passwordInput.value;

    browser.runtime.sendMessage({
      action: 'store',
      origin,
      username,
      password
    });

    alert(`Stored credentials for ${origin}`);
  });

  // Fill current page (content script does actual filling)
  fillBtn.addEventListener('click', () => {
    browser.runtime.sendMessage({
      action: 'fill',
      origin
    });
  });
});
