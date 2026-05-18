// Initialize UI and Load Settings
document.addEventListener('DOMContentLoaded', () => {
  const portInput = document.getElementById('port');
  const secretInput = document.getElementById('secret');
  const selectorInput = document.getElementById('selector');
  const statusDiv = document.getElementById('status');

  // Load saved settings
  chrome.storage.local.get(['port', 'secret', 'selector'], (result) => {
    if (result.port) portInput.value = result.port;
    if (result.secret) secretInput.value = result.secret;
    if (result.selector) selectorInput.value = result.selector;
  });

  // Tab Switching
  document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.tab, .tab-content').forEach(el => el.classList.remove('active'));
      tab.classList.add('active');
      document.getElementById(tab.dataset.tab).classList.add('active');
    });
  });

  // Save Settings
  document.getElementById('saveSettingsBtn').addEventListener('click', () => {
    const port = portInput.value.trim() || '14201';
    const secret = secretInput.value.trim();

    chrome.storage.local.set({ port, secret }, () => {
      showStatus("Settings saved successfully!", "success");
    });
  });

  // Extract and Send
  document.getElementById('extractBtn').addEventListener('click', () => {
    const selector = selectorInput.value.trim() || 'body';
    
    // Persist selector for convenience
    chrome.storage.local.set({ selector });

    showStatus("Extracting content...", "neutral");

    chrome.runtime.sendMessage({ action: "START_EXTRACTION", selector }, (response) => {
      if (response && response.success) {
        showStatus("Job ingested into vault!", "success");
      } else {
        const errorMsg = response?.error || "Connection failed. Is RoleFlux open?";
        showStatus("Error: " + errorMsg, "error");
      }
    });
  });

  function showStatus(msg, type) {
    statusDiv.textContent = msg;
    statusDiv.className = "";
    if (type === "success") statusDiv.classList.add('status-success');
    if (type === "error") statusDiv.classList.add('status-error');
    if (type === "neutral") {
      statusDiv.style.display = "block";
      statusDiv.style.color = "var(--text)";
    }
  }
});
