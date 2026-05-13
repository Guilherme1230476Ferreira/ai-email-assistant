// =============================================================================
// MailMate — Popup Script
// Handles login form, auth status display, and server URL configuration
// =============================================================================

const loginView = document.getElementById('login-view');
const loggedInView = document.getElementById('logged-in-view');
const loginForm = document.getElementById('login-form');
const loginBtn = document.getElementById('login-btn');
const loginError = document.getElementById('login-error');
const logoutBtn = document.getElementById('logout-btn');
const userEmailSpan = document.getElementById('user-email');
const apiUrlInput = document.getElementById('api-url');
const saveUrlBtn = document.getElementById('save-url-btn');

// ── Init ─────────────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', async () => {
  // Load saved API URL
  const { apiUrl } = await chrome.storage.local.get('apiUrl');
  if (apiUrl) apiUrlInput.value = apiUrl;

  // Check auth status
  chrome.runtime.sendMessage({ type: 'GET_AUTH_STATUS' }, (response) => {
    if (response?.loggedIn) {
      showLoggedIn(response.email);
    } else {
      showLoginForm();
    }
  });
});

// ── Login ────────────────────────────────────────────────────────────────────

loginForm.addEventListener('submit', async (e) => {
  e.preventDefault();

  const email = document.getElementById('email').value.trim();
  const password = document.getElementById('password').value;

  if (!email || !password) return;

  loginBtn.disabled = true;
  loginBtn.textContent = 'Signing in...';
  loginError.classList.remove('show');

  chrome.runtime.sendMessage(
    { type: 'LOGIN', payload: { email, password } },
    (response) => {
      if (response?.ok) {
        showLoggedIn(email);
      } else {
        loginError.textContent = response?.error || 'Login failed';
        loginError.classList.add('show');
        loginBtn.disabled = false;
        loginBtn.textContent = 'Sign In';
      }
    }
  );
});

// ── Logout ───────────────────────────────────────────────────────────────────

logoutBtn.addEventListener('click', () => {
  chrome.runtime.sendMessage({ type: 'LOGOUT' }, () => {
    showLoginForm();
  });
});

// ── Save URL ─────────────────────────────────────────────────────────────────

saveUrlBtn.addEventListener('click', () => {
  const url = apiUrlInput.value.trim().replace(/\/+$/, ''); // Remove trailing slashes
  if (url) {
    chrome.storage.local.set({ apiUrl: url }, () => {
      saveUrlBtn.textContent = '✓ Saved';
      setTimeout(() => { saveUrlBtn.textContent = 'Save URL'; }, 1500);
    });
  }
});

// ── View Switching ───────────────────────────────────────────────────────────

function showLoggedIn(email) {
  loginView.classList.add('hidden');
  loggedInView.classList.remove('hidden');
  userEmailSpan.textContent = email || '';
}

function showLoginForm() {
  loggedInView.classList.add('hidden');
  loginView.classList.remove('hidden');
  loginBtn.disabled = false;
  loginBtn.textContent = 'Sign In';
}
