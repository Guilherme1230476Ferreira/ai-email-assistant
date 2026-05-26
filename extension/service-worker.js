// =============================================================================
// MailMate — Background Service Worker (Manifest V3)
//
// Handles:
// 1. API authentication (stores/retrieves JWT from chrome.storage.local)
// 2. Proxies generate requests from content script → backend
// 3. Parses SSE stream and forwards tokens back to content script
// =============================================================================

const DEFAULT_API_URL = 'https://dreamy-swimwear-daffodil.ngrok-free.dev';

// ── Helpers ──────────────────────────────────────────────────────────────────

async function getApiUrl() {
  const { apiUrl } = await chrome.storage.local.get('apiUrl');
  return apiUrl || DEFAULT_API_URL;
}

async function getToken() {
  const { token } = await chrome.storage.local.get('token');
  return token || null;
}

// ── Extension Heartbeat Ping ──────────────────────────────────────────────────

async function pingExtension() {
  const token = await getToken();
  if (!token) return; // not logged in, skip
  try {
    const apiUrl = await getApiUrl();
    await fetch(`${apiUrl}/api/extension/ping`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    });
  } catch {
    // silently ignore — server may be temporarily unreachable
  }
}

// Ping on install / activate
chrome.runtime.onInstalled.addListener(() => pingExtension());
chrome.runtime.onStartup.addListener(() => pingExtension());

// Recurring ping every 60 seconds via alarms
chrome.alarms.create('extension-ping', { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === 'extension-ping') pingExtension();
});

// ── Message Router ───────────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'LOGIN') {
    handleLogin(message.payload).then(sendResponse);
    return true; // async
  }

  if (message.type === 'LOGOUT') {
    chrome.storage.local.remove(['token', 'userEmail'], () => sendResponse({ ok: true }));
    return true;
  }

  if (message.type === 'GET_AUTH_STATUS') {
    getAuthStatus().then(sendResponse);
    return true;
  }

  if (message.type === 'GENERATE_REPLY') {
    handleGenerateReply(message.payload, sender.tab.id);
    sendResponse({ ok: true, streaming: true });
    return false;
  }
});

// ── Login Handler ────────────────────────────────────────────────────────────

async function handleLogin({ email, password }) {
  try {
    const apiUrl = await getApiUrl();
    const res = await fetch(`${apiUrl}/api/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      return { ok: false, error: err.message || `Login failed (${res.status})` };
    }

    const data = await res.json();
    await chrome.storage.local.set({
      token: data.token,
      userEmail: email,
    });
    return { ok: true };
  } catch (e) {
    return { ok: false, error: `Connection failed: ${e.message}` };
  }
}

// ── Auth Status ──────────────────────────────────────────────────────────────

async function getAuthStatus() {
  const { token, userEmail } = await chrome.storage.local.get(['token', 'userEmail']);
  if (!token) return { loggedIn: false };

  try {
    const apiUrl = await getApiUrl();
    const res = await fetch(`${apiUrl}/api/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    if (res.ok) {
      return { loggedIn: true, email: userEmail };
    }
    // Token expired
    await chrome.storage.local.remove(['token', 'userEmail']);
    return { loggedIn: false };
  } catch {
    return { loggedIn: false, error: 'Cannot reach server' };
  }
}

// ── Generate Reply (SSE Stream) ──────────────────────────────────────────────

async function handleGenerateReply({ prompt }, tabId) {
  const token = await getToken();
  if (!token) {
    chrome.tabs.sendMessage(tabId, {
      type: 'GENERATE_ERROR',
      error: 'Not logged in. Click the MailMate icon to log in.',
    });
    return;
  }

  try {
    const apiUrl = await getApiUrl();
    const res = await fetch(`${apiUrl}/api/emails/generate/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ prompt }),
    });

    if (!res.ok) {
      const err = await res.text();
      chrome.tabs.sendMessage(tabId, {
        type: 'GENERATE_ERROR',
        error: `Server error (${res.status}): ${err}`,
      });
      return;
    }

    // Parse the SSE stream
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    let currentEventType = 'token'; // default event type

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // Process complete SSE lines
      const lines = buffer.split('\n');
      buffer = lines.pop(); // Keep incomplete line in buffer

      for (const line of lines) {
        const trimmed = line.trim();

        // Empty line = end of SSE event block, reset event type
        if (!trimmed) {
          currentEventType = 'token';
          continue;
        }

        // Track the event type for the next data: line
        if (trimmed.startsWith('event: ')) {
          currentEventType = trimmed.slice(7).trim();
          continue;
        }

        if (trimmed.startsWith('data: ')) {
          const data = trimmed.slice(6);

          if (currentEventType === 'done' || data === '[DONE]') {
            chrome.tabs.sendMessage(tabId, { type: 'GENERATE_DONE' });
          } else if (currentEventType === 'error') {
            chrome.tabs.sendMessage(tabId, {
              type: 'GENERATE_ERROR',
              error: data,
            });
          } else if (currentEventType === 'log') {
            chrome.tabs.sendMessage(tabId, {
              type: 'GENERATE_LOG',
              message: data,
            });
          } else {
            // Default: treat as token
            chrome.tabs.sendMessage(tabId, {
              type: 'GENERATE_TOKEN',
              token: data,
            });
          }
        }
      }
    }

    // Signal completion (in case no explicit done event was sent)
    chrome.tabs.sendMessage(tabId, { type: 'GENERATE_DONE' });
  } catch (e) {
    chrome.tabs.sendMessage(tabId, {
      type: 'GENERATE_ERROR',
      error: `Connection failed: ${e.message}`,
    });
  }
}
