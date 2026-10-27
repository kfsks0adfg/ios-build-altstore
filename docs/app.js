(() => {
  const fallback = new URL('altstore.json', window.location.href).href;
  const sourceURL = window.IOS_PACK_SOURCE_URL || fallback;
  const sourceText = document.querySelector('#source-url');
  const openButton = document.querySelector('#open-altstore');
  const copyButton = document.querySelector('#copy-source');
  const copyStatus = document.querySelector('#copy-status');
  const appsElement = document.querySelector('#apps');

  sourceText.textContent = sourceURL;
  openButton.href = `altstore://source?url=${encodeURIComponent(sourceURL)}`;

  copyButton.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(sourceURL);
      copyStatus.textContent = 'URL copiée.';
    } catch {
      copyStatus.textContent = 'Appui long sur l’URL pour la copier.';
    }
  });

  fetch(sourceURL, { cache: 'no-store' })
    .then((response) => {
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return response.json();
    })
    .then((source) => {
      if (!Array.isArray(source.apps) || source.apps.length === 0) {
        appsElement.innerHTML = '<p class="muted">Aucune application publiée pour le moment.</p>';
        return;
      }
      appsElement.replaceChildren(...source.apps.map((app) => {
        const version = app.versions?.[0];
        const card = document.createElement('article');
        card.className = 'app-card';

        const icon = document.createElement('img');
        icon.src = app.iconURL;
        icon.alt = '';
        icon.loading = 'lazy';

        const title = document.createElement('h3');
        title.textContent = app.name;
        const description = document.createElement('p');
        description.textContent = app.subtitle || app.localizedDescription;
        const release = document.createElement('p');
        release.className = 'version';
        release.textContent = version
          ? `Version ${version.version} · build ${version.buildVersion}`
          : 'Version indisponible';
        card.append(icon, title, description, release);
        return card;
      }));
    })
    .catch((error) => {
      appsElement.innerHTML = '<p class="muted">La source ne peut pas être chargée pour le moment.</p>';
      console.error('Source AltStore:', error);
    });
})();

