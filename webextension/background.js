console.log("Background script loaded");

// Connect to native host (runs as thread in CLI process)
const port = browser.runtime.connectNative("password_manager");

// Listen for responses from native host
port.onMessage.addListener((response) => {
  console.log("Received from native host:", response);
  // Response is handled by the popup that sent the message
});

// Handle errors
port.onDisconnect.addListener(() => {
  console.error("Native host disconnected:", browser.runtime.lastError?.message);
});

// Listen for messages from popup and content scripts
browser.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  console.log("Background received message:", msg);

  if (msg.action === 'fill' && msg.url) {
    // Send request to native host
    port.postMessage({
      origin: msg.url
    });

    // Listen for response
    const responseListener = (response) => {
      console.log("Forwarding response to popup:", response);
      sendResponse(response);
      port.onMessage.removeListener(responseListener);
    };
    
    port.onMessage.addListener(responseListener);
    
    // Timeout after 5 seconds
    setTimeout(() => {
      port.onMessage.removeListener(responseListener);
      sendResponse({ error: "Request timeout" });
    }, 5000);

    return true; // Keep channel open for async response
  }
});
