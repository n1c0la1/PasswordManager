// Cross-browser compatibility
const browserAPI = (typeof browser !== 'undefined') ? browser : chrome;

console.log("Background script loaded");

const SERVER_URL = "http://127.0.0.1:9123";
let authToken = null;

function getAuthToken() {
  if (authToken) {
    return Promise.resolve(authToken);
  }

  return browserAPI.storage.local.get("authToken").then((result) => {
    authToken = result.authToken || null;
    return authToken;
  });
}

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

browserAPI.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.authToken) {
    authToken = changes.authToken.newValue || null;
  }
});

// Listen for messages from popup and content scripts
browserAPI.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  console.log("Background received message:", msg);

  if (msg.action === "fill" && msg.url) {
    getAuthToken()
      .then((token) => {
        if (!token) {
          sendResponse({
            error: "Authentication token not set. Please configure in extension settings."
          });
          return;
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
            token: token
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
      })
      .catch((error) => {
        console.error("Failed to load token:", error);
        sendResponse({
          error: "Failed to load token from storage. Please open settings and save again."
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