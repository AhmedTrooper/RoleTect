// Initialize UI and Load Settings (Firefox)
document.addEventListener('DOMContentLoaded', async () => {
  const portInput = document.getElementById('port');
  const secretInput = document.getElementById('secret');
  const selectorInput = document.getElementById('selector');
  const statusDiv = document.getElementById('status');

  // Load saved settings
  const result = await browser.storage.local.get(['port', 'secret', 'selector']);
  if (result.port) portInput.value = result.port;
  if (result.secret) secretInput.value = result.secret;
  if (result.selector) selectorInput.value = result.selector;

  // Tab Switching
  document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.tab, .tab-content').forEach(el => el.classList.remove('active'));
      tab.classList.add('active');
      document.getElementById(tab.dataset.tab).classList.add('active');
    });
  });

  // Save Settings
  document.getElementById('saveSettingsBtn').addEventListener('click', async () => {
    const port = portInput.value.trim() || '14201';
    const secret = secretInput.value.trim();

    await browser.storage.local.set({ port, secret });
    showStatus("Settings saved successfully!", "success");
  });

  // Extract and Send
  document.getElementById('extractBtn').addEventListener('click', async () => {
    const selector = selectorInput.value.trim() || 'body';
    
    // Persist selector for convenience
    await browser.storage.local.set({ selector });

    showStatus("Processing extraction...", "neutral");

    try {
      const response = await browser.runtime.sendMessage({ action: "START_EXTRACTION", selector });
      if (response && response.success) {
        showStatus("Job ingested into vault!", "success");
      } else {
        throw new Error(response?.error || "Connection failed. Is RoleFlux open?");
      }
    } catch (err) {
      showStatus("Error: " + err.message, "error");
    }
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
