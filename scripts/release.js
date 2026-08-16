#!/usr/bin/env node
/**
 * 升版本 → 提交 → 打 tag → 推送（触发 GitHub Actions 发布）
 *
 * 用法:
 *   npm run version                 # patch
 *   npm run version -- minor
 *   npm run version -- major
 *   npm run version -- 8.3.0
 *   npm run version -- patch --dry-run   # 只升版本不推送
 *
 * 环境变量:
 *   RELEASE_SKIP_PUSH=1  只本地 commit + tag，不 push
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const VERSION_FILES = [
  "package.json",
  "package-lock.json",
  "src-tauri/tauri.conf.json",
  "src-tauri/Cargo.toml",
  "src/constants/app.ts",
];

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    cwd: root,
    encoding: "utf8",
    shell: process.platform === "win32",
    stdio: opts.silent ? "pipe" : "inherit",
    ...opts,
  });
  if (r.error) throw r.error;
  if (r.status !== 0) {
    const err = (r.stderr || r.stdout || "").toString().trim();
    throw new Error(err || `${cmd} ${args.join(" ")} failed (exit ${r.status})`);
  }
  return (r.stdout || "").toString().trim();
}

function runOut(cmd, args) {
  const r = spawnSync(cmd, args, {
    cwd: root,
    encoding: "utf8",
    shell: process.platform === "win32",
  });
  if (r.error) throw r.error;
  if (r.status !== 0) {
    throw new Error((r.stderr || r.stdout || "").toString().trim() || "command failed");
  }
  return (r.stdout || "").toString().trim();
}

function parseArgs(argv) {
  const flags = new Set();
  const positional = [];
  for (const a of argv) {
    if (a === "--dry-run" || a === "-n") flags.add("dry-run");
    else if (a === "--yes" || a === "-y") flags.add("yes");
    else if (a === "--no-push") flags.add("no-push");
    else if (a.startsWith("-")) {
      throw new Error(`未知参数: ${a}`);
    } else {
      positional.push(a);
    }
  }
  return {
    bumpArg: positional[0] || "patch",
    dryRun: flags.has("dry-run"),
    noPush: flags.has("no-push") || process.env.RELEASE_SKIP_PUSH === "1",
  };
}

function assertGitRepo() {
  try {
    runOut("git", ["rev-parse", "--is-inside-work-tree"]);
  } catch {
    throw new Error("当前目录不是 git 仓库");
  }
}

function currentBranch() {
  return runOut("git", ["branch", "--show-current"]);
}

function hasUpstream() {
  try {
    runOut("git", ["rev-parse", "--abbrev-ref", "@{upstream}"]);
    return true;
  } catch {
    return false;
  }
}

function porcelain() {
  return runOut("git", ["status", "--porcelain"]);
}

function ensureCleanOrOnlyVersionFiles() {
  const status = porcelain();
  if (!status) return;

  const lines = status.split(/\r?\n/).filter(Boolean);
  const unexpected = [];
  for (const line of lines) {
    // XY PATH or XY PATH -> PATH2
    const pathPart = line.slice(3).split(" -> ").pop().replace(/^"|"$/g, "");
    const norm = pathPart.replace(/\\/g, "/");
    if (!VERSION_FILES.includes(norm)) {
      unexpected.push(line);
    }
  }
  if (unexpected.length) {
    console.error("工作区有未提交的非版本文件，请先提交或 stash：\n");
    for (const l of unexpected) console.error("  " + l);
    console.error("\n处理后再执行 npm run version");
    process.exit(1);
  }
}

function tagExists(tag) {
  try {
    runOut("git", ["rev-parse", "-q", "--verify", `refs/tags/${tag}`]);
    return true;
  } catch {
    return false;
  }
}

function readPkgVersion() {
  const pkg = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  return pkg.version;
}

function main() {
  const { bumpArg, dryRun, noPush } = parseArgs(process.argv.slice(2));

  assertGitRepo();
  ensureCleanOrOnlyVersionFiles();

  const oldVersion = readPkgVersion();
  console.log(`\n==> 升版本 (${bumpArg})  当前 ${oldVersion}`);

  // 1) bump
  run("node", [path.join("scripts", "bump-version.js"), bumpArg]);
  const newVersion = readPkgVersion();
  const tag = `v${newVersion}`;

  if (tagExists(tag)) {
    throw new Error(`标签已存在: ${tag}，请换版本号或删除旧标签`);
  }

  const branch = currentBranch() || "main";
  console.log(`\n==> 将提交并打标签 ${tag}（分支: ${branch}）`);

  if (dryRun) {
    console.log("\n[dry-run] 已写入版本文件，未 commit / tag / push");
    console.log("恢复可用: git checkout -- " + VERSION_FILES.join(" "));
    return;
  }

  // 2) commit
  run("git", ["add", ...VERSION_FILES]);
  // 若无变更（例如指定相同版本），仍允许继续打 tag 的情况较少，直接失败更清晰
  const staged = runOut("git", ["diff", "--cached", "--name-only"]);
  if (!staged) {
    throw new Error("版本文件无变更，无法创建发布提交");
  }

  run("git", ["commit", "-m", `chore: release ${tag}`]);

  // 3) tag
  run("git", ["tag", "-a", tag, "-m", `Release ${tag}`]);

  if (noPush) {
    console.log("\n已本地 commit + tag，跳过 push（--no-push / RELEASE_SKIP_PUSH）");
    console.log(`手动推送: git push && git push origin ${tag}`);
    return;
  }

  // 4) push branch + tag
  console.log("\n==> 推送到远程…");
  if (hasUpstream()) {
    run("git", ["push"]);
  } else {
    run("git", ["push", "-u", "origin", branch]);
  }
  run("git", ["push", "origin", tag]);

  console.log(`
✅ 发布已触发
  版本: ${oldVersion} → ${newVersion}
  标签: ${tag}
  流水线: https://github.com/Livancen/source_transform/actions
  Releases: https://github.com/Livancen/source_transform/releases
`);
}

try {
  main();
} catch (e) {
  console.error("\n❌ 发布失败:", e.message || e);
  process.exit(1);
}
