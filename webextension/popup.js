let selectedEntry = null;

document.addEventListener('DOMContentLoaded', async () => {
  const fillBtn = document.getElementById('fillBtn');
  const closeErrorBtn = document.getElementById('closeErrorBtn');
  const closeSelectionBtn = document.getElementById('closeSelectionBtn');
  
  // Add event listeners
  fillBtn.addEventListener('click', handleFillClick);
  closeErrorBtn.addEventListener('click', closeErrorModal);
  closeSelectionBtn.addEventListener('click', closeSelectionModal);
});

async function handleFillClick() {
  // Get current tab
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) {
    showError('Error', 'Could not determine current page');
    return;
  }

  // Request credentials from native host
  try {
    const response = await browser.runtime.sendMessage({
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

    // Handle "not found" response
    if (response.found === false) {
      showError('No Credentials', `No credentials found for ${getDomain(tab.url)}`);
      return;
    }

    // Single entry found - fill and close
    if (response.found === true) {
      await fillPage(tab.id, response);
      window.close();
      return;
    }

    // Multiple entries - show selection modal
    if (response.entries && Array.isArray(response.entries)) {
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
      <div class="entry-name">${entry.entryname || 'Unnamed'}</div>
      <div class="entry-user">${entry.username || '(no username)'}</div>
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
    await browser.tabs.sendMessage(tabId, {
      action: 'fill',
      username: credentials.username,
      password: credentials.password
    });
  } catch (error) {
    console.error('Failed to fill page:', error);
    showError('Fill Failed', 'Could not fill the login form');
  }
}
