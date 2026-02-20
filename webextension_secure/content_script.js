browser.runtime.onMessage.addListener((msg) => {
  if (msg.action === 'fill') {
    fillLoginForm(msg.username, msg.password);
  }
});

function fillLoginForm(username, password) {
  // Find username field
  const usernameField = document.querySelector(
    'input[type="text"][name*="user" i], ' +
    'input[type="text"][id*="user" i], ' +
    'input[type="email"], ' +
    'input[autocomplete="username"]'
  );
  
  // Find password field
  const passwordField = document.querySelector('input[type="password"]');
  
  if (usernameField) {
    usernameField.value = username;
    usernameField.dispatchEvent(new Event('input', { bubbles: true }));
    usernameField.dispatchEvent(new Event('change', { bubbles: true }));
  }
  
  if (passwordField) {
    passwordField.value = password;
    passwordField.dispatchEvent(new Event('input', { bubbles: true }));
    passwordField.dispatchEvent(new Event('change', { bubbles: true }));
  }
}
