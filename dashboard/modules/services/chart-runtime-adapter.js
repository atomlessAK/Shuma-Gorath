// @ts-check

const DEFAULT_CHART_RUNTIME_SRC = '/dashboard/assets/vendor/chart-lite-1.0.0.min.js';

let loadPromise = null;
let refCount = 0;
let adapterOwnedScript = null;
let adapterOwnedGlobal = false;

const normalizeRuntimeSrc = (value) => {
  const source = String(value || '').trim();
  return source || DEFAULT_CHART_RUNTIME_SRC;
};

const chartConstructorFrom = (win) =>
  win && typeof win.Chart === 'function' ? win.Chart : null;

const markScriptReady = (script) => {
  if (!script || !script.dataset) return;
  script.dataset.shumaRuntimeReady = 'true';
};

const findRuntimeScript = (doc, src) => {
  if (!doc || typeof doc.querySelectorAll !== 'function') return null;
  const scripts = Array.from(doc.querySelectorAll('script[src]'));
  const target = String(src || '').trim();
  return scripts.find((entry) => {
    const declared = String(entry.getAttribute('data-shuma-runtime-script') || '').trim();
    const source = String(entry.getAttribute('src') || '').trim();
    return declared === target || source === target;
  }) || null;
};

const loadRuntimeScript = (doc, win, src) =>
  new Promise((resolve, reject) => {
    const existingChart = chartConstructorFrom(win);
    if (existingChart) {
      resolve(existingChart);
      return;
    }

    const done = () => {
      const chart = chartConstructorFrom(win);
      if (chart) {
        resolve(chart);
        return;
      }
      reject(new Error('Chart runtime loaded but window.Chart is unavailable.'));
    };

    const onError = () => {
      reject(new Error(`Failed to load chart runtime script: ${src}`));
    };
    const onLoad = (script) => {
      markScriptReady(script);
      done();
    };

    let script = findRuntimeScript(doc, src);
    if (!script) {
      if (!doc || typeof doc.createElement !== 'function') {
        reject(new Error('Chart runtime requires a browser document context.'));
        return;
      }
      script = doc.createElement('script');
      script.setAttribute('src', src);
      script.setAttribute('data-shuma-runtime-script', src);
      script.async = true;
      script.addEventListener('load', () => onLoad(script), { once: true });
      script.addEventListener('error', onError, { once: true });
      adapterOwnedScript = script;
      adapterOwnedGlobal = true;
      const target = doc.head || doc.body;
      if (!target || typeof target.appendChild !== 'function') {
        reject(new Error('Chart runtime could not find a script insertion target.'));
        return;
      }
      target.appendChild(script);
      const chartAfterAppend = chartConstructorFrom(win);
      if (chartAfterAppend) {
        markScriptReady(script);
        resolve(chartAfterAppend);
      }
      return;
    }

    if (script.dataset && script.dataset.shumaRuntimeReady === 'true') {
      done();
      return;
    }

    script.addEventListener('load', () => onLoad(script), { once: true });
    script.addEventListener('error', onError, { once: true });
  });

export async function acquireChartRuntime(options = {}) {
  const doc = options.document || (typeof document !== 'undefined' ? document : null);
  const win = options.window || (typeof window !== 'undefined' ? window : null);
  const src = normalizeRuntimeSrc(options.src);
  if (!doc || !win) {
    throw new Error('Chart runtime requires browser window/document context.');
  }

  if (!loadPromise) {
    loadPromise = loadRuntimeScript(doc, win, src);
  }

  try {
    const chart = await loadPromise;
    refCount += 1;
    return chart;
  } catch (error) {
    loadPromise = null;
    throw error;
  }
}

export function getChartConstructor(options = {}) {
  const win = options.window || (typeof window !== 'undefined' ? window : null);
  return chartConstructorFrom(win);
}

export function releaseChartRuntime(options = {}) {
  if (refCount > 0) {
    refCount -= 1;
  }
  if (refCount > 0) return;

  const win = options.window || (typeof window !== 'undefined' ? window : null);
  const doc = options.document || (typeof document !== 'undefined' ? document : null);
  loadPromise = null;

  if (adapterOwnedScript && adapterOwnedScript.parentNode) {
    adapterOwnedScript.parentNode.removeChild(adapterOwnedScript);
  }
  adapterOwnedScript = null;

  if (adapterOwnedGlobal && win && Object.prototype.hasOwnProperty.call(win, 'Chart')) {
    try {
      delete win.Chart;
    } catch (_error) {
      win.Chart = undefined;
    }
  }
  adapterOwnedGlobal = false;

  if (doc) {
    const orphan = findRuntimeScript(doc, DEFAULT_CHART_RUNTIME_SRC);
    if (orphan && orphan.dataset) {
      delete orphan.dataset.shumaRuntimeReady;
    }
  }
}
