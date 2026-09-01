const root = document.documentElement;
const languageButton = document.querySelector('[data-language-toggle]');
const menuButton = document.querySelector('[data-menu-toggle]');
const navigation = document.querySelector('[data-nav]');
const header = document.querySelector('[data-header]');

const languageLabels = {
  zh: {
    html: 'zh-CN',
    button: '切换为英文',
    navigation: '主要导航',
    title: '课溯 · VeriLecture｜让每一堂课，都有迹可循',
  },
  en: {
    html: 'en',
    button: 'Switch to Simplified Chinese',
    navigation: 'Primary navigation',
    title: 'VeriLecture | Every lecture, traceable',
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
  menuButton.setAttribute('aria-expanded', 'false');
  navigation?.classList.remove('is-open');
  menuButton.focus();
});

navigation?.querySelectorAll('a').forEach((link) => {
  link.addEventListener('click', () => {
    menuButton?.setAttribute('aria-expanded', 'false');
    navigation.classList.remove('is-open');
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
  audio: {
    src: './assets/product-audio-import.webp',
    altZh: '课溯导入课堂录音界面',
    altEn: 'VeriLecture lecture audio import screen',
    count: '01 / 04',
    captionZh: '导入时就说明：什么留在本机，什么需要你的授权。',
    captionEn: 'Import starts by showing what stays local and what requires your permission.',
  },
  records: {
    src: './assets/product-audio-records.webp',
    altZh: '课溯音频记录界面',
    altEn: 'VeriLecture audio records screen',
    count: '02 / 04',
    captionZh: '每一段录音都是证据链的入口，原始内容始终保留。',
    captionEn: 'Every recording is an entry point to its evidence trail, with the source preserved.',
  },
  lexicon: {
    src: './assets/product-lexicon.webp',
    altZh: '课溯专业词库界面',
    altEn: 'VeriLecture course terminology screen',
    count: '03 / 04',
    captionZh: '教材先在本机解析，专业术语成为后续校准的可靠约束。',
    captionEn: 'Course material is parsed locally first, grounding later calibration in real terminology.',
  },
  onboarding: {
    src: './assets/product-onboarding.webp',
    altZh: '课溯首次使用音频导入引导',
    altEn: 'VeriLecture first-run audio import guide',
    count: '04 / 04',
    captionZh: '不要求先理解复杂设置，从一段熟悉的课堂录音开始。',
    captionEn: 'No complex setup knowledge required — begin with a lecture you already know.',
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
