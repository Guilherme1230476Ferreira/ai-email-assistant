// =============================================================================
// MailMate — Popup Script
// Handles login form, auth status display, and logout
// =============================================================================

const loginView = document.getElementById('login-view');
const loggedInView = document.getElementById('logged-in-view');
const loginForm = document.getElementById('login-form');
const loginBtn = document.getElementById('login-btn');
const loginError = document.getElementById('login-error');
const logoutBtn = document.getElementById('logout-btn');
const userEmailSpan = document.getElementById('user-email');

// ── Init ─────────────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
  chrome.runtime.sendMessage({ type: 'GET_AUTH_STATUS' }, (response) => {
    if (response?.loggedIn) {
      showLoggedIn(response.email);
    } else {
      showLoginForm();
    }
  });
});

// ── Login ────────────────────────────────────────────────────────────────────

loginForm.addEventListener('submit', (e) => {
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
