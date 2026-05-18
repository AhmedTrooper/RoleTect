// Runs inside the webpage context to read the DOM safely (Firefox)
browser.runtime.onMessage.addListener((request, sender) => {
  if (request.action === "GET_DOM") {
    try {
      const targetElement = document.querySelector(request.selector);

      if (!targetElement) {
        return Promise.resolve({ success: false, error: `Selector '${request.selector}' not found on this page.` });
      }

      return Promise.resolve({
        success: true,
        url: window.location.href,
        html: targetElement.innerText || targetElement.textContent || targetElement.outerHTML
      });
    } catch (err) {
      return Promise.resolve({ success: false, error: err.message });
    }
  }
});
