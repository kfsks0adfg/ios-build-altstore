(() => {
  'use strict';

  const config = window.IOS_STUDIO_CONFIG || {};
  const state = {
    repository: localStorage.getItem('forge-ios-repository') || config.repository || '',
    branch: config.branch || 'main',
    workflow: config.workflow || 'ios-release.yml',
    token: sessionStorage.getItem('forge-ios-token') || '',
    user: null,
    file: null,
    installPrompt: null,
    toastTimer: null,
  };

  const $ = (selector) => document.querySelector(selector);
  const $$ = (selector) => [...document.querySelectorAll(selector)];
  const elements = {
    repository: $('#repository'),
    token: $('#github-token'),
    connect: $('#connect-github'),
    disconnect: $('#disconnect-github'),
    connectionButton: $('#connection-button'),
    connectionDot: $('#connection-dot'),
    connectionLabel: $('#connection-label'),
    connectionStatus: $('#connection-status'),
    visibility: $('#repo-visibility'),
    branch: $('#branch'),
    version: $('#version'),
    buildStatus: $('#build-status'),
    runs: $('#workflow-runs'),
    releases: $('#releases'),
    filePath: $('#file-path'),
    editor: $('#code-editor'),
    editorFilename: $('#editor-filename'),
    editorState: $('#editor-state'),
    editorStatus: $('#editor-status'),
    saveFile: $('#save-file'),
    sourceURL: $('#source-url'),
    apps: $('#apps'),
    appsCount: $('#apps-count'),
    copyStatus: $('#copy-status'),
    toast: $('#toast'),
  };

  elements.repository.value = state.repository;
  elements.branch.value = state.branch;
  elements.token.value = state.token;

  function setStatus(element, message, isError = false) {
    element.textContent = message;
    element.classList.toggle('error', isError);
  }

  function showToast(message) {
    clearTimeout(state.toastTimer);
    elements.toast.textContent = message;
    elements.toast.hidden = false;
    state.toastTimer = setTimeout(() => { elements.toast.hidden = true; }, 4200);
  }

  function requireToken() {
    if (state.token) return true;
    setStatus(elements.connectionStatus, 'Ajoutez un jeton GitHub pour effectuer cette action.', true);
    $('#connection-panel').scrollIntoView({ behavior: 'smooth', block: 'center' });
    return false;
  }

  function repositoryParts() {
    const value = elements.repository.value.trim().replace(/^https:\/\/github\.com\//i, '').replace(/\/$/, '');
    if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value)) {
      throw new Error('Le dépôt doit être écrit sous la forme utilisateur/depot.');
    }
    return value.split('/');
  }

  function repositoryName() {
    return repositoryParts().join('/');
  }

  function encodedPath(path) {
    return path.split('/').map(encodeURIComponent).join('/');
  }

  async function github(path, options = {}) {
    const headers = {
      Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28',
      ...options.headers,
    };
    if (state.token) headers.Authorization = `Bearer ${state.token}`;
    if (options.body && !headers['Content-Type']) headers['Content-Type'] = 'application/json';

    const response = await fetch(`https://api.github.com${path}`, {
      ...options,
      headers,
      cache: 'no-store',
    });
    if (response.status === 204) return null;

    let data = null;
    try { data = await response.json(); } catch { /* Réponse sans JSON. */ }
    if (!response.ok) {
      const detail = data?.message ? ` — ${data.message}` : '';
      throw new Error(`GitHub répond ${response.status}${detail}`);
    }
    return data;
  }

  function apiPath(suffix = '') {
    return `/repos/${repositoryName()}${suffix}`;
  }

  function setConnected(repo, user = null) {
    state.repository = repo.full_name;
    state.user = user;
    localStorage.setItem('forge-ios-repository', repo.full_name);
    elements.repository.value = repo.full_name;
    elements.visibility.textContent = repo.private ? 'privé' : 'public';
    elements.connectionDot.classList.toggle('connected', Boolean(state.token));
    elements.connectionLabel.textContent = state.token
      ? (user?.login || 'Connecté')
      : 'Mode public';
    elements.disconnect.hidden = !state.token;
    elements.editor.readOnly = !state.token || !state.file;
    elements.saveFile.disabled = !state.token || !state.file;
  }

  async function connect() {
    const submittedToken = elements.token.value.trim();
    const previousToken = state.token;
    state.token = submittedToken;
    setStatus(elements.connectionStatus, 'Connexion en cours…');
    elements.connect.disabled = true;

    try {
      const userPromise = state.token ? github('/user') : Promise.resolve(null);
      const repoPromise = github(apiPath());
      const [user, repo] = await Promise.all([userPromise, repoPromise]);
      if (state.token) sessionStorage.setItem('forge-ios-token', state.token);
      else sessionStorage.removeItem('forge-ios-token');
      setConnected(repo, user);
      setStatus(elements.connectionStatus, state.token
        ? `Connecté en tant que ${user.login}. Les actions d’écriture sont disponibles.`
        : 'Dépôt public chargé en lecture seule.');
      await Promise.all([loadActivity(), openFile(elements.filePath.value, false)]);
    } catch (error) {
      state.token = previousToken;
      elements.token.value = previousToken;
      setStatus(elements.connectionStatus, error.message, true);
    } finally {
      elements.connect.disabled = false;
    }
  }

  function disconnect() {
    state.token = '';
    state.user = null;
    sessionStorage.removeItem('forge-ios-token');
    elements.token.value = '';
    elements.connectionDot.classList.remove('connected');
    elements.connectionLabel.textContent = 'Mode public';
    elements.disconnect.hidden = true;
    elements.editor.readOnly = true;
    elements.saveFile.disabled = true;
    elements.editorState.textContent = 'lecture seule';
    setStatus(elements.connectionStatus, 'Jeton supprimé de cette session. Le dépôt reste lisible publiquement.');
  }

  function formatDate(value) {
    return new Intl.DateTimeFormat('fr-FR', {
      day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit',
    }).format(new Date(value));
  }

  function createListItem({ href, icon, iconClass = '', title, subtitle, meta }) {
    const item = document.createElement('a');
    item.className = 'list-item';
    item.href = href;
    item.target = '_blank';
    item.rel = 'noopener noreferrer';

    const symbol = document.createElement('span');
    symbol.className = `list-icon ${iconClass}`.trim();
    symbol.textContent = icon;
    const copy = document.createElement('span');
    copy.className = 'list-copy';
    const strong = document.createElement('strong');
    strong.textContent = title;
    const small = document.createElement('small');
    small.textContent = subtitle;
    copy.append(strong, small);
    const metadata = document.createElement('span');
    metadata.className = 'list-meta';
    metadata.textContent = meta;
    item.append(symbol, copy, metadata);
    return item;
  }

  function runAppearance(run) {
    if (run.status !== 'completed') return { icon: '…', className: 'pending', label: 'en cours' };
    if (run.conclusion === 'success') return { icon: '✓', className: '', label: 'réussi' };
    return { icon: '!', className: 'failure', label: run.conclusion || 'échec' };
  }

  async function loadActivity() {
    elements.runs.innerHTML = '<p class="empty">Chargement…</p>';
    elements.releases.innerHTML = '<p class="empty">Chargement…</p>';
    try {
      const [runsData, releases] = await Promise.all([
        github(apiPath('/actions/runs?per_page=6')),
        github(apiPath('/releases?per_page=6')),
      ]);

      const runs = runsData.workflow_runs || [];
      elements.runs.replaceChildren(...(runs.length ? runs.map((run) => {
        const appearance = runAppearance(run);
        return createListItem({
          href: run.html_url,
          icon: appearance.icon,
          iconClass: appearance.className,
          title: run.display_title || run.name,
          subtitle: `${run.name} · ${run.event}`,
          meta: appearance.label,
        });
      }) : [emptyMessage('Aucune exécution pour le moment.')]));

      elements.releases.replaceChildren(...(releases.length ? releases.map((release) => {
        const ipa = release.assets?.find((asset) => asset.name.toLowerCase().endsWith('.ipa'));
        return createListItem({
          href: ipa?.browser_download_url || release.html_url,
          icon: ipa ? '↓' : '↗',
          title: release.name || release.tag_name,
          subtitle: ipa ? ipa.name : 'Release sans IPA',
          meta: formatDate(release.published_at || release.created_at),
        });
      }) : [emptyMessage('Aucune IPA publiée pour le moment.')]));
    } catch (error) {
      elements.runs.replaceChildren(emptyMessage(error.message));
      elements.releases.replaceChildren(emptyMessage('Impossible de lire les Releases.'));
    }
  }

  function emptyMessage(message) {
    const element = document.createElement('p');
    element.className = 'empty';
    element.textContent = message;
    return element;
  }

  function selectedProject() {
    return $('input[name="project"]:checked').value;
  }

  async function testBuild() {
    if (!requireToken()) return;
    const button = $('#test-build');
    button.disabled = true;
    setStatus(elements.buildStatus, 'Lancement du build de test…');
    try {
      await github(apiPath(`/actions/workflows/${encodeURIComponent(state.workflow)}/dispatches`), {
        method: 'POST',
        body: JSON.stringify({
          ref: elements.branch.value.trim() || state.branch,
          inputs: { example: selectedProject() },
        }),
      });
      setStatus(elements.buildStatus, 'Build demandé. Il apparaîtra dans la liste dans quelques secondes.');
      showToast('Build de test lancé sur GitHub Actions.');
      setTimeout(loadActivity, 3500);
    } catch (error) {
      setStatus(elements.buildStatus, error.message, true);
    } finally {
      button.disabled = false;
    }
  }

  async function releaseBuild() {
    if (!requireToken()) return;
    const version = elements.version.value.trim();
    if (!/^\d+\.\d+\.\d+(?:[-.][0-9A-Za-z.-]+)?$/.test(version)) {
      setStatus(elements.buildStatus, 'Utilisez une version comme 1.0.0 ou 1.0.0-beta.1.', true);
      elements.version.focus();
      return;
    }

    const branch = elements.branch.value.trim() || state.branch;
    const prefix = selectedProject() === 'react-native' ? 'rn-v' : 'swift-v';
    const tag = `${prefix}${version}`;
    const button = $('#release-build');
    button.disabled = true;
    setStatus(elements.buildStatus, `Création du tag ${tag}…`);
    try {
      const ref = await github(apiPath(`/git/ref/heads/${encodedPath(branch)}`));
      await github(apiPath('/git/refs'), {
        method: 'POST',
        body: JSON.stringify({ ref: `refs/tags/${tag}`, sha: ref.object.sha }),
      });
      setStatus(elements.buildStatus, `${tag} créé. GitHub compile maintenant l’IPA et publiera la Release.`);
      showToast(`Publication ${tag} lancée.`);
      setTimeout(loadActivity, 3500);
    } catch (error) {
      setStatus(elements.buildStatus, error.message, true);
    } finally {
      button.disabled = false;
    }
  }

  function decodeBase64(value) {
    const bytes = Uint8Array.from(atob(value.replace(/\n/g, '')), (character) => character.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  }

  function encodeBase64(value) {
    const bytes = new TextEncoder().encode(value);
    let binary = '';
    const chunkSize = 0x8000;
    for (let index = 0; index < bytes.length; index += chunkSize) {
      binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize));
    }
    return btoa(binary);
  }

  async function openFile(path = elements.filePath.value, announce = true) {
    const cleanPath = path.trim().replace(/^\/+/, '');
    if (!cleanPath || cleanPath.includes('..')) {
      setStatus(elements.editorStatus, 'Chemin de fichier invalide.', true);
      return;
    }
    if (state.file && elements.editor.value !== state.file.original && announce
      && !window.confirm('Abandonner les modifications non enregistrées ?')) return;

    setStatus(elements.editorStatus, 'Ouverture du fichier…');
    elements.editor.readOnly = true;
    elements.saveFile.disabled = true;
    try {
      const branch = elements.branch.value.trim() || state.branch;
      const file = await github(apiPath(`/contents/${encodedPath(cleanPath)}?ref=${encodeURIComponent(branch)}`));
      if (file.type !== 'file' || typeof file.content !== 'string') throw new Error('Ce chemin ne désigne pas un fichier texte.');
      const content = decodeBase64(file.content);
      state.file = { path: cleanPath, sha: file.sha, original: content, branch };
      elements.filePath.value = cleanPath;
      elements.editor.value = content;
      elements.editorFilename.textContent = cleanPath.split('/').pop();
      elements.editor.readOnly = !state.token;
      elements.saveFile.disabled = !state.token;
      elements.editorState.textContent = state.token ? 'modifiable' : 'lecture seule';
      $$('.file-shortcut').forEach((button) => button.classList.toggle('active', button.dataset.path === cleanPath));
      setStatus(elements.editorStatus, state.token
        ? 'Fichier chargé. Les modifications seront commitées sur GitHub.'
        : 'Fichier chargé en lecture seule. Connectez un jeton pour le modifier.');
    } catch (error) {
      state.file = null;
      elements.editor.value = '';
      elements.editorFilename.textContent = cleanPath.split('/').pop() || 'Fichier';
      setStatus(elements.editorStatus, error.message, true);
    }
  }

  async function saveFile() {
    if (!requireToken() || !state.file) return;
    if (elements.editor.value === state.file.original) {
      setStatus(elements.editorStatus, 'Aucune modification à enregistrer.');
      return;
    }

    elements.saveFile.disabled = true;
    setStatus(elements.editorStatus, 'Enregistrement sur GitHub…');
    try {
      const filename = state.file.path.split('/').pop();
      const result = await github(apiPath(`/contents/${encodedPath(state.file.path)}`), {
        method: 'PUT',
        body: JSON.stringify({
          message: `Modifier ${filename} depuis Forge iOS`,
          content: encodeBase64(elements.editor.value),
          sha: state.file.sha,
          branch: state.file.branch,
        }),
      });
      state.file.sha = result.content.sha;
      state.file.original = elements.editor.value;
      setStatus(elements.editorStatus, `Enregistré dans ${state.file.branch}.`);
      showToast(`${filename} enregistré sur GitHub.`);
    } catch (error) {
      setStatus(elements.editorStatus, error.message, true);
    } finally {
      elements.saveFile.disabled = false;
    }
  }

  async function loadAltStoreSource() {
    const fallback = new URL('altstore.json', window.location.href).href;
    const sourceURL = window.IOS_PACK_SOURCE_URL || fallback;
    elements.sourceURL.textContent = sourceURL;
    $('#open-altstore').href = `altstore://source?url=${encodeURIComponent(sourceURL)}`;
    try {
      const response = await fetch(sourceURL, { cache: 'no-store' });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const source = await response.json();
      const apps = Array.isArray(source.apps) ? source.apps : [];
      elements.appsCount.textContent = String(apps.length);
      elements.apps.replaceChildren(...(apps.length ? apps.map((app) => {
        const card = document.createElement('article');
        card.className = 'app-item';
        const icon = document.createElement('img');
        icon.src = app.iconURL || 'assets/forge-icon.svg';
        icon.alt = '';
        icon.loading = 'lazy';
        const copy = document.createElement('div');
        const name = document.createElement('strong');
        name.textContent = app.name || 'Application';
        const version = document.createElement('small');
        const latest = app.versions?.[0];
        version.textContent = latest ? `Version ${latest.version} · build ${latest.buildVersion}` : 'Aucune version';
        copy.append(name, version);
        card.append(icon, copy);
        return card;
      }) : [emptyMessage('Aucune application publiée pour le moment.')]));
    } catch (error) {
      elements.apps.replaceChildren(emptyMessage('La source sera disponible après la première publication.'));
    }
  }

  function setupEvents() {
    elements.connect.addEventListener('click', connect);
    elements.disconnect.addEventListener('click', disconnect);
    elements.connectionButton.addEventListener('click', () => $('#connection-panel').scrollIntoView({ behavior: 'smooth', block: 'center' }));
    $('#toggle-token').addEventListener('click', (event) => {
      const visible = elements.token.type === 'text';
      elements.token.type = visible ? 'password' : 'text';
      event.currentTarget.textContent = visible ? 'Voir' : 'Masquer';
    });

    $$('input[name="project"]').forEach((input) => input.addEventListener('change', () => {
      $$('.build-choice').forEach((choice) => choice.classList.toggle('selected', choice.contains(input) && input.checked));
    }));
    $('#test-build').addEventListener('click', testBuild);
    $('#release-build').addEventListener('click', releaseBuild);
    $('#refresh-activity').addEventListener('click', loadActivity);

    $$('.file-shortcut').forEach((button) => button.addEventListener('click', () => openFile(button.dataset.path)));
    $('#open-file').addEventListener('click', () => openFile());
    elements.filePath.addEventListener('keydown', (event) => { if (event.key === 'Enter') openFile(); });
    elements.editor.addEventListener('input', () => {
      if (!state.file) return;
      const dirty = elements.editor.value !== state.file.original;
      elements.editorState.textContent = dirty ? 'modifié' : (state.token ? 'modifiable' : 'lecture seule');
    });
    elements.editor.addEventListener('keydown', (event) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        saveFile();
      }
    });
    elements.saveFile.addEventListener('click', saveFile);

    $('#copy-source').addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(elements.sourceURL.textContent);
        setStatus(elements.copyStatus, 'URL copiée.');
      } catch {
        setStatus(elements.copyStatus, 'Appui long sur l’URL pour la copier.', true);
      }
    });

    window.addEventListener('beforeinstallprompt', (event) => {
      event.preventDefault();
      state.installPrompt = event;
    });
    $('#install-pwa').addEventListener('click', async () => {
      if (state.installPrompt) {
        state.installPrompt.prompt();
        await state.installPrompt.userChoice;
        state.installPrompt = null;
      } else if (window.navigator.standalone || window.matchMedia('(display-mode: standalone)').matches) {
        showToast('Forge iOS est déjà ouverte comme une app.');
      } else {
        showToast('Sur iPhone : touchez Partager, puis « Sur l’écran d’accueil ».');
      }
    });

    window.addEventListener('beforeunload', (event) => {
      if (state.file && elements.editor.value !== state.file.original) {
        event.preventDefault();
      }
    });
  }

  async function start() {
    setupEvents();
    if ('serviceWorker' in navigator) navigator.serviceWorker.register('service-worker.js').catch(() => {});
    await loadAltStoreSource();
    try {
      const [user, repo] = await Promise.all([
        state.token ? github('/user') : Promise.resolve(null),
        github(apiPath()),
      ]);
      setConnected(repo, user);
      setStatus(elements.connectionStatus, state.token
        ? `Session restaurée pour ${user.login}.`
        : 'Dépôt public chargé en lecture seule.');
      await Promise.all([loadActivity(), openFile(elements.filePath.value, false)]);
    } catch (error) {
      setStatus(elements.connectionStatus, `Connexion automatique impossible : ${error.message}`, true);
      await loadActivity();
    }
  }

  start();
})();
