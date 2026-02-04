// Cross-browser compatibility
const browserAPI = (typeof browser !== 'undefined') ? browser : chrome;

let selectedEntry = null;

document.addEventListener('DOMContentLoaded', async () => {
  console.log('Popup loaded');
  
  const fillBtn = document.getElementById('fillBtn');
  const settingsBtn = document.getElementById('settingsBtn');
  const closeErrorBtn = document.getElementById('closeErrorBtn');
  const closeSelectionBtn = document.getElementById('closeSelectionBtn');
  const cancelSettingsBtn = document.getElementById('cancelSettingsBtn');
  const saveSettingsBtn = document.getElementById('saveSettingsBtn');
  const tokenInput = document.getElementById('tokenInput');
  
  console.log('Elements found:', {
    fillBtn: !!fillBtn,
    settingsBtn: !!settingsBtn,
    closeErrorBtn: !!closeErrorBtn,
    closeSelectionBtn: !!closeSelectionBtn,
    cancelSettingsBtn: !!cancelSettingsBtn,
    saveSettingsBtn: !!saveSettingsBtn,
    tokenInput: !!tokenInput
  });
  
  // Add event listeners
  if (fillBtn) fillBtn.addEventListener('click', handleFillClick);
  if (settingsBtn) settingsBtn.addEventListener('click', openSettingsModal);
  if (closeErrorBtn) closeErrorBtn.addEventListener('click', closeErrorModal);
  if (closeSelectionBtn) closeSelectionBtn.addEventListener('click', closeSelectionModal);
  if (cancelSettingsBtn) cancelSettingsBtn.addEventListener('click', closeSettingsModal);
  if (saveSettingsBtn) saveSettingsBtn.addEventListener('click', saveToken);

  // Load token from storage
  try {
    const result = await browserAPI.storage.local.get('authToken');
    if (result.authToken) {
      tokenInput.value = result.authToken;
    }
  } catch (error) {
    console.error('Failed to load token:', error);
  }
});

function openSettingsModal() {
  console.log('openSettingsModal called');
  const modal = document.getElementById('settingsModal');
  console.log('Settings modal element:', modal);
  if (modal) {
    modal.classList.add('show');
    console.log('Modal classes after add:', modal.className);
  } else {
    console.error('Settings modal not found!');
  }
}

function closeSettingsModal() {
  document.getElementById('settingsModal').classList.remove('show');
}

async function saveToken() {
  const token = document.getElementById('tokenInput').value.trim();
  if (!token) {
    showError('Error', 'Token cannot be empty');
    return;
  }
  try {
    await browserAPI.runtime.sendMessage({ action: 'setToken', token: token });
    await browserAPI.storage.local.set({ authToken: token });
    closeSettingsModal();
    showError('Success', 'Token saved successfully');
  } catch (error) {
    console.error('Failed to save token:', error);
    showError('Error', 'Failed to save token: ' + error.message);
  }
}

async function handleFillClick() {
  // Get current tab
  const [tab] = await browserAPI.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) {
    showError('Error', 'Could not determine current page');
    return;
  }

  // Request credentials from password manager
  try {
    const response = await browserAPI.runtime.sendMessage({
      action: 'fill',
      url: tab.url
    });

    if (!response) {
      showError('Error', 'No response from password manager');
      return;
    }

    // Handle error responses
    if (response.error) {
      showError('Error', response.error);
      return;
    }

    // Handle "not_found" response
    if (response.status === 'not_found') {
      showError('No Credentials', `No credentials found for ${getDomain(tab.url)}`);
      return;
    }

    // Handle "error" status
    if (response.status === 'error') {
      showError('Error', response.message || 'Unknown error');
      return;
    }

    // Single entry found - fill and close
    if (response.status === 'ok' && response.mode === 'single') {
      await fillPage(tab.id, response);
      window.close();
      return;
    }

    // Multiple entries - show selection modal
    if (response.status === 'ok' && response.mode === 'multiple' && response.entries) {
      showSelectionModal(response.entries, tab.id);
      return;
    }

  } catch (error) {
    console.error('Error:', error);
    showError('Error', `Failed to get credentials: ${error.message}`);
  }
}

function getDomain(url) {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

function showError(title, message) {
  document.getElementById('errorTitle').textContent = title;
  document.getElementById('errorMessage').textContent = message;
  document.getElementById('errorModal').classList.add('show');
}

function closeErrorModal() {
  document.getElementById('errorModal').classList.remove('show');
}

function showSelectionModal(entries, tabId) {
  const entryList = document.getElementById('entryList');
  entryList.innerHTML = '';

  entries.forEach((entry, index) => {
    const div = document.createElement('div');
    div.className = 'entry-item';
    div.innerHTML = `
      <div class="entry-name">${entry.url || 'Unnamed'}</div>
      <div class="entry-user">${entry.username || 'No username'}</div>
    `;
    div.addEventListener('click', async () => {
      await fillPage(tabId, entry);
      window.close();
    });
    entryList.appendChild(div);
  });

  document.getElementById('selectionModal').classList.add('show');
}

function closeSelectionModal() {
  document.getElementById('selectionModal').classList.remove('show');
}

async function fillPage(tabId, credentials) {
  try {
    await browserAPI.tabs.sendMessage(tabId, {
      action: 'fill',
      username: credentials.username,
      password: credentials.password
    });
  } catch (error) {
    console.error('Failed to fill page:', error);
    showError('Fill Failed', 'Could not fill the login form');
  }
}
