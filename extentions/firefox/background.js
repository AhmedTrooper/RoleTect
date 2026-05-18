// Background Script: Handles network requests to the local CSynth server (Firefox)
browser.runtime.onMessage.addListener((request, sender) => {
  if (request.action === "START_EXTRACTION") {
    return handleExtraction(request.selector);
  }
});

async function handleExtraction(selector) {
  try {
    // 1. Get Port and Secret from storage
    const settings = await browser.storage.local.get(['port', 'secret']);
    const port = settings.port || '14201';
    const secret = settings.secret;

    if (!secret) {
      throw new Error("Secret Key missing. Please set it in Extension Settings.");
    }

    // 2. Find the active tab
    const tabs = await browser.tabs.query({ active: true, currentWindow: true });
    const tab = tabs[0];
    if (!tab || !tab.id) throw new Error("No active tab found.");

    // 3. Inject content script
    await browser.scripting.executeScript({
      target: { tabId: tab.id },
      files: ["content.js"]
    });

    // 4. Extract data from page
    const domData = await browser.tabs.sendMessage(tab.id, {
      action: "GET_DOM",
      selector: selector
    });

    if (!domData.success) throw new Error(domData.error);

    // 5. POST to CSynth local server
    const serverUrl = `http://127.0.0.1:${port}/ingest`;
    
    const serverResponse = await fetch(serverUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        url: domData.url,
        raw_description: domData.html,
        secret: secret
      })
    });

    if (!serverResponse.ok) {
      const errorData = await serverResponse.json().catch(() => ({}));
      throw new Error(errorData.message || `Server rejected with status: ${serverResponse.status}`);
    }

    return { success: true };

  } catch (error) {
    console.error("CSynth Firefox Extension Error:", error);
    return { success: false, error: error.message };
  }
}
