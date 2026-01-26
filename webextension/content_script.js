function findLoginFields() {
  const pass = document.querySelector('input[type="password"]');
  if (pass) {
    const form = pass.form;
    const candidate = form
      ? form.querySelector('input[type="text"], input[type="email"], input:not([type])')
      : document.querySelector('input[type="text"], input[type="email"], input:not([type])');
    return { username: candidate, password: pass };
  }
  const inputs = Array.from(document.querySelectorAll('input'));
  const username = inputs.find(i => /user|email|login|username/i.test(i.name||i.id||i.placeholder||''));
  const password = inputs.find(i => i.type === 'password' || /pass|password/i.test(i.name||i.id||i.placeholder||''));
  return { username, password };
}

function fillFields(username, password) {
  const { username: userField, password: passField } = findLoginFields();
  if (userField && username) { userField.value = username; userField.dispatchEvent(new Event('input', { bubbles: true })); }
  if (passField && password) { passField.value = password; passField.dispatchEvent(new Event('input', { bubbles: true })); }
}

browser.runtime.onMessage.addListener((msg) => {
  if (msg && msg.action === 'fill') fillFields(msg.username, msg.password);
});
