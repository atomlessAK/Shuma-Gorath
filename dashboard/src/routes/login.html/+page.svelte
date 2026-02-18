<script>
  import { onMount } from 'svelte';

  let apiKey = '';
  let submitting = false;
  let messageText = '';
  let messageKind = 'info';
  let nextPath = '/dashboard/index.html';

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
    messageText = text;
    messageKind = kind;
  }

  async function submitLogin(event) {
    event.preventDefault();
    const normalized = String(apiKey || '').trim();
    if (!normalized) {
      setMessage('Enter your key.', 'error');
      return;
    }

    submitting = true;
    setMessage('', 'info');

    try {
      const resp = await fetch('/admin/login', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ api_key: normalized })
      });
      if (!resp.ok) {
        throw new Error('Login failed. Check your key.');
      }
      apiKey = '';
      window.location.replace(nextPath);
    } catch (error) {
      setMessage(error.message || 'Login failed.', 'error');
      submitting = false;
    }
  }

  onMount(async () => {
    const params = new URLSearchParams(window.location.search || '');
    nextPath = safeNextPath(params.get('next') || '');
    const session = await getSessionState();
    if (session && session.authenticated === true && session.method === 'session') {
      window.location.replace(nextPath);
    }
  });
</script>

<svelte:head>
  <title>Shuma-Gorath Dashboard Login</title>
</svelte:head>

<main class="login-shell">
  <section class="login-card panel panel-border pad-lg" aria-labelledby="login-title">
    <h1 id="login-title" class="hidden">Dashboard Login</h1>
    <form id="login-form" class="login-form" novalidate on:submit={submitLogin}>
      <label class="control-label" for="login-apikey">Enter your key</label>
      <input
        id="login-apikey"
        class="input-field input-field--mono"
        type="password"
        autocomplete="off"
        spellcheck="false"
        required
        aria-label="API key"
        bind:value={apiKey}
      >
      <button id="login-submit" class="btn btn-submit" type="submit" disabled={submitting}>
        {submitting ? 'Logging in...' : 'Login'}
      </button>
    </form>
    <p id="login-msg" class={`message ${messageKind}`}>{messageText}</p>
  </section>
</main>
