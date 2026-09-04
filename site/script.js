const root = document.documentElement;
const languageButton = document.querySelector('[data-language-toggle]');
const menuButton = document.querySelector('[data-menu-toggle]');
const navigation = document.querySelector('[data-nav]');
const header = document.querySelector('[data-header]');
const windowsDownloadLinks = [...document.querySelectorAll('[data-windows-download]')];
const platformDownloadLinks = [...document.querySelectorAll('[data-platform-download]')];
const releaseVersionNodes = [...document.querySelectorAll('[data-release-version]')];
const releasePlatformsZh = document.querySelector('[data-release-platforms-zh]');
const releasePlatformsEn = document.querySelector('[data-release-platforms-en]');

const releaseApiUrl = 'https://api.github.com/repos/xiajiadi/verilecture-v3/releases?per_page=10';
const fallbackRelease = {
  tag: 'v0.3.0-alpha.4',
  assets: [
    {
      name: 'VeriLecture_0.3.0-alpha.4_x64-setup.exe',
      browser_download_url: 'https://github.com/xiajiadi/verilecture-v3/releases/download/v0.3.0-alpha.4/VeriLecture_0.3.0-alpha.4_x64-setup.exe',
    },
    {
      name: 'VeriLecture_0.3.0-alpha.4_amd64.AppImage',
      browser_download_url: 'https://github.com/xiajiadi/verilecture-v3/releases/download/v0.3.0-alpha.4/VeriLecture_0.3.0-alpha.4_amd64.AppImage',
    },
    {
      name: 'VeriLecture_0.3.0-alpha.4_aarch64.dmg',
      browser_download_url: 'https://github.com/xiajiadi/verilecture-v3/releases/download/v0.3.0-alpha.4/VeriLecture_0.3.0-alpha.4_aarch64.dmg',
    },
  ],
};

const languageLabels = {
  zh: {
    html: 'zh-CN',
    button: '切换为英文',
    navigation: '主要导航',
    title: '课溯 · VeriLecture｜让复习重点回到课堂原音',
  },
  en: {
    html: 'en',
    button: 'Switch to Chinese',
    navigation: 'Primary navigation',
    title: '课溯 · VeriLecture | Review points tied to lecture audio',
  },
};

function applyLanguage(language) {
  const safeLanguage = language === 'en' ? 'en' : 'zh';
  const labels = languageLabels[safeLanguage];
  root.lang = labels.html;
  languageButton?.setAttribute('aria-label', labels.button);
  navigation?.setAttribute('aria-label', labels.navigation);
  document.title = labels.title;
  document.querySelectorAll('[data-aria-zh][data-aria-en]').forEach((node) => {
    node.setAttribute('aria-label', node.dataset[`aria${safeLanguage === 'en' ? 'En' : 'Zh'}`]);
  });
  document.querySelectorAll('[data-alt-zh][data-alt-en]').forEach((image) => {
    image.alt = image.dataset[`alt${safeLanguage === 'en' ? 'En' : 'Zh'}`];
  });
  try {
    window.localStorage.setItem('verilecture-site-language', safeLanguage);
  } catch {
    // The language toggle remains functional when storage is unavailable.
  }
}

let storedLanguage = 'zh';
try {
  storedLanguage = window.localStorage.getItem('verilecture-site-language') || 'zh';
} catch {
  storedLanguage = 'zh';
}
applyLanguage(storedLanguage);

const sameDocumentHashLinks = [...document.querySelectorAll('a[href^="#"]:not(.skip-link)')];
let pendingHashFrame = 0;
let instantScrollTimer = 0;
let previousScrollBehavior = null;

function getLocalHash(link) {
  const url = new URL(link.href, window.location.href);
  if (
    url.origin !== window.location.origin ||
    url.pathname !== window.location.pathname ||
    url.search !== window.location.search ||
    !url.hash
  ) {
    return null;
  }

  return url.hash;
}

function getHashTarget(hash) {
  if (!hash) return null;

  try {
    return document.getElementById(decodeURIComponent(hash.slice(1)));
  } catch {
    return null;
  }
}

function closeNavigation() {
  menuButton?.setAttribute('aria-expanded', 'false');
  navigation?.classList.remove('is-open');
}

function getAnchorOffset() {
  const headerHeight = header?.getBoundingClientRect().height || 0;
  const configuredOffset = Number.parseFloat(getComputedStyle(root).scrollPaddingTop);
  const safeConfiguredOffset = Number.isFinite(configuredOffset) ? configuredOffset : 0;

  return Math.max(headerHeight + 11, safeConfiguredOffset);
}

function scrollToPosition(top) {
  if (!instantScrollTimer) previousScrollBehavior = root.style.scrollBehavior;
  root.style.scrollBehavior = 'auto';
  window.scrollTo(0, top);
  window.clearTimeout(instantScrollTimer);
  instantScrollTimer = window.setTimeout(() => {
    root.style.scrollBehavior = previousScrollBehavior || '';
    previousScrollBehavior = null;
    instantScrollTimer = 0;
  }, 160);
}

function scrollToHash(hash) {
  const target = getHashTarget(hash);
  if (!target) return;

  const targetTop = target.getBoundingClientRect().top + window.scrollY;
  const maxScrollTop = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
  const nextScrollTop = Math.min(
    maxScrollTop,
    Math.max(0, targetTop - getAnchorOffset()),
  );

  scrollToPosition(nextScrollTop);
}

function queueHashNavigation(hash = window.location.hash) {
  window.cancelAnimationFrame(pendingHashFrame);
  pendingHashFrame = window.requestAnimationFrame(() => {
    pendingHashFrame = window.requestAnimationFrame(() => {
      if (hash) {
        scrollToHash(hash);
      } else {
        window.scrollTo(0, 0);
      }
    });
  });
}

function applyHashNavigation(hash = window.location.hash) {
  if (hash) {
    scrollToHash(hash);
  } else {
    scrollToPosition(0);
  }
  queueHashNavigation(hash);
}

sameDocumentHashLinks.forEach((link) => {
  link.addEventListener('click', (event) => {
    if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

    const hash = getLocalHash(link);
    if (!hash || !getHashTarget(hash)) return;

    event.preventDefault();
    closeNavigation();
    if (window.location.hash !== hash) {
      window.history.pushState(null, '', hash);
    }
    applyHashNavigation(hash);
  });
});

try {
  window.history.scrollRestoration = 'manual';
} catch {}

const handleHistoryNavigation = () => {
  closeNavigation();
  applyHashNavigation(window.location.hash);
};

window.addEventListener('popstate', handleHistoryNavigation);
window.addEventListener('hashchange', handleHistoryNavigation);
window.addEventListener('pageshow', () => {
  if (window.location.hash) applyHashNavigation(window.location.hash);
});

if (window.location.hash) applyHashNavigation(window.location.hash);

function isWindowsInstaller(asset) {
  return /\.exe$/i.test(asset?.name || '') && /(setup|installer)/i.test(asset.name);
}

function isLinuxPackage(asset) {
  return /\.appimage$/i.test(asset?.name || '');
}

function isMacPackage(asset) {
  return /\.dmg$/i.test(asset?.name || '');
}

function setReleaseVersion(tag) {
  releaseVersionNodes.forEach((node) => {
    node.textContent = tag;
  });
}

function setWindowsDownload(asset, tag) {
  if (!asset?.browser_download_url) return;

  windowsDownloadLinks.forEach((link) => {
    link.href = asset.browser_download_url;
    link.download = asset.name;
    link.dataset.releaseTag = tag;
  });
  setReleaseVersion(tag);
}

function setPlatformDownloads(release) {
  const assets = Array.isArray(release.assets) ? release.assets : [];
  const platformMatchers = {
    windows: isWindowsInstaller,
    linux: isLinuxPackage,
    macos: isMacPackage,
  };

  platformDownloadLinks.forEach((link) => {
    const matcher = platformMatchers[link.dataset.platformDownload];
    const asset = matcher ? assets.find(matcher) : null;
    if (!asset?.browser_download_url) {
      link.hidden = true;
      return;
    }

    link.hidden = false;
    link.href = asset.browser_download_url;
    link.download = asset.name;
    link.dataset.releaseTag = release.tag;
  });
}

function setPlatformSummary(release) {
  const assets = Array.isArray(release.assets) ? release.assets : [];
  const hasWindows = assets.some(isWindowsInstaller);
  const hasLinux = assets.some(isLinuxPackage);
  const hasMac = assets.some(isMacPackage);
  const availableZh = [];
  const availableEn = [];
  const pendingZh = [];
  const pendingEn = [];

  if (hasWindows) {
    availableZh.push('Windows x64');
    availableEn.push('Windows x64');
  } else {
    pendingZh.push('Windows x64');
    pendingEn.push('Windows x64');
  }
  if (hasLinux) {
    availableZh.push('Linux AppImage');
    availableEn.push('Linux AppImage');
  } else {
    pendingZh.push('Linux AppImage');
    pendingEn.push('Linux AppImage');
  }
  if (hasMac) {
    availableZh.push('macOS DMG');
    availableEn.push('macOS DMG');
  } else {
    pendingZh.push('macOS DMG');
    pendingEn.push('macOS DMG');
  }

  if (releasePlatformsZh) {
    const availableLabelZh = availableZh.join('、');
    const availableStatusZh = availableZh.length
      ? `已发布 ${availableLabelZh}。`
      : '暂无可下载的桌面包。';
    const pendingStatusZh = pendingZh.length
      ? `尚未发布 ${pendingZh.join('、')}。`
      : '';
    releasePlatformsZh.textContent = [
      `版本 ${release.tag}：${availableStatusZh}`,
      pendingStatusZh,
      '本地 ASR 状态见平台说明。',
    ]
      .filter(Boolean)
      .join('');
  }
  if (releasePlatformsEn) {
    const availableLabelEn = availableEn.join(', ');
    releasePlatformsEn.textContent = `Release ${release.tag}: ${availableEn.length ? `${availableLabelEn} package${availableEn.length === 1 ? '' : 's'} available` : 'No desktop package is available'}${pendingEn.length ? `; not published yet: ${pendingEn.join(' and ')}.` : '.'} Local ASR support is shown for each platform.`;
  }
}

async function updateReleaseDownload() {
  try {
    const response = await fetch(releaseApiUrl, {
      headers: { Accept: 'application/vnd.github+json' },
      cache: 'no-store',
    });
    if (!response.ok) throw new Error(`Release API returned ${response.status}`);

    const releases = await response.json();
    const release = releases.find((candidate) => !candidate.draft && Array.isArray(candidate.assets));
    if (!release) return;

    setPlatformSummary({
      tag: release.tag_name,
      assets: release.assets,
    });
    setPlatformDownloads({
      tag: release.tag_name,
      assets: release.assets,
    });

    const windowsAsset = release.assets.find(isWindowsInstaller);
    if (windowsAsset) setWindowsDownload(windowsAsset, release.tag_name);
  } catch {
    // The fallback points to the last known published release.
    setPlatformSummary(fallbackRelease);
    setPlatformDownloads(fallbackRelease);
    setWindowsDownload(fallbackRelease.assets.find(isWindowsInstaller), fallbackRelease.tag);
  }
}

setPlatformSummary(fallbackRelease);
setPlatformDownloads(fallbackRelease);
setWindowsDownload(fallbackRelease.assets.find(isWindowsInstaller), fallbackRelease.tag);
updateReleaseDownload();

languageButton?.addEventListener('click', () => {
  applyLanguage(root.lang === 'en' ? 'zh' : 'en');
});

menuButton?.addEventListener('click', () => {
  const open = menuButton.getAttribute('aria-expanded') === 'true';
  menuButton.setAttribute('aria-expanded', String(!open));
  navigation?.classList.toggle('is-open', !open);
});

document.addEventListener('keydown', (event) => {
  if (event.key !== 'Escape' || menuButton?.getAttribute('aria-expanded') !== 'true') return;
  closeNavigation();
  menuButton.focus();
});

document.addEventListener('click', (event) => {
  if (!(event.target instanceof Element)) return;
  document.querySelectorAll('[data-download-picker][open]').forEach((picker) => {
    if (!picker.contains(event.target)) picker.removeAttribute('open');
  });
});

document.addEventListener('keydown', (event) => {
  if (event.key !== 'Escape') return;
  const openPicker = document.querySelector('[data-download-picker][open]');
  if (!openPicker) return;
  openPicker.removeAttribute('open');
  openPicker.querySelector('summary')?.focus();
});

document.querySelectorAll('[data-platform-download]').forEach((link) => {
  link.addEventListener('click', () => {
    link.closest('[data-download-picker]')?.removeAttribute('open');
  });
});

window.addEventListener(
  'scroll',
  () => {
    header?.classList.toggle('is-scrolled', window.scrollY > 18);
  },
  { passive: true },
);

const shots = {
  'result-points': {
    src: './assets/product-trace-result.webp',
    altZh: '课溯 Windows x64 实机结果页：复习重点与回听',
    altEn: 'VeriLecture Windows x64 result screen with review points and playback',
    count: '01 / 04',
    captionZh: 'Windows x64 实机运行 · 重点带有来源时间点，可回听原音。',
    captionEn: 'Windows x64 run · review points include timestamps for playback.',
  },
  audio: {
    src: './assets/product-audio-import.webp',
    altZh: '课溯导入课堂录音界面',
    altEn: 'VeriLecture screen for importing lecture audio',
    count: '02 / 04',
    captionZh: '导入前先说明哪些内容留在本机，哪些步骤需要授权。',
    captionEn: 'Before import, the screen shows what stays local and what needs consent.',
  },
  settings: {
    src: './assets/product-settings.png',
    altZh: '课溯模型与硬件设置界面',
    altEn: 'VeriLecture model and hardware settings screen',
    count: '03 / 04',
    captionZh: '设备、模型和数据边界集中显示；可用路线由硬件条件决定。',
    captionEn: 'Device, model, and data settings appear together; hardware determines the local route.',
  },
  lexicon: {
    src: './assets/product-lexicon.webp',
    altZh: '课溯专业词库界面',
    altEn: 'VeriLecture course terms screen',
    count: '04 / 04',
    captionZh: '教材先在本机解析；专业术语可用于后续校准。',
    captionEn: 'Course material is parsed locally first; its terms can guide later checks.',
  },
};

const showcaseImage = document.querySelector('[data-showcase-image]');
const stageCount = document.querySelector('[data-stage-count]');
const captionZh = document.querySelector('[data-caption-zh]');
const captionEn = document.querySelector('[data-caption-en]');
const shotButtons = [...document.querySelectorAll('[data-shot]')];

shotButtons.forEach((button) => {
  button.addEventListener('click', () => {
    const shot = shots[button.dataset.shot];
    if (!shot || !showcaseImage) return;

    shotButtons.forEach((candidate) => {
      candidate.setAttribute('aria-selected', String(candidate === button));
    });

    showcaseImage.classList.add('is-changing');
    window.setTimeout(() => {
      const finishChange = () => showcaseImage.classList.remove('is-changing');
      showcaseImage.addEventListener('load', finishChange, { once: true });
      showcaseImage.dataset.altZh = shot.altZh;
      showcaseImage.dataset.altEn = shot.altEn;
      showcaseImage.alt = root.lang === 'en' ? shot.altEn : shot.altZh;
      showcaseImage.src = shot.src;
      if (stageCount) stageCount.textContent = shot.count;
      if (captionZh) captionZh.textContent = shot.captionZh;
      if (captionEn) captionEn.textContent = shot.captionEn;
      if (showcaseImage.complete) finishChange();
    }, 180);
  });

  button.addEventListener('keydown', (event) => {
    const currentIndex = shotButtons.indexOf(button);
    let nextIndex;
    if (event.key === 'ArrowRight') nextIndex = (currentIndex + 1) % shotButtons.length;
    if (event.key === 'ArrowLeft') nextIndex = (currentIndex - 1 + shotButtons.length) % shotButtons.length;
    if (event.key === 'Home') nextIndex = 0;
    if (event.key === 'End') nextIndex = shotButtons.length - 1;
    if (nextIndex === undefined) return;
    event.preventDefault();
    shotButtons[nextIndex].focus();
    shotButtons[nextIndex].click();
  });
});

const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const revealItems = document.querySelectorAll('.reveal');

if (reducedMotion || !('IntersectionObserver' in window)) {
  revealItems.forEach((item) => item.classList.add('is-visible'));
} else {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add('is-visible');
        observer.unobserve(entry.target);
      });
    },
    { threshold: 0.12, rootMargin: '0px 0px -5% 0px' },
  );
  revealItems.forEach((item) => observer.observe(item));
}

document.querySelectorAll('[data-year]').forEach((node) => {
  node.textContent = String(new Date().getFullYear());
});
