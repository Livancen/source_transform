# 素材转换工具 (source_transform)

[![Release](https://img.shields.io/github/v/release/Livancen/source_transform?include_prereleases&style=flat-square)](https://github.com/Livancen/source_transform/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](./LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey?style=flat-square)](https://github.com/Livancen/source_transform/releases)

本地批量处理 **图片** 与 **视频** 的桌面应用。基于 Tauri 2 + Vue 3，内置 FFmpeg / ImageMagick，无需单独配置环境。

---

## 功能概览

应用按工作模式划分，图片与视频互不混合处理；复杂操作使用独立工作台。

| 模式 | 说明 |
|------|------|
| **图片** | 批量处理图片：压缩、改分辨率、转格式、旋转 |
| **视频** | 批量处理视频：压缩、分辨率、码率、Profile/Level、H.265↔H.264、转格式、旋转、静音、帧率 |
| **比例裁剪** | 按多个 W:H 比例批量居中裁剪（图 + 视频），使用专用命名规则 |
| **自定义裁剪** | 单文件像素级框选裁剪，支持图片与视频 |
| **拼接** | 双视频或双图片上下/左右拼接 |

### 处理能力明细

| 能力 | 图片 | 视频 |
|------|:----:|:----:|
| 压缩（质量 / 可选降分辨率） | ✅ | ✅ |
| 单独改分辨率 | ✅ | ✅ |
| 转格式 | ✅ | ✅ |
| 旋转 | ✅ | ✅ |
| 降码率 | — | ✅ |
| Profile / Level | — | ✅ |
| H.265 → H.264 | — | ✅ |
| H.264 → H.265 | — | ✅ |
| 静音 | — | ✅ |
| 调整帧率 | — | ✅ |
| 比例裁剪 | ✅ | ✅ |
| 自定义裁剪 | ✅ | ✅ |
| 双路拼接 | ✅ | ✅ |

### 其他特性

- **勾选处理**：输入列表支持勾选，仅处理已选文件
- **预览**：缩略图 + 点击预览（图片原图 / 视频可播放）
- **输出命名**：设置页可配置原名 / 时间戳 / 标准时间 / 自定义文本（比例裁剪除外）
- **局域网上传**：启动后提供上传链接，可将文件传到输入目录
- **自动更新**：启动检测新版本，确认后下载并安装

---

## 支持格式

| 类型 | 扩展名 |
|------|--------|
| 图片 | `jpg` `jpeg` `png` `gif` `bmp` `webp` `tiff` |
| 视频 | `mp4` `avi` `mov` `mkv` `wmv` `flv` `webm` `m4v` |

---

## 下载安装

前往 [Releases](https://github.com/Livancen/source_transform/releases) 下载对应平台安装包：

| 平台 | 文件名示例 |
|------|------------|
| Windows | `source_transform_x.y.z_windows-x86_64_setup.exe` |
| macOS Apple Silicon | `source_transform_x.y.z_darwin-aarch64.dmg` |
| macOS Intel | `source_transform_x.y.z_darwin-x86_64.dmg` |

安装后首次使用：在设置中指定 **输入目录** 与 **输出目录**，或使用应用数据目录下的默认路径。

---

## 快速上手

1. 选择 **图片** 或 **视频** 模式（列表会按类型过滤）
2. 勾选要处理的文件，打开处理选项并配置参数
3. 点击 **开始处理**，底部状态栏显示进度
4. 结果写入输出目录；比例裁剪、自定义裁剪、拼接使用各自入口

### 比例裁剪命名

```
原名-比例(比值)-(范围枚举)-(日期).扩展名
```

示例：`促销活动A-1-1(1.00)-(0.94～1.05)-(260322).mp4`

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | [Tauri 2](https://tauri.app/) |
| 前端 | Vue 3、Vue Router、TypeScript、Vite、UnoCSS |
| 后端 | Rust |
| 媒体处理 | FFmpeg / FFprobe（视频）、ImageMagick（图片） |
| 更新 | `@tauri-apps/plugin-updater` + GitHub Releases |

---

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/) stable
- [Tauri 前置依赖](https://v2.tauri.app/start/prerequisites/)
- Windows / macOS 开发机

### 安装与运行

```bash
# 克隆
git clone https://github.com/Livancen/source_transform.git
cd source_transform

# 安装依赖
npm install

# 准备 sidecar（ffmpeg / ffprobe / magick）
# Windows (Git Bash) / macOS：
bash scripts/download-sidecars.sh

# 开发模式
npm run tauri dev
```

若仓库已包含 Windows 下 `src-tauri/binaries/` 二进制，Windows 开发可跳过下载步骤。

### 常用命令

```bash
npm run dev              # 仅前端 Vite
npm run build            # 前端类型检查 + 构建
npm run tauri build      # 打包桌面安装包
npm run version:bump     # 升 patch 版本
npm run version:bump -- minor
npm run version:bump -- 8.3.0
```

### 项目结构

```
source_transform/
├── src/                      # 前端 (Vue 3)
│   ├── components/           # UI 组件
│   ├── composables/          # 业务逻辑 hooks
│   ├── views/                # 页面（首页 / 设置）
│   └── types.ts
├── src-tauri/                # 桌面后端 (Rust)
│   ├── src/
│   │   ├── commands.rs       # Tauri 命令
│   │   ├── process.rs        # 图片 / 视频处理
│   │   ├── naming.rs         # 输出命名
│   │   └── upload_server.rs  # 局域网上传
│   ├── binaries/             # sidecar 二进制
│   └── tauri.conf.json
├── scripts/
│   ├── bump-version.js       # 统一升版
│   └── download-sidecars.sh  # 下载 FFmpeg 等
├── docs/                     # 补充文档
└── .github/workflows/        # 发布流水线
```

---

## 发布

一键升版本、打标签并推送，触发 GitHub Actions 构建安装包：

```bash
npm run version              # patch
npm run version -- minor     # minor
npm run version -- 8.3.0     # 指定版本
```

发布与自动更新所需 Secrets、签名密钥说明见：

- [docs/UPDATE_AND_RELEASE.md](./docs/UPDATE_AND_RELEASE.md)

---

## 路线图 / 后续

- [ ] 拼接自由画布布局
- [ ] 更多图片批量能力与预设
- [ ] Linux 构建与 sidecar 支持

欢迎通过 [Issues](https://github.com/Livancen/source_transform/issues) 反馈问题或建议。

---

## 许可证

[MIT](./LICENSE)

---

## 致谢

- [Tauri](https://tauri.app/)
- [FFmpeg](https://ffmpeg.org/)
- [ImageMagick](https://imagemagick.org/)
- [Vue.js](https://vuejs.org/)
