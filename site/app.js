(() => {
  const repository = window.NAGI_SITE_CONFIG?.githubRepository?.trim() || '';
  const download = document.getElementById('download');
  const lowerDownload = document.getElementById('download-bottom');
  const note = document.getElementById('release-note');
  const version = document.getElementById('version');
  const source = document.getElementById('source-link');

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
    version.textContent = `${release.tag_name} · Windows 10 / 11 · 64-bit`;
    note.textContent = 'Latest stable release · Installer language selection included';
  }).catch(() => {
    note.textContent = 'GitHub release information is temporarily unavailable. You can still download the current installer.';
  }).finally(() => {
    clearTimeout(timeout);
  });
})();
