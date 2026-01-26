console.log("Background script loaded");

// Persistent connection to native host
const port = browser.runtime.connectNative("com.example.fill_test");

// Track which tab requested a fill
let fillTabId = null;

// Promise resolver for popup entry preview (optional)
let pendingPopupResolve = null;

// Listen for responses from native host
port.onMessage.addListener((response) => {
  console.log("Received from native host:", response);

  // Handle error responses
  if (response.error) {
    console.log("Native host error:", response.error);
    fillTabId = null;
    return;
  }

  // Handle credential lookup response
  if (response.found) {
    // Fill request (from Fill button)
    if (fillTabId !== null) {
      browser.tabs.sendMessage(fillTabId, {
        action: "fill",
        username: response.username,
        password: response.password
      });
      fillTabId = null;
      return;
    }

    // Popup preview request
    if (pendingPopupResolve) {
      pendingPopupResolve({
        username: response.username,
        password: response.password
      });
      pendingPopupResolve = null;
      return;
    }
    return;
  }

  // No credentials found
  if (response.found === false) {
    console.log("No credentials found");
    fillTabId = null;
    return;
  }

  console.warn("Unknown native response:", response);
});

// Helper to get active tab so that it can be reused
async function getActiveTab() {
  const tabs = await browser.tabs.query({ active: true, currentWindow: true });
  return tabs[0];
}

// Request credentials and fill the current page
async function requestentries() {
  const tab = await getActiveTab();
  if (!tab || !tab.id || !tab.url) return;

  fillTabId = tab.id;

  console.log("Requesting credentials for:", tab.url);

  port.postMessage({
    origin: tab.url // full URL; Rust normalizes
  });
}

// Request credentials for popup preview (no fill)
function requestCredentialsForPopup(origin) {
  return new Promise((resolve) => {
    pendingPopupResolve = resolve;
    port.postMessage({
      origin
    });
  });
}

// Listen for messages from popup/content scripts
browser.runtime.onMessage.addListener((msg) => {
  console.log("Background received message:", msg);

  switch (msg.action) {
    case "get-credentials":
      requestCredentialsForPopup(msg.origin);
      break;

    case "fill":
      console.log("Initiating fill request");
      requestentries();
      break;
  }
});
