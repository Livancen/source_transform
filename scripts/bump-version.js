#!/usr/bin/env node
/**
 * 统一提升应用版本号
 *
 * 用法:
 *   node scripts/bump-version.js              # patch: 8.2.0 -> 8.2.1
 *   node scripts/bump-version.js patch
 *   node scripts/bump-version.js minor       # 8.2.0 -> 8.3.0
 *   node scripts/bump-version.js major       # 8.2.0 -> 9.0.0
 *   node scripts/bump-version.js 8.3.0       # 指定版本
 *
 * npm:
 *   npm run version:bump
 *   npm run version:bump -- minor
 *   npm run version:bump -- 8.3.0
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const SEMVER = /^(\d+)\.(\d+)\.(\d+)(?:-[\w.-]+)?$/;

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function writeJson(file, data) {
  fs.writeFileSync(file, JSON.stringify(data, null, 2) + "\n", "utf8");
}

function readText(file) {
  return fs.readFileSync(file, "utf8");
}

function writeText(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function parseSemver(v) {
  const m = String(v).trim().replace(/^v/i, "").match(SEMVER);
  if (!m) return null;
  return { major: +m[1], minor: +m[2], patch: +m[3], raw: `${m[1]}.${m[2]}.${m[3]}` };
}

function bump(from, kind) {
  const cur = parseSemver(from);
  if (!cur) throw new Error(`当前版本无效: ${from}`);

  if (kind === "major") return `${cur.major + 1}.0.0`;
  if (kind === "minor") return `${cur.major}.${cur.minor + 1}.0`;
  if (kind === "patch") return `${cur.major}.${cur.minor}.${cur.patch + 1}`;

  const explicit = parseSemver(kind);
  if (explicit) return explicit.raw;

  throw new Error(`参数无效: ${kind}（可用 patch | minor | major | x.y.z）`);
}

function compareSemver(a, b) {
  const pa = parseSemver(a);
  const pb = parseSemver(b);
  if (!pa || !pb) return 0;
  if (pa.major !== pb.major) return pa.major - pb.major;
  if (pa.minor !== pb.minor) return pa.minor - pb.minor;
  return pa.patch - pb.patch;
}

const pkgPath = path.join(root, "package.json");
const lockPath = path.join(root, "package-lock.json");
const tauriPath = path.join(root, "src-tauri", "tauri.conf.json");
const cargoPath = path.join(root, "src-tauri", "Cargo.toml");
const appConstPath = path.join(root, "src", "constants", "app.ts");

const arg = (process.argv[2] || "patch").trim();
const pkg = readJson(pkgPath);
const oldVersion = pkg.version;
const newVersion = bump(oldVersion, arg);

if (compareSemver(newVersion, oldVersion) <= 0 && parseSemver(arg)) {
  console.warn(
    `警告: 新版本 ${newVersion} 未高于当前 ${oldVersion}，自动更新可能不会触发。`,
  );
}

// 1. package.json
pkg.version = newVersion;
writeJson(pkgPath, pkg);

// 2. package-lock.json（仅根包）
if (fs.existsSync(lockPath)) {
  const lock = readJson(lockPath);
  lock.version = newVersion;
  if (lock.packages && lock.packages[""]) {
    lock.packages[""].version = newVersion;
  }
  writeJson(lockPath, lock);
}

// 3. tauri.conf.json
const tauri = readJson(tauriPath);
tauri.version = newVersion;
writeJson(tauriPath, tauri);

// 4. Cargo.toml
let cargo = readText(cargoPath);
if (!/^version\s*=\s*"/m.test(cargo)) {
  throw new Error("Cargo.toml 中未找到 version 字段");
}
cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${newVersion}"`);
writeText(cargoPath, cargo);

// 5. APP_VERSION 常量
if (fs.existsSync(appConstPath)) {
  let appTs = readText(appConstPath);
  if (!/export const APP_VERSION\s*=\s*["'][^"']*["']/.test(appTs)) {
    throw new Error("src/constants/app.ts 中未找到 APP_VERSION");
  }
  appTs = appTs.replace(
    /export const APP_VERSION\s*=\s*["'][^"']*["']/,
    `export const APP_VERSION = "${newVersion}"`,
  );
  writeText(appConstPath, appTs);
}

console.log(`版本已更新: ${oldVersion} → ${newVersion}`);
console.log("");
console.log("已同步文件:");
console.log("  - package.json");
console.log("  - package-lock.json");
console.log("  - src-tauri/tauri.conf.json");
console.log("  - src-tauri/Cargo.toml");
console.log("  - src/constants/app.ts");
console.log("");
console.log("发布步骤:");
console.log(`  git add -A`);
console.log(`  git commit -m "chore: release v${newVersion}"`);
console.log(`  git tag v${newVersion}`);
console.log(`  git push && git push origin v${newVersion}`);
