(() => {
  const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
  if (!motionPreference.matches && 'IntersectionObserver' in window) {
    document.documentElement.classList.add('has-motion');
    const revealObserver = new IntersectionObserver(entries => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add('is-visible');
        revealObserver.unobserve(entry.target);
      }
    }, { threshold: 0.12, rootMargin: '0px 0px -24px 0px' });
    document.querySelectorAll('[data-reveal]').forEach(element => revealObserver.observe(element));
  }

  const repository = window.NAGI_SITE_CONFIG?.githubRepository?.trim() || '';
  const download = document.getElementById('download');
  const lowerDownload = document.getElementById('download-bottom');
  const note = document.getElementById('release-note');
  const version = document.getElementById('version');
  const source = document.getElementById('source-link');
  const downloadVersion = document.getElementById('download-version');
  const downloadFootnote = document.getElementById('download-footnote');
  const downloadLabels = document.querySelectorAll('[data-download-label]');
  const isMac = /Macintosh|Mac OS X/.test(navigator.userAgent) || /Mac/.test(navigator.platform || '');
  const assetName = isMac ? 'Nagi-macOS-universal.dmg' : 'Nagi-Installer.exe';

  if (isMac) {
    for (const link of [download, lowerDownload]) {
      link.removeAttribute('href');
      link.removeAttribute('download');
      link.setAttribute('aria-disabled', 'true');
    }
    version.textContent = 'macOS · Universal';
    note.textContent = 'macOS release coming soon';
    document.title = 'Nagi — Your Mac browser, back to the basics';
    document.querySelector('meta[name="description"]')?.setAttribute('content', 'Meet Nagi, a straightforward macOS browser for everyday browsing.');
    document.querySelector('meta[property="og:title"]')?.setAttribute('content', 'Nagi — Your Mac browser, back to the basics');
    document.getElementById('platform-eyebrow').textContent = 'For macOS 12 and later';
    document.getElementById('hero-title').textContent = 'Your Mac browser,';
    document.getElementById('engine-name').textContent = 'WebKit';
    document.getElementById('engine-description').textContent = 'Uses Apple’s built-in macOS web engine';
    document.getElementById('filter-description').textContent = 'Choose a search engine, turn on dark mode, and browse in 14 interface languages. Domain filtering is available in the Windows release.';
    document.getElementById('privacy-platform-copy').textContent = 'Bookmarks, history, downloads and preferences stay in your macOS application support profile. WebKit handles website authentication using macOS system services.';
    document.getElementById('password-description').textContent = 'Save bookmarks, revisit history and manage downloads. Website password saving and autofill depend on macOS and WebKit settings.';
    document.getElementById('profile-fact').textContent = 'Saved in your Mac user profile';
    document.getElementById('visual-platform').textContent = 'Nagi for macOS';
    document.getElementById('download-platform-label').textContent = 'Ready for your Mac';
    document.getElementById('download-platform-copy').textContent = 'Universal app for Apple silicon and Intel Macs running macOS 12 or later.';
    document.getElementById('download-product-name').textContent = 'Nagi for macOS';
    document.getElementById('download-version').textContent = 'Universal app · DMG';
    for (const label of downloadLabels) label.textContent = 'Download for macOS';
  }

  if (!repository || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
    if (isMac) {
      for (const link of [download, lowerDownload]) {
        link.removeAttribute('href');
        link.setAttribute('aria-disabled', 'true');
      }
      for (const label of downloadLabels) label.textContent = 'macOS release coming soon';
      note.textContent = 'The macOS installer is not available yet';
    }
    return;
  }
  const releases = `https://github.com/${repository}/releases`;
  source.href = releases;

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 10000);
  fetch(`https://api.github.com/repos/${repository}/releases/latest`, {
    headers: { Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28' },
    cache: 'default', signal: controller.signal
  }).then(response => {
    if (!response.ok) throw new Error('Release unavailable');
    return response.json();
  }).then(release => {
    const installer = release.assets?.find(asset => asset.name === assetName);
    if (isMac && !installer) {
      note.textContent = 'macOS release coming soon';
      downloadFootnote.textContent = 'The Mac app will be available after signing and notarization';
      for (const link of [download, lowerDownload]) {
        link.removeAttribute('href');
        link.removeAttribute('download');
        link.setAttribute('aria-disabled', 'true');
      }
      for (const label of downloadLabels) label.textContent = 'macOS release coming soon';
      return;
    }
    const tag = /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(release.tag_name || '');
    let assetUrl;
    try { assetUrl = new URL(installer?.browser_download_url || ''); } catch { assetUrl = null; }
    const expectedPath = `/${repository}/releases/download/${release.tag_name}/${assetName}`;
    if (release.draft === true || release.prerelease === true || !tag || !assetUrl
      || assetUrl.protocol !== 'https:' || assetUrl.hostname !== 'github.com' || assetUrl.port
      || assetUrl.username || assetUrl.password || assetUrl.search || assetUrl.hash
      || assetUrl.pathname !== expectedPath) {
      throw new Error('Installer asset not found');
    }
    for (const link of [download, lowerDownload]) {
      link.href = assetUrl.href;
      link.removeAttribute('download');
      link.removeAttribute('aria-disabled');
    }
    version.textContent = `${release.tag_name} · Windows · 64-bit`;
    if (isMac) {
      version.textContent = `${release.tag_name} · macOS · Universal`;
      downloadVersion.textContent = `${release.tag_name} · Universal DMG`;
      downloadFootnote.textContent = 'For Apple silicon and Intel Macs';
    } else {
      downloadVersion.textContent = `${release.tag_name} · 64-bit installer`;
      downloadFootnote.textContent = 'Choose your language during setup';
    }
    note.textContent = 'Latest stable release';
  }).catch(() => {
    note.textContent = 'GitHub release info is temporarily unavailable';
    downloadFootnote.textContent = isMac ? 'Check back after the next macOS release' : 'The current installer is still available';
    if (isMac) {
      for (const link of [download, lowerDownload]) {
        link.href = releases;
        link.removeAttribute('download');
        link.removeAttribute('aria-disabled');
      }
      for (const label of downloadLabels) label.textContent = 'View GitHub releases';
    }
  }).finally(() => {
    clearTimeout(timeout);
  });
})();
