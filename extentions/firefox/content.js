// content.js: Runs inside the webpage context to read the DOM safely (Firefox)
browser.runtime.onMessage.addListener((request, sender) => {
  if (request.action === "GET_DOM") {
    try {
      // Use querySelectorAll to catch multiple distinct sections
      const targetElements = document.querySelectorAll(request.selector);

      if (targetElements.length === 0) {
        return Promise.resolve({
          success: false,
          error: `Selector '${request.selector}' not found on this page.`,
        });
      }

      // Process each element found and join their results with double newlines
      const finalCleanData = Array.from(targetElements)
        .map(el => extractStructuredData(el, request.excludeSelector))
        .filter(text => text.length > 0)
        .join("\n\n");

      return Promise.resolve({
        success: true,
        url: window.location.href,
        html: finalCleanData,
      });
    } catch (err) {
      return Promise.resolve({ success: false, error: err.message });
    }
  }
});

/**
 * Cleans the DOM node and extracts a precise, structured text representation.
 */
function extractStructuredData(element, userExcludeSelector) {
  // 1. Clone the element so we don't accidentally mutate the live webpage
  const clone = element.cloneNode(true);

  // 2. Remove noisy tags that confuse AI and bloat the payload
  const noiseSelectors =
    "script, style, noscript, svg, img, iframe, nav, footer, button, header, aside, .visually-hidden";
  clone.querySelectorAll(noiseSelectors).forEach((el) => el.remove());

  // 3. Remove user-defined excluded elements if provided
  if (userExcludeSelector) {
    try {
      clone.querySelectorAll(userExcludeSelector).forEach((el) => el.remove());
    } catch (e) {
      console.error("Invalid user exclude selector:", userExcludeSelector);
    }
  }

  // 4. Temporarily append to DOM to allow browser to natively calculate innerText
  // This preserves proper spacing for lists, paragraphs, and headers without duplication.
  const tempContainer = document.createElement("div");
  tempContainer.style.position = "fixed";
  tempContainer.style.left = "-9999px";
  tempContainer.style.visibility = "hidden";
  tempContainer.appendChild(clone);
  document.body.appendChild(tempContainer);

  // 5. Extract text with native browser block spacing
  let rawText = tempContainer.innerText || "";

  // 6. Cleanup
  document.body.removeChild(tempContainer);

  // 7. Token-squash while preserving context and lists
  return rawText
    .replace(/[ \t]+/g, " ")       // Squash horizontal spaces
    .replace(/[\r\n]{3,}/g, "\n\n") // Squash massive vertical gaps into max 2 newlines
    .trim();
}
