const refreshButton = document.getElementById('refresh');
const details = document.getElementById('details');
const usagePanel = document.getElementById('usage');
const providerTitle = document.getElementById('provider-title');
const grip = document.getElementById('grip');
const providers = { claude: 'Claude Code', codex: 'OpenAI Codex' };
const desktopApps = { claude: 'Claude Desktop', codex: 'ChatGPT Desktop' };
const invoke = window.__TAURI__.core.invoke;
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
let openingApp = false;
const resetTime = new Intl.DateTimeFormat(undefined, {
  month: 'short',
  day: 'numeric',
  hour: 'numeric',
  minute: '2-digit',
});

const messages = {
  'sign-in': 'Sign in again through the provider’s CLI.',
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
  if (usage.status !== 'ok') {
    usagePanel.textContent = messages[usage.status] ?? messages.unavailable;
    return;
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
      row.append(element('small', `Resets ${resetTime.format(limit.resetMs)}`));
    }
    usagePanel.append(row);
  }
}

function updateRings() {
  for (const id of Object.keys(providers)) {
    const button = document.getElementById(id);
    const usage = usageByProvider[id];
    const limit = usage?.limits?.[0];
    const percent = limit ? `${Math.round(limit.used)}%` : '—';
    button.querySelector('.percent').textContent = percent;
    button.querySelector('.ring').style.setProperty('--used', limit ? `${limit.used}%` : '0%');
    const action = `Open ${desktopApps[id]}`;
    button.setAttribute('aria-label', limit ? `${action}, ${percent} of usage limit used` : action);
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

async function refresh() {
  if (refreshButton.disabled) return;
  clearTimeout(refreshTimer);
  refreshButton.disabled = true;
  refreshButton.textContent = 'Checking…';
  try {
    usageByProvider = await invoke('refresh');
  } catch {
    usageByProvider = {
      claude: { status: 'unavailable' },
      codex: { status: 'unavailable' },
    };
  } finally {
    updateRings();
    renderDetails();
    refreshButton.disabled = false;
    refreshButton.textContent = 'Refresh';
    refreshTimer = setTimeout(refresh, 5 * 60 * 1000);
  }
}

refreshButton.addEventListener('click', refresh);
document.getElementById('quit').addEventListener('click', () => invoke('quit'));
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
  if (dragPointer === null) closeTimer = setTimeout(() => showView('grip'), 275);
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
