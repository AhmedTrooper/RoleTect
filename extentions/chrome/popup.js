// Initialize UI and Load Settings
document.addEventListener('DOMContentLoaded', () => {
  const hostTypeSelect = document.getElementById('hostType');
  const localPortInput = document.getElementById('localPort');
  const customHostInput = document.getElementById('customHost');
  const customPortInput = document.getElementById('customPort');
  const remoteUrlInput = document.getElementById('remoteUrl');
  
  const secretInput = document.getElementById('secret');
  const selectorInput = document.getElementById('selector');
  const excludeSelectorInput = document.getElementById('excludeSelector');
  const statusDiv = document.getElementById('status');

  // Advanced Mode Elements
  const modeToggle = document.getElementById('modeToggle');
  const simpleInputArea = document.getElementById('simpleInputArea');
  const advancedInputArea = document.getElementById('advancedInputArea');
  const siteSelector = document.getElementById('siteSelector');

  // Site Management Elements
  const siteMapList = document.getElementById('siteMapList');
  const newSiteTitle = document.getElementById('newSiteTitle');
  const newSiteSelector = document.getElementById('newSiteSelector');
  const newSiteExclude = document.getElementById('newSiteExclude');
  const addSiteBtn = document.getElementById('addSiteBtn');

  // Backup Elements
  const exportBtn = document.getElementById('exportBtn');
  const importFile = document.getElementById('importFile');
  const restoreOptions = document.getElementById('restoreOptions');
  const safeRestoreBtn = document.getElementById('safeRestoreBtn');
  const unsafeRestoreBtn = document.getElementById('unsafeRestoreBtn');

  let importedData = null;
  let siteMaps = [];

  // Built-in Defaults
  const BUILT_IN_SITES = [
    { title: "LinkedIn", selector: ".jobs-search__job-details--wrapper" },
    { title: "Indeed", selector: "#jobDescriptionText" },
    { title: "Glassdoor", selector: ".JobDetails_jobDescription__uW_fK" },
    { title: "Wellfound (AngelList)", selector: ".styles_jobDescription__xL_qW" },
    { title: "Y Combinator (WnYC)", selector: ".job-description" },
    { title: "Greenhouse", selector: "#content" },
    { title: "Lever", selector: ".section-wrapper.page-full-width" },
    { title: "Workday", selector: "[data-automation-id='jobPostingDescription']" }
  ];

  const hostGroups = {
    localhost: document.getElementById('localhostGroup'),
    customLocal: document.getElementById('customLocalGroup'),
    remote: document.getElementById('remoteGroup')
  };

  // Tab Switching
  document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.tab, .tab-content').forEach(el => el.classList.remove('active'));
      tab.classList.add('active');
      document.getElementById(tab.dataset.tab).classList.add('active');

      // Reload settings on tab change to discard unsaved changes
      loadSettings();
    });
  });

  function loadSettings() {
    chrome.storage.local.get([
      'hostType', 'localPort', 'customHost', 'customPort', 'remoteUrl', 'secret', 'selector', 'excludeSelector',
      'uiMode', 'siteMaps', 'activeSiteIndex'
    ], (result) => {
      if (result.hostType) {
        hostTypeSelect.value = result.hostType;
        updateHostGroups(result.hostType);
      }
      if (result.localPort) localPortInput.value = result.localPort;
      if (result.customHost) customHostInput.value = result.customHost;
      if (result.customPort) customPortInput.value = result.customPort;
      if (result.remoteUrl) remoteUrlInput.value = result.remoteUrl;
      if (result.secret) secretInput.value = result.secret;
      if (result.selector) selectorInput.value = result.selector;
      if (result.excludeSelector) excludeSelectorInput.value = result.excludeSelector;

      // Mode logic
      const isAdvanced = result.uiMode === 'advanced';
      modeToggle.checked = isAdvanced;
      updateModeUI(isAdvanced);

      // Site Maps logic: If empty, offer to load built-ins or just load what's there
      siteMaps = result.siteMaps || [];
      renderSiteMaps();
      
      if (result.activeSiteIndex !== undefined) {
        siteSelector.value = result.activeSiteIndex;
      }
    });
  }

  // Mode Toggle Logic
  modeToggle.addEventListener('change', () => {
    const isAdvanced = modeToggle.checked;
    updateModeUI(isAdvanced);
    chrome.storage.local.set({ uiMode: isAdvanced ? 'advanced' : 'simple' });
  });

  function updateModeUI(isAdvanced) {
    simpleInputArea.style.display = isAdvanced ? 'none' : 'block';
    advancedInputArea.style.display = isAdvanced ? 'block' : 'none';
  }

  function renderSiteMaps() {
    // Update Select Dropdown in Extract Tab
    siteSelector.innerHTML = siteMaps.length > 0 
      ? siteMaps.map((site, index) => `<option value="${index}">${site.title}</option>`).join('')
      : '<option value="">-- No Sites Saved --</option>';

    // Update List in Settings Tab
    let listHtml = siteMaps.map((site, index) => {
      const defaultSite = BUILT_IN_SITES.find(s => s.title === site.title);
      const isModified = defaultSite && (defaultSite.selector !== site.selector || defaultSite.exclude !== site.exclude);
      
      return `
        <div class="site-map-item" style="border-bottom: 1px solid rgba(255,255,255,0.05); padding-bottom: 8px;">
          <div style="flex-grow: 1;">
            <div style="display: flex; align-items: center; gap: 5px;">
              <strong style="font-size: 11px;">${site.title}</strong>
              ${defaultSite ? '<span style="font-size: 8px; background: var(--accent); color: white; padding: 1px 4px; border-radius: 3px;">Built-in</span>' : ''}
              ${isModified ? '<span style="font-size: 8px; background: #e3b341; color: black; padding: 1px 4px; border-radius: 3px;">Modified</span>' : ''}
            </div>
            <code style="display: block; font-size: 9px; color: var(--muted); margin-top: 2px;">IN: ${site.selector}</code>
            ${site.exclude ? `<code style="display: block; font-size: 9px; color: #f85149; margin-top: 2px;">EX: ${site.exclude}</code>` : ''}
          </div>
          <div style="display: flex; flex-direction: column; gap: 4px;">
            <button class="edit-btn secondary-btn" data-index="${index}" style="font-size: 9px !important; padding: 2px 5px !important; border-color: var(--accent) !important;">Edit</button>
            <button class="delete-btn" data-index="${index}">Delete</button>
            ${defaultSite ? `<button class="reset-site-btn secondary-btn" style="font-size: 9px !important; padding: 2px 5px !important;" data-title="${site.title}" data-index="${index}">Reset</button>` : ''}
          </div>
        </div>
      `;
    }).join('');

    // Add "Load Built-ins" button if missing
    const missingBuiltIns = BUILT_IN_SITES.filter(b => !siteMaps.find(s => s.title === b.title));
    if (missingBuiltIns.length > 0) {
      listHtml += `
        <button id="loadBuiltInsBtn" class="secondary-btn" style="width: 100%; margin-top: 10px; font-size: 11px;">
          Load ${missingBuiltIns.length} Default Templates
        </button>
      `;
    }

    if (siteMaps.length > 0) {
      listHtml += `
        <button id="factoryResetBtn" class="secondary-btn" style="width: 100%; margin-top: 10px; font-size: 11px; border-color: #f8514944 !important; color: #f85149 !important;">
          Factory Reset All Sites
        </button>
      `;
    }

    siteMapList.innerHTML = listHtml;

    // Attach Events
    document.querySelectorAll('.delete-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const index = parseInt(e.target.dataset.index);
        siteMaps.splice(index, 1);
        saveSiteMaps();
      });
    });

    document.querySelectorAll('.reset-site-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const title = e.target.dataset.title;
        const index = parseInt(e.target.dataset.index);
        const defaultSite = BUILT_IN_SITES.find(s => s.title === title);
        if (defaultSite) {
          siteMaps[index].selector = defaultSite.selector;
          saveSiteMaps();
          showStatus(`${title} reset to default.`, "neutral");
        }
      });
    });

    document.querySelectorAll('.edit-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const index = parseInt(e.target.dataset.index);
        const site = siteMaps[index];
        newSiteTitle.value = site.title;
        newSiteSelector.value = site.selector;
        newSiteExclude.value = site.exclude || '';
        
        addSiteBtn.textContent = "Update Site Map";
        addSiteBtn.dataset.editIndex = index;
        newSiteTitle.focus();
      });
    });

    const loadBtn = document.getElementById('loadBuiltInsBtn');
    if (loadBtn) {
      loadBtn.addEventListener('click', () => {
        missingBuiltIns.forEach(b => siteMaps.push({ ...b }));
        saveSiteMaps();
        showStatus("Defaults loaded!", "success");
      });
    }

    const factoryBtn = document.getElementById('factoryResetBtn');
    if (factoryBtn) {
      factoryBtn.addEventListener('click', () => {
        if (confirm("This will delete all custom sites and reset built-ins. Proceed?")) {
          siteMaps = BUILT_IN_SITES.map(s => ({ ...s }));
          saveSiteMaps();
          showStatus("Factory reset complete.", "success");
        }
      });
    }
  }

  function saveSiteMaps() {
    chrome.storage.local.set({ siteMaps }, () => {
      renderSiteMaps();
    });
  }

  // Add/Update Site Map
  addSiteBtn.addEventListener('click', () => {
    const title = newSiteTitle.value.trim();
    const selector = newSiteSelector.value.trim();
    const exclude = newSiteExclude.value.trim();

    if (!title || !selector) {
      showStatus("Please enter both Title and Selector.", "error");
      return;
    }

    if (addSiteBtn.dataset.editIndex !== undefined) {
      const index = parseInt(addSiteBtn.dataset.editIndex);
      siteMaps[index] = { title, selector, exclude };
      delete addSiteBtn.dataset.editIndex;
      addSiteBtn.textContent = "Add Site Map";
      showStatus("Site map updated!", "success");
    } else {
      siteMaps.push({ title, selector, exclude });
      showStatus("Site map added!", "success");
    }

    newSiteTitle.value = '';
    newSiteSelector.value = '';
    newSiteExclude.value = '';
    saveSiteMaps();
  });

  // Site Selector Change
  siteSelector.addEventListener('change', () => {
    chrome.storage.local.set({ activeSiteIndex: siteSelector.value });
  });

  // Export Logic
  exportBtn.addEventListener('click', () => {
    chrome.storage.local.get(null, (allData) => {
      const blob = new Blob([JSON.stringify(allData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `roletect_backup_${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showStatus("Backup exported!", "success");
    });
  });

  // Import Logic
  importFile.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        importedData = JSON.parse(event.target.result);
        restoreOptions.style.display = 'block';
        showStatus("JSON valid. Choose Restore method.", "neutral");
      } catch (err) {
        showStatus("Invalid JSON file.", "error");
      }
    };
    reader.readAsText(file);
  });

  safeRestoreBtn.addEventListener('click', () => {
    if (!importedData) return;
    
    chrome.storage.local.get(['siteMaps'], (result) => {
      const currentMaps = result.siteMaps || [];
      const incomingMaps = importedData.siteMaps || [];
      
      // Merge unique by title
      const mergedMaps = [...currentMaps];
      incomingMaps.forEach(inMap => {
        if (!mergedMaps.find(m => m.title === inMap.title)) {
          mergedMaps.push(inMap);
        }
      });

      chrome.storage.local.set({ ...importedData, siteMaps: mergedMaps }, () => {
        showStatus("Safe Merge complete!", "success");
        restoreOptions.style.display = 'none';
        loadSettings();
      });
    });
  });

  unsafeRestoreBtn.addEventListener('click', () => {
    if (!importedData) return;
    chrome.storage.local.clear(() => {
      chrome.storage.local.set(importedData, () => {
        showStatus("Overwrite complete!", "success");
        restoreOptions.style.display = 'none';
        loadSettings();
      });
    });
  });

  // GitHub Button Logic
  document.getElementById('githubIssueBtn').addEventListener('click', () => {
    const baseUrl = "https://github.com/AhmedTrooper/RoleTect/issues/new";
    const title = encodeURIComponent("[Site Template] Add support for [SITE NAME]");
    const body = encodeURIComponent("### Site Name\n(e.g. Indeed, Glassdoor)\n\n### Selector\n(e.g. .job-details-container)\n\n### Sample URL\n(optional link to a job post)");
    window.open(`${baseUrl}?title=${title}&body=${body}`, '_blank');
  });

  // Load saved settings initially
  loadSettings();

  // Save Selector
  document.getElementById('saveSelectorBtn').addEventListener('click', () => {
    const selector = selectorInput.value.trim() || 'body';
    const excludeSelector = excludeSelectorInput.value.trim();
    chrome.storage.local.set({ selector, excludeSelector }, () => {
      showStatus("Selectors saved!", "success");
    });
  });

  // Reset Selector
  document.getElementById('resetSelectorBtn').addEventListener('click', () => {
    selectorInput.value = 'body';
    excludeSelectorInput.value = '';
    chrome.storage.local.set({ selector: 'body', excludeSelector: '' }, () => {
      showStatus("Selectors reset to default.", "neutral");
    });
  });

  // Host Type Change Logic
  hostTypeSelect.addEventListener('change', (e) => {
    updateHostGroups(e.target.value);
  });

  function updateHostGroups(selectedType) {
    Object.keys(hostGroups).forEach(type => {
      hostGroups[type].style.display = (type === selectedType) ? 'block' : 'none';
    });
  }

  // Save Settings
  document.getElementById('saveSettingsBtn').addEventListener('click', () => {
    const hostType = hostTypeSelect.value;
    const localPort = localPortInput.value.trim() || '14207';
    const customHost = customHostInput.value.trim();
    const customPort = customPortInput.value.trim() || '14207';
    const remoteUrl = remoteUrlInput.value.trim();
    const secret = secretInput.value.trim();

    let finalHost = '';

    if (hostType === 'localhost') {
      finalHost = `http://127.0.0.1:${localPort}`;
    } else if (hostType === 'customLocal') {
      const h = customHost || '127.0.0.1';
      finalHost = `http://${h}:${customPort}`;
    } else if (hostType === 'remote') {
      finalHost = remoteUrl;
      // Force HTTPS for remote URLs (security best practice)
      if (finalHost) {
        if (finalHost.startsWith('http://')) {
          finalHost = finalHost.replace('http://', 'https://');
        } else if (!finalHost.startsWith('https://')) {
          finalHost = 'https://' + finalHost;
        }
      }
    }

    // Remove trailing slash for consistency
    if (finalHost.endsWith('/')) {
      finalHost = finalHost.slice(0, -1);
    }

    if (!finalHost) {
      showStatus("Please provide a valid Host or URL.", "error");
      return;
    }

    const saveAction = () => {
      chrome.storage.local.set({ 
        host: finalHost, 
        hostType, 
        localPort, 
        customHost, 
        customPort, 
        remoteUrl, 
        secret 
      }, () => {
        showStatus("Settings saved successfully!", "success");
      });
    };

    // If it's a remote URL, we must request permission dynamically
    if (hostType === 'remote' || (hostType === 'customLocal' && customHost !== '127.0.0.1' && customHost !== 'localhost')) {
      try {
        const origin = new URL(finalHost).origin + '/*';
        chrome.permissions.request({
          origins: [origin]
        }, (granted) => {
          if (granted) {
            saveAction();
          } else {
            showStatus("Permission denied. Cannot save custom host.", "error");
          }
        });
      } catch (e) {
        showStatus("Invalid URL format.", "error");
      }
    } else {
      saveAction();
    }
  });

  // Extract and Send
  document.getElementById('extractBtn').addEventListener('click', () => {
    let selector = 'body';
    let excludeSelector = '';

    if (modeToggle.checked) {
      const index = siteSelector.value;
      if (index !== "" && siteMaps[index]) {
        selector = siteMaps[index].selector;
        excludeSelector = siteMaps[index].exclude || '';
      } else {
        showStatus("No site template selected.", "error");
        return;
      }
    } else {
      selector = selectorInput.value.trim() || 'body';
      excludeSelector = excludeSelectorInput.value.trim();
    }
    
    showStatus("Extracting content...", "neutral");

    chrome.runtime.sendMessage({ action: "START_EXTRACTION", selector, excludeSelector }, (response) => {
      if (response && response.success) {
        showStatus("Job ingested into Inbox vault!", "success");
      } else {
        const errorMsg = response?.error || "Connection failed. Is your RoleTect instance reachable?";
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
