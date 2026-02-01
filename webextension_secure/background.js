// Listen for toolbar icon clicks
browser.action.onClicked.addListener(async (tab) => {
  try {
    // Read clipboard
    const clipboardText = await navigator.clipboard.readText();
    
    if (!clipboardText || !clipboardText.includes('\n')) {
      // Invalid format - notify user
      browser.scripting.executeScript({
        target: { tabId: tab.id },
        func: () => alert('No valid credentials in clipboard.\n\nUse CLI: get <name> --copy')
      });
      return;
    }
    
    // Parse clipboard: "username\npassword"
    const lines = clipboardText.split('\n');
    const username = lines[0].trim();
    const password = lines[1].trim();
    
    if (!username || !password) {
      browser.scripting.executeScript({
        target: { tabId: tab.id },
        func: () => alert('Invalid clipboard format')
      });
      return;
    }
    
    // Send to content script
    await browser.tabs.sendMessage(tab.id, {
      action: 'fill',
      username: username,
      password: password
    });
    
    // Clear clipboard immediately after successful fill
    await navigator.clipboard.writeText('');
    
  } catch (error) {
    console.error('Fill error:', error);
    browser.scripting.executeScript({
      target: { tabId: tab.id },
      func: (msg) => alert('Failed to fill: ' + msg),
      args: [error.message]
    });
  }
});
