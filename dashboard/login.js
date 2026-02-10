function safeNextPath(raw) {
  const fallback = '/dashboard/index.html';
  if (!raw) return fallback;
  try {
    const decoded = decodeURIComponent(raw);
    const url = new URL(decoded, window.location.origin);
    if (url.origin !== window.location.origin) return fallback;
    if (!url.pathname.startsWith('/dashboard/')) return fallback;
    return `${url.pathname}${url.search}${url.hash}`;
  } catch (_e) {
    return fallback;
  }
}

async function getSessionState() {
  try {
    const resp = await fetch('/admin/session', { credentials: 'same-origin' });
    if (!resp.ok) return null;
    return await resp.json();
  } catch (_e) {
    return null;
  }
}

function setMessage(text, kind) {
  const msg = document.getElementById('login-msg');
  if (!msg) return;
  msg.textContent = text;
  msg.className = `message ${kind}`;
}

async function initLoginPage() {
  const form = document.getElementById('login-form');
  const keyInput = document.getElementById('login-apikey');
  const submitBtn = document.getElementById('login-submit');
  if (!form || !keyInput || !submitBtn) return;

  const params = new URLSearchParams(window.location.search || '');
  const nextPath = safeNextPath(params.get('next') || '');

  const session = await getSessionState();
  if (session && session.authenticated === true && session.method === 'session') {
    window.location.replace(nextPath);
    return;
  }

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const apiKey = (keyInput.value || '').trim();
    if (!apiKey) {
      setMessage('Enter your key.', 'error');
      keyInput.focus();
      return;
    }

    submitBtn.disabled = true;
    submitBtn.textContent = 'Logging in...';
    setMessage('', 'info');

    try {
      const resp = await fetch('/admin/login', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ api_key: apiKey })
      });
      if (!resp.ok) {
        throw new Error('Login failed. Check your key.');
      }
      keyInput.value = '';
      window.location.replace(nextPath);
    } catch (e) {
      setMessage(e.message || 'Login failed.', 'error');
      submitBtn.disabled = false;
      submitBtn.textContent = 'Login';
      keyInput.focus();
    }
  });
}

initLoginPage();
