# 自动更新与发布说明

## 1. 更新机制

- 使用 `@tauri-apps/plugin-updater`
- 启动约 1.5s 后检查：
  `https://github.com/Livancen/source_transform/releases/latest/download/latest.json`
- 有新版本 → 弹窗 → 用户确认 → 下载进度条 → 安装并重启

## 2. 签名密钥（必须配置）

更新包需要签名，**私钥绝不可提交到仓库**。

本仓库 `tauri.conf.json` 已写入 **公钥**。  
私钥请保存到安全位置，并配置到 GitHub Secrets：

| Secret 名称 | 说明 |
|-------------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | 私钥文件内容（整段 base64）或私钥路径内容 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码（若生成时未设密码可留空） |

本地重新生成密钥（会更换公钥，需同步改 `tauri.conf.json`）：

```bash
npm run tauri signer generate -w ~/.tauri/source_transform.key
```

## 3. GitHub Actions 发布

工作流：`.github/workflows/release.yml`

### 更新日志（CHANGELOG.md）

Release 正文从根目录 `CHANGELOG.md` 按版本号提取，**发版前请先写好对应段落**：

```markdown
## [8.6.2] - 2026-09-10

1. 新功能说明
2. 修复说明
```

流水线会匹配 `## [当前 package.json 版本]`，取该标题到下一版本标题之间的内容作为 GitHub Release Body；找不到则失败。

### 触发方式（推荐一键）

```bash
# 1) 先在 CHANGELOG.md 写好新版本说明
# 2) 升版本 + commit + 打 tag + push（触发 GitHub Actions 发布）
npm run version                 # patch  8.2.0 → 8.2.1
npm run version -- minor        # minor  8.2.0 → 8.3.0
npm run version -- major        # major  8.2.0 → 9.0.0
npm run version -- 8.3.0        # 指定版本

# 仅预览升版本、不提交推送
npm run version -- patch --dry-run

# 本地 commit + tag，不 push
npm run version -- patch --no-push
```

要求：工作区干净（或仅有版本相关文件改动）。脚本会拒绝在存在其他未提交改动时发布。
### 手动方式

```bash
npm run version:bump -- minor
git add -A
git commit -m "chore: release v8.3.0"
git tag v8.3.0
git push && git push origin v8.3.0
```

### 产物命名

格式：`source_transform_[版本]_[平台]-[架构][setup].[后缀]`

示例：

| 平台 | 文件名示例 |
|------|------------|
| Windows NSIS | `source_transform_8.2.0_windows-x86_64_setup.exe` |
| Windows MSI | `source_transform_8.2.0_windows-x86_64.msi` |
| macOS arm64 | `source_transform_8.2.0_darwin-aarch64.dmg` |
| macOS x64 | `source_transform_8.2.0_darwin-x86_64.dmg` |

另有对应 `.sig` 签名文件，以及 `latest.json`（更新器用）。

> 多平台会共用版本号，因此文件名中保留了平台/架构，避免 mac arm/x64 的 `.dmg` 互相覆盖。

## 4. Sidecar 二进制（构建时自动下载）

应用依赖 `ffmpeg` / `ffprobe` / `magick` 作为 sidecar。

**CI 会在构建前自动执行** `scripts/download-sidecars.sh`，无需把 macOS 二进制提交进仓库。

| 平台 | 来源 |
|------|------|
| Windows x64 | 仓库已有则跳过；否则从 BtbN FFmpeg + ImageMagick portable 下载 |
| macOS arm64 / x64 | ffmpeg/ffprobe 静态包 + brew ImageMagick |

本地手动准备：

```bash
# Windows (Git Bash)
bash scripts/download-sidecars.sh x86_64-pc-windows-msvc

# macOS Apple Silicon
bash scripts/download-sidecars.sh aarch64-apple-darwin

# macOS Intel
bash scripts/download-sidecars.sh x86_64-apple-darwin
```

命名约定：

| 平台 | 命名示例 |
|------|----------|
| Windows x64 | `ffmpeg-x86_64-pc-windows-msvc.exe` |
| macOS arm64 | `ffmpeg-aarch64-apple-darwin` |
| macOS x64 | `ffmpeg-x86_64-apple-darwin` |

## 5. 本地测试更新（可选）

1. 构建旧版本安装并运行  
2. 发布更高版本到 GitHub Releases  
3. 启动旧版本应弹出更新提示  

开发模式（`tauri dev`）下检查更新可能因无 endpoint 资源而失败，属正常，会静默忽略。
