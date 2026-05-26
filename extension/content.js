// =============================================================================
// MailMate â€” Content Script (injected into Gmail)
//
// 1. Watches for compose windows opening (MutationObserver)
// 2. Injects "âœ¨ Generate Reply" button into compose toolbar
// 3. On click: extracts email thread, sends to background service worker
// 4. Receives streamed tokens and types them into compose body
// =============================================================================

(() => {
  'use strict';

  const BUTTON_ID_PREFIX = 'mailmate-btn-';
  let buttonCounter = 0;

  // â”€â”€ Gmail Compose Detection â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  /**
   * Gmail renders compose windows dynamically. We use MutationObserver
   * to detect when new compose containers appear in the DOM.
   */
  function watchForComposeWindows() {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (node.nodeType !== Node.ELEMENT_NODE) continue;

          // Gmail compose windows: look for the compose body (contenteditable div)
          const composeBoxes = node.querySelectorAll
            ? node.querySelectorAll('div[role="textbox"][aria-label="Message Body"], div[role="textbox"][g_editable="true"]')
            : [];

          for (const box of composeBoxes) {
            const composeContainer = findComposeContainer(box);
            if (composeContainer && !composeContainer.querySelector('[data-mailmate-injected]')) {
              injectButton(composeContainer, box);
            }
          }
        }
      }
    });

    observer.observe(document.body, { childList: true, subtree: true });

    // Also check for already-open compose windows
    setTimeout(scanExistingComposeWindows, 2000);
  }

  function scanExistingComposeWindows() {
    const composeBoxes = document.querySelectorAll(
      'div[role="textbox"][aria-label="Message Body"], div[role="textbox"][g_editable="true"]'
    );
    for (const box of composeBoxes) {
      const container = findComposeContainer(box);
      if (container && !container.querySelector('[data-mailmate-injected]')) {
        injectButton(container, box);
      }
    }
  }

  /**
   * Walk up the DOM from the compose textbox to find the compose container.
   * Gmail uses various container classes â€” we look for common patterns.
   */
  function findComposeContainer(textbox) {
    let el = textbox;
    for (let i = 0; i < 20; i++) {
      el = el.parentElement;
      if (!el) return null;

      // Full compose window or reply box
      if (
        el.classList.contains('M9') || // Full compose
        el.classList.contains('ip') || // Inline reply
        el.classList.contains('iN') || // Another compose variant
        (el.getAttribute('role') === 'dialog') // Modal compose
      ) {
        return el;
      }
    }
    // Fallback: return a reasonable ancestor
    return textbox.parentElement?.parentElement?.parentElement?.parentElement || null;
  }

  // â”€â”€ Button Injection â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  function injectButton(composeContainer, composeBody) {
    const btnId = `${BUTTON_ID_PREFIX}${buttonCounter++}`;

    // Find the toolbar â€” Gmail's bottom toolbar row
    const toolbar =
      composeContainer.querySelector('tr.btC') || // Standard toolbar
      composeContainer.querySelector('.btC') || // Variant
      composeContainer.querySelector('[role="toolbar"]') || // Accessibility
      findToolbar(composeContainer);

    if (!toolbar) {
      // Fallback: insert right before the compose body
      const btn = createButton(btnId, composeBody);
      composeBody.parentElement.insertBefore(btn, composeBody);
      return;
    }

    // Find the right place in the toolbar (after the send button area)
    const lastTd = toolbar.querySelector('td.gU.Up') || toolbar.lastElementChild;
    const td = document.createElement('td');
    td.appendChild(createButton(btnId, composeBody));

    if (lastTd && lastTd.parentElement === toolbar) {
      toolbar.insertBefore(td, lastTd);
    } else {
      toolbar.appendChild(td);
    }
  }

  function findToolbar(container) {
    // Look for the row that contains the Send button
    const sendBtn = container.querySelector('[role="button"][aria-label*="Send"]') ||
                    container.querySelector('[data-tooltip*="Send"]');
    if (sendBtn) {
      let el = sendBtn;
      for (let i = 0; i < 5; i++) {
        el = el.parentElement;
        if (!el) break;
        if (el.tagName === 'TR' || el.getAttribute('role') === 'toolbar') return el;
      }
    }
    return null;
  }

  function createButton(id, composeBody) {
    const wrapper = document.createElement('div');
    wrapper.setAttribute('data-mailmate-injected', 'true');
    wrapper.className = 'mailmate-wrapper';

    const btn = document.createElement('button');
    btn.id = id;
    btn.type = 'button';
    btn.className = 'mailmate-generate-btn';
    btn.innerHTML = `
      <svg class="mailmate-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3l1.912 5.813a2 2 0 0 0 1.272 1.278L21 12l-5.816 1.91a2 2 0 0 0-1.275 1.278L12 21l-1.91-5.812a2 2 0 0 0-1.277-1.278L3 12l5.813-1.91a2 2 0 0 0 1.278-1.277L12 3z"/>
      </svg>
      <span class="mailmate-btn-text">Generate Reply</span>
    `;

    btn.addEventListener('click', (e) => {
      e.preventDefault();
      e.stopPropagation();
      handleGenerate(btn, composeBody);
    });

    wrapper.appendChild(btn);
    return wrapper;
  }

  // â”€â”€ Generation Logic â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  async function handleGenerate(btn, composeBody) {
    // Prevent double-clicks
    if (btn.classList.contains('mailmate-loading')) return;

    // Extract the email thread / quoted text
    const prompt = extractEmailContext(composeBody);
    if (!prompt || prompt.trim().length < 5) {
      showToast('No email content found to reply to. Open a reply or paste the email text.');
      return;
    }

    // Set loading state
    btn.classList.add('mailmate-loading');
    btn.querySelector('.mailmate-btn-text').textContent = 'Generating...';

    // Clear compose body for the new response
    composeBody.focus();

    // Send to background service worker
    chrome.runtime.sendMessage({
      type: 'GENERATE_REPLY',
      payload: { prompt },
    });
  }

  /**
   * Extract the email content to reply to.
   * In Gmail replies, the original email is in a .gmail_quote div.
   * For forwarded emails, it's in a .gmail_extra div.
   */
  function extractEmailContext(composeBody) {
    const container = findComposeContainer(composeBody);
    if (!container) return '';

    // Try to find the quoted email text
    const quote =
      container.querySelector('.gmail_quote') ||
      container.querySelector('blockquote') ||
      container.querySelector('.gmail_extra');

    if (quote) {
      return quote.innerText.trim();
    }

    // For new compose or if no quote found, use whatever's in the compose body
    // This allows users to paste an email and generate a reply
    const bodyText = composeBody.innerText.trim();
    if (bodyText.length > 10) {
      return bodyText;
    }

    // Check for the entire message view (when replying inline)
    const messageView = document.querySelector('.a3s.aiL'); // Gmail message body
    if (messageView) {
      return messageView.innerText.trim();
    }

    return '';
  }

  // â”€â”€ Receive Tokens from Background â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  chrome.runtime.onMessage.addListener((message) => {
    if (message.type === 'GENERATE_TOKEN') {
      insertTokenIntoCompose(message.token);
    }

    if (message.type === 'GENERATE_LOG') {
      // Show real pipeline progress on the button
      const msg = message.message || '';
      let statusText = 'Generating...';
      if (msg.includes('[1/4]')) statusText = 'Embedding...';
      else if (msg.includes('[2/4]')) statusText = 'Searching KB...';
      else if (msg.includes('[3/4]')) statusText = 'Calling LLM...';
      else if (msg.includes('[4/4]')) statusText = 'Streaming...';
      document.querySelectorAll('.mailmate-generate-btn.mailmate-loading .mailmate-btn-text').forEach((el) => {
        el.textContent = statusText;
      });
    }

    if (message.type === 'GENERATE_DONE') {
      resetAllButtons();
      showToast('✨ Reply generated!');
    }

    if (message.type === 'GENERATE_ERROR') {
      resetAllButtons();
      showToast(`❌ ${message.error}`, true);
    }
  });

  /**
   * Insert a token (word/chunk) into the currently active compose body.
   */
  function insertTokenIntoCompose(token) {
    // Find the active compose body
    const activeCompose =
      document.querySelector('div[role="textbox"][aria-label="Message Body"]:focus') ||
      document.querySelector('div[role="textbox"][g_editable="true"]:focus') ||
      document.querySelector('div[role="textbox"][aria-label="Message Body"]');

    if (!activeCompose) return;

    // Append the token text
    activeCompose.focus();

    // Use insertText for proper undo support
    document.execCommand('insertText', false, token);
  }

  function resetAllButtons() {
    document.querySelectorAll('.mailmate-generate-btn.mailmate-loading').forEach((btn) => {
      btn.classList.remove('mailmate-loading');
      btn.querySelector('.mailmate-btn-text').textContent = 'Generate Reply';
    });
  }

  // â”€â”€ Toast Notifications â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  function showToast(message, isError = false) {
    // Remove existing toast
    document.querySelector('.mailmate-toast')?.remove();

    const toast = document.createElement('div');
    toast.className = `mailmate-toast ${isError ? 'mailmate-toast-error' : ''}`;
    toast.textContent = message;
    document.body.appendChild(toast);

    // Animate in
    requestAnimationFrame(() => toast.classList.add('mailmate-toast-show'));

    // Auto-remove after 4s
    setTimeout(() => {
      toast.classList.remove('mailmate-toast-show');
      setTimeout(() => toast.remove(), 300);
    }, 4000);
  }

  // â”€â”€ Initialize â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

  watchForComposeWindows();
  console.log('[MailMate] Content script loaded â€” watching for compose windows');
})();
