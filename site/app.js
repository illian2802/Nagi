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

  if (!repository || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) return;
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
    const installer = release.assets?.find(asset => asset.name === 'Nagi-Installer.exe');
    const tag = /^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(release.tag_name || '');
    let assetUrl;
    try { assetUrl = new URL(installer?.browser_download_url || ''); } catch { assetUrl = null; }
    const expectedPath = `/${repository}/releases/download/${release.tag_name}/Nagi-Installer.exe`;
    if (release.draft === true || release.prerelease === true || !tag || !assetUrl
      || assetUrl.protocol !== 'https:' || assetUrl.hostname !== 'github.com' || assetUrl.port
      || assetUrl.username || assetUrl.password || assetUrl.search || assetUrl.hash
      || assetUrl.pathname !== expectedPath) {
      throw new Error('Installer asset not found');
    }
    for (const link of [download, lowerDownload]) {
      link.href = assetUrl.href;
      link.removeAttribute('download');
    }
    version.textContent = `${release.tag_name} · Windows · 64-bit`;
    downloadVersion.textContent = `${release.tag_name} · 64-bit installer`;
    note.textContent = 'Latest stable release';
    downloadFootnote.textContent = 'Choose your language during setup';
  }).catch(() => {
    note.textContent = 'GitHub release info is temporarily unavailable';
    downloadFootnote.textContent = 'The current installer is still available';
  }).finally(() => {
    clearTimeout(timeout);
  });
})();
