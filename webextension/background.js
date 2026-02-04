// Cross-browser compatibility
const browserAPI = (typeof browser !== 'undefined') ? browser : chrome;

console.log("Background script loaded");

const SERVER_URL = "http://127.0.0.1:9123";
let authToken = null;

// Retrieve token from extension storage
browserAPI.storage.local.get("authToken").then((result) => {
  if (result.authToken) {
    authToken = result.authToken;
    console.log("Token loaded from storage");
  } else {
    console.log("No token stored. User needs to provide one via popup.");
  }
}).catch((error) => {
  console.error("Failed to load token:", error);
});

// Listen for messages from popup and content scripts
browserAPI.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  console.log("Background received message:", msg);

  if (msg.action === "fill" && msg.url) {
    if (!authToken) {
      sendResponse({
        error: "Authentication token not set. Please configure in extension settings."
      });
      return true;
    }

    // Send request to localhost server
    fetch(SERVER_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        action: "fill",
        url: msg.url,
        token: authToken
      })
    })
      .then((response) => response.json())
      .then((data) => {
        console.log("Response from server:", data);
        sendResponse(data);
      })
      .catch((error) => {
        console.error("Error connecting to server:", error);
        sendResponse({
          error: "Failed to connect to password manager: " + error.message
        });
      });

    return true; // Keep channel open for async response
  }

  if (msg.action === "setToken" && msg.token) {
    authToken = msg.token;
    browserAPI.storage.local.set({ authToken: msg.token });
    sendResponse({ success: true });
    return true;
  }
});