#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const bundleRoot = path.resolve(process.cwd(), 'dist/dashboard/_app');

function readLimit(name, fallback) {
  const raw = process.env[name];
  const parsed = Number(raw || fallback);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`Invalid ${name}=${raw}; expected a positive number.`);
  }
  return parsed;
}

const limits = {
  totalBytes: readLimit('SHUMA_DASHBOARD_BUNDLE_MAX_TOTAL_BYTES', 350000),
  totalJsBytes: readLimit('SHUMA_DASHBOARD_BUNDLE_MAX_JS_BYTES', 320000),
  totalCssBytes: readLimit('SHUMA_DASHBOARD_BUNDLE_MAX_CSS_BYTES', 40000),
  maxSingleJsBytes: readLimit('SHUMA_DASHBOARD_BUNDLE_MAX_JS_CHUNK_BYTES', 150000),
  maxSingleCssBytes: readLimit('SHUMA_DASHBOARD_BUNDLE_MAX_CSS_ASSET_BYTES', 30000)
};

if (!fs.existsSync(bundleRoot)) {
  console.error(`Dashboard bundle directory is missing: ${bundleRoot}`);
  console.error('Run `make dashboard-build` before checking bundle budgets.');
  process.exit(1);
}

function walkFiles(dirPath, files) {
  const entries = fs.readdirSync(dirPath, { withFileTypes: true });
  entries.forEach((entry) => {
    const absolutePath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      walkFiles(absolutePath, files);
      return;
    }
    if (!entry.isFile()) return;
    files.push(absolutePath);
  });
}

const files = [];
walkFiles(bundleRoot, files);

const stats = {
  totalBytes: 0,
  totalJsBytes: 0,
  totalCssBytes: 0,
  maxSingleJsBytes: 0,
  maxSingleJsPath: '',
  maxSingleCssBytes: 0,
  maxSingleCssPath: ''
};

files.forEach((absolutePath) => {
  const relativePath = path.relative(process.cwd(), absolutePath);
  const size = fs.statSync(absolutePath).size;
  stats.totalBytes += size;
  if (relativePath.endsWith('.js')) {
    stats.totalJsBytes += size;
    if (size > stats.maxSingleJsBytes) {
      stats.maxSingleJsBytes = size;
      stats.maxSingleJsPath = relativePath;
    }
    return;
  }
  if (relativePath.endsWith('.css')) {
    stats.totalCssBytes += size;
    if (size > stats.maxSingleCssBytes) {
      stats.maxSingleCssBytes = size;
      stats.maxSingleCssPath = relativePath;
    }
  }
});

function formatBytes(value) {
  return `${Number(value).toLocaleString('en-US')} B`;
}

const failures = [];
if (stats.totalBytes > limits.totalBytes) {
  failures.push(
    `total dashboard _app bytes ${formatBytes(stats.totalBytes)} exceeds limit ${formatBytes(limits.totalBytes)}`
  );
}
if (stats.totalJsBytes > limits.totalJsBytes) {
  failures.push(
    `total JS bytes ${formatBytes(stats.totalJsBytes)} exceeds limit ${formatBytes(limits.totalJsBytes)}`
  );
}
if (stats.totalCssBytes > limits.totalCssBytes) {
  failures.push(
    `total CSS bytes ${formatBytes(stats.totalCssBytes)} exceeds limit ${formatBytes(limits.totalCssBytes)}`
  );
}
if (stats.maxSingleJsBytes > limits.maxSingleJsBytes) {
  failures.push(
    `largest JS asset ${stats.maxSingleJsPath} is ${formatBytes(stats.maxSingleJsBytes)} (limit ${formatBytes(limits.maxSingleJsBytes)})`
  );
}
if (stats.maxSingleCssBytes > limits.maxSingleCssBytes) {
  failures.push(
    `largest CSS asset ${stats.maxSingleCssPath} is ${formatBytes(stats.maxSingleCssBytes)} (limit ${formatBytes(limits.maxSingleCssBytes)})`
  );
}

console.log('Dashboard bundle budget report:');
console.log(`- root: ${bundleRoot}`);
console.log(`- total: ${formatBytes(stats.totalBytes)} (limit ${formatBytes(limits.totalBytes)})`);
console.log(`- js total: ${formatBytes(stats.totalJsBytes)} (limit ${formatBytes(limits.totalJsBytes)})`);
console.log(`- css total: ${formatBytes(stats.totalCssBytes)} (limit ${formatBytes(limits.totalCssBytes)})`);
console.log(
  `- largest js: ${stats.maxSingleJsPath || '-'} (${formatBytes(stats.maxSingleJsBytes)}; limit ${formatBytes(limits.maxSingleJsBytes)})`
);
console.log(
  `- largest css: ${stats.maxSingleCssPath || '-'} (${formatBytes(stats.maxSingleCssBytes)}; limit ${formatBytes(limits.maxSingleCssBytes)})`
);

if (failures.length > 0) {
  console.error('Dashboard bundle budget check failed:');
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}

console.log('Dashboard bundle budget check passed.');
