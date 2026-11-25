const refreshButton = document.getElementById('refresh');
const details = document.getElementById('details');
const usagePanel = document.getElementById('usage');
const providerTitle = document.getElementById('provider-title');
const grip = document.getElementById('grip');
const providers = { claude: 'Claude Code', codex: 'OpenAI Codex' };
const desktopApps = { claude: 'Claude Desktop', codex: 'ChatGPT Desktop' };
const invoke = window.__TAURI__.core.invoke;
const theme = document.getElementById('theme');
try {
  const savedTheme = localStorage.getItem('quota-sentinel-theme');
  if (Array.from(theme.options).some(option => option.value === savedTheme)) theme.value = savedTheme;
} catch {
  // The current theme still works if WebView storage is unavailable.
}
document.documentElement.dataset.theme = theme.value;
theme.addEventListener('change', () => {
  document.documentElement.dataset.theme = theme.value;
  try { localStorage.setItem('quota-sentinel-theme', theme.value); } catch {}
});
let usageByProvider = {};
let selected;
let closeTimer;
let layout = Promise.resolve();
let requestedView = 'grip';
let dragPointer = null;
let dragEnding = false;
let dragReady = Promise.resolve();
let dragMovePending = false;
let refreshTimer;
let checkingUsage = false;
let refreshRequested = false;
let lastReading;
let openingApp = false;
const resetTime = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: 'numeric',
  hour: 'numeric',
  minute: '2-digit',
});

const messages = {
  'sign-in': 'Sign in again through the provider’s CLI.',
  'claude-session-missing': 'No readable Claude Code session. Open Claude Code to check.',
  'claude-session-expired': 'Waiting for Claude Code to renew its session.',
  'rejected': 'Claude’s usage request was rejected. Check /usage in Claude Code.',
  'unsupported': 'No supported usage windows were returned.',
  'rate-limited': 'The provider asked us to wait. Try again in a few minutes.',
  'redirect': 'The provider redirected the request. It was blocked.',
  'unavailable': 'Could not retrieve usage.',
};

function element(tag, text, className) {
  const node = document.createElement(tag);
  node.textContent = text;
  if (className) node.className = className;
  return node;
}

function renderDetails() {
  if (!selected) return;
  const usage = usageByProvider[selected];
  providerTitle.textContent = providers[selected];
  details.dataset.provider = selected;
  usagePanel.replaceChildren();
  if (!usage) {
    usagePanel.textContent = 'Checking usage…';
    return;
  }
  const stale = usage.status !== 'ok';
  if (stale) {
    const status = element('div', messages[usage.status] ?? messages.unavailable, 'usage-status');
    usagePanel.append(status);
    if (selected === 'claude' && ['claude-session-missing', 'claude-session-expired'].includes(usage.status)) {
      const action = element('div', '', 'session-action');
      const terminal = element('button', 'Open Claude Code');
      terminal.type = 'button';
      terminal.addEventListener('click', async () => {
        terminal.disabled = true;
        try {
          if (!await invoke('open_claude_code')) {
            status.textContent = 'Claude Code was not found automatically. In Terminal, try claude.';
          }
        } catch (error) {
          status.textContent = typeof error === 'string' ? error : 'Could not open Windows Terminal.';
        } finally {
          terminal.disabled = false;
        }
      });
      action.append(terminal);
      usagePanel.append(action);
    }
    if (!usage.lastUpdatedMs) return;
    usagePanel.append(element('small', `Last updated ${resetTime.format(usage.lastUpdatedMs)}`, 'usage-stamp'));
  }

  for (const limit of usage.limits) {
    const row = element('div', '', 'limit');
    const label = element('div', '', 'limit-label');
    label.append(element('span', limit.label), element('strong', `${Math.round(limit.used)}% used`));
    const progress = document.createElement('progress');
    progress.max = 100;
    progress.value = limit.used;
    progress.setAttribute('aria-label', `${limit.label} usage`);
    row.append(label, progress);
    if (limit.resetMs && Number.isFinite(new Date(limit.resetMs).getTime())) {
      const text = stale && limit.resetMs <= Date.now()
        ? 'Awaiting refresh'
        : `Resets ${resetTime.format(limit.resetMs)}`;
      row.append(element('small', text));
    }
    usagePanel.append(row);
  }
}

function updateRings() {
  for (const id of Object.keys(providers)) {
    const button = document.getElementById(id);
    const usage = usageByProvider[id];
    const limit = usage?.limits?.[0];
    const stale = Boolean(limit && usage.status !== 'ok');
    button.toggleAttribute('data-stale', stale);
    const percent = limit ? `${Math.round(limit.used)}%` : '—';
    button.querySelector('.percent').textContent = percent;
    const ring = button.querySelector('.ring');
    ring.style.setProperty('--used', limit?.used ?? 0);
    ring.toggleAttribute('data-empty', !limit?.used);
    const action = `Open ${desktopApps[id]}`;
    button.setAttribute('aria-label', limit ? `${action}, ${percent} used${stale ? ' at last update' : ''}` : action);
  }
}

async function launchDesktop(provider) {
  if (openingApp || dragPointer !== null) return;
  openingApp = true;
  try {
    await invoke('launch_desktop', { provider });
  } catch (error) {
    showView('details', provider);
    usagePanel.textContent = typeof error === 'string' ? error : 'Could not open the desktop app.';
  } finally {
    openingApp = false;
  }
}

function showView(view, provider) {
  if (view !== 'grip' && dragPointer !== null) return;
  clearTimeout(closeTimer);
  selected = provider;
  for (const id of Object.keys(providers)) {
    document.getElementById(id).setAttribute('aria-expanded', String(id === selected));
  }
  if (selected) renderDetails();
  else details.hidden = true;
  if (requestedView === view) return;
  requestedView = view;
  layout = layout.catch(() => {}).then(async () => {
    if (view !== requestedView) return;
    const side = await invoke('set_dock_view', { view });
    document.body.dataset.side = side;
    document.body.dataset.view = view;
    grip.setAttribute('aria-expanded', String(view !== 'grip'));
    if (view === 'details' && selected) details.hidden = false;
  });
  layout.catch(() => {});
}

async function refresh(force = false) {
  if (checkingUsage) {
    refreshRequested ||= force;
    return;
  }
  clearTimeout(refreshTimer);
  checkingUsage = true;
  if (force) {
    refreshButton.disabled = true;
    refreshButton.textContent = 'Checking…';
  }
  try {
    usageByProvider = await invoke('refresh', { force });
  } catch {
    usageByProvider = {
      claude: { ...usageByProvider.claude, status: 'unavailable' },
      codex: { status: 'unavailable' },
    };
  } finally {
    const reading = JSON.stringify([usageByProvider, usageByProvider.claude?.limits?.map(
      limit => usageByProvider.claude.status !== 'ok' && limit.resetMs <= Date.now()
    )]);
    if (reading !== lastReading) {
      lastReading = reading;
      updateRings();
      renderDetails();
    }
    checkingUsage = false;
    refreshButton.disabled = false;
    refreshButton.textContent = 'Refresh';
    if (refreshRequested) {
      refreshRequested = false;
      void refresh(true);
    } else {
      refreshTimer = setTimeout(refresh, 5000);
    }
  }
}

refreshButton.addEventListener('click', () => refresh(true));
document.getElementById('quit').addEventListener('click', () => invoke('quit'));
document.addEventListener('contextmenu', event => {
  event.preventDefault();
  void invoke('show_quit_menu').catch(() => {});
});
grip.addEventListener('pointerenter', () => {
  if (requestedView === 'grip') showView('rings');
});
grip.addEventListener('focus', () => {
  if (requestedView === 'grip') showView('rings');
});
grip.addEventListener('click', () => {
  if (requestedView === 'grip') showView('rings');
});
grip.addEventListener('pointerdown', event => {
  if (event.button !== 0 || dragPointer !== null) return;
  event.preventDefault();
  dragPointer = event.pointerId;
  grip.setPointerCapture(dragPointer);
  showView('grip');
  dragReady = layout.catch(() => {}).then(() => invoke('drag_dock', { phase: 'start' }));
  dragReady.catch(() => {});
});
grip.addEventListener('pointermove', event => {
  if (event.pointerId !== dragPointer || dragMovePending || dragEnding) return;
  dragMovePending = true;
  dragReady.then(() => invoke('drag_dock', { phase: 'move' }))
    .catch(() => {}).finally(() => { dragMovePending = false; });
});
function endDrag(event) {
  if (event.pointerId !== dragPointer || dragEnding) return;
  dragEnding = true;
  dragReady.then(async () => {
    document.body.dataset.side = await invoke('drag_dock', { phase: 'drop' });
  }).catch(() => {}).finally(async () => {
    await invoke('drag_dock', { phase: 'end' }).catch(() => {});
    dragPointer = null;
    dragEnding = false;
  });
}
for (const event of ['pointerup', 'pointercancel', 'lostpointercapture']) {
  grip.addEventListener(event, endDrag);
}
for (const provider of Object.keys(providers)) {
  const button = document.getElementById(provider);
  button.addEventListener('pointerenter', () => showView('details', provider));
  button.addEventListener('focus', () => showView('details', provider));
  button.addEventListener('click', () => launchDesktop(provider));
}
document.documentElement.addEventListener('pointerenter', () => clearTimeout(closeTimer));
document.documentElement.addEventListener('pointerleave', () => {
  if (document.activeElement === theme) return;
  if (dragPointer === null) closeTimer = setTimeout(() => showView('grip'), 275);
});
theme.addEventListener('blur', () => {
  if (!document.documentElement.matches(':hover') && dragPointer === null) {
    closeTimer = setTimeout(() => showView('grip'), 275);
  }
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape') {
    if (selected) {
      document.getElementById(selected).focus();
      showView('rings');
    } else {
      grip.focus();
      showView('grip');
    }
  }
});
refresh();
