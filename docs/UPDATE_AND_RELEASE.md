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

### 触发方式

```bash
# 1. 同步版本号：package.json / src-tauri/tauri.conf.json / Cargo.toml
# 2. 打 tag 并推送
git tag v8.3.0
git push origin v8.3.0
```

也可在 Actions 页手动 `workflow_dispatch`（需有 tag 上下文时更稳妥）。

### 产物

- Windows：NSIS / MSI 安装包 + `.sig`
- macOS：`.app` / `.dmg` / `.tar.gz` + `.sig`
- `latest.json`：供更新器读取

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
