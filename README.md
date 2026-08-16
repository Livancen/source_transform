# 素材转换工具 (source_transform)

[![Release](https://img.shields.io/github/v/release/Livancen/source_transform?include_prereleases&style=flat-square)](https://github.com/Livancen/source_transform/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](./LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey?style=flat-square)](https://github.com/Livancen/source_transform/releases)

本地批量处理 **图片** 与 **视频** 的桌面应用。基于 Tauri 2 + Vue 3，内置 FFmpeg / ImageMagick，无需单独配置环境。

---

## 功能概览

应用按工作模式划分。图片 / 视频 **批量处理互不混合**；裁剪与拼接使用独立工作台。

| 模式 | 说明 |
|------|------|
| **图片** | 批量处理图片：压缩、改分辨率、转格式、旋转 |
| **视频** | 批量处理视频：压缩、分辨率、码率、Profile/Level、H.265↔H.264、转格式、旋转、静音、帧率 |
| **比例裁剪** | 按多个 W:H 比例批量居中裁剪（图 + 视频），使用专用命名规则 |
| **自定义裁剪** | 单文件像素级框选裁剪，支持图片与视频 |
| **拼接** | 双视频或双图片，上下 / 左右槽位拼接 |
| **自定义拼接** | 自由画布多图层布局（最多 6 层），图 / 视频可混排 |

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
| 自定义拼接（自由画布） | ✅ | ✅ |

### 其他特性

- **勾选处理**：输入列表支持勾选，仅处理已选文件
- **预览**：缩略图 + 点击预览（图片原图 / 视频可播放）
- **输出命名**：设置页可配置原名 / 时间戳 / 标准时间 / 自定义文本（比例裁剪除外）
- **局域网上传**：启动后提供上传链接，可将文件传到输入目录
- **自动更新**：启动检测新版本，确认后下载并安装
- **系统托盘**：关闭窗口可隐藏到托盘，不直接退出

---

## 工作模式说明

### 图片 / 视频（批量）

1. 选择对应模式，列表按类型过滤  
2. 勾选要处理的文件，在选项条中配置参数  
3. 点击 **开始处理**，底部状态栏显示进度  
4. 结果写入输出目录  

### 比例裁剪

- 可同时处理图与视频  
- 支持添加多个比例（如 `9:16`、`1:1`）  
- **命名使用专用规则**（不受设置页通用命名影响）：

```
原名-比例(比值)-(范围枚举)-(日期).扩展名
```

示例：`促销活动A-1-1(1.00)-(0.94～1.05)-(260322).mp4`

### 自定义裁剪

- 每次处理 **1 个** 文件（图或视频）  
- 在预览区拖动 / 缩放裁剪框，可精确输入像素坐标  
- 视频以首帧预览；导出走通用命名规则  

### 拼接

固定 **2 路** 同类素材（双视频 **或** 双图片，不混排）。

| 项 | 说明 |
|----|------|
| 布局 | 上下拼接 / 左右拼接 |
| 槽位比例 | 9:16、16:9、1:1、4:3、3:4、原比例 |
| 填充 | 素材 **fill** 拉伸填满槽位 |
| 输出尺寸 | 可选手动指定最终宽高 |
| 帧率（仅视频） | **30** 或 **60**；选 60 且源不足时 **补帧**（重复帧） |
| Profile/Level（仅视频） | 可选；Profile：Baseline / Main / High；Level：3.0～5.2 |
| 输出 | 图片 → `png`；视频 → `mp4`（无音轨，`libx264` + `yuv420p`） |

### 自定义拼接

自由画布多图层合成，适合复杂排版。

| 项 | 说明 |
|----|------|
| 图层数 | 最多 **6** 层 |
| 素材 | 图片与视频 **可混排** |
| 输出类型 | 全图 → `png`；**含任意视频** → `mp4` |
| 画布 | 预设（9:16、16:9、1:1、4:5 等）或自定义宽高 |
| 背景 | 黑 / 白；纯图输出时可透明 |
| 图层操作 | 拖拽移动、边角缩放、层级上移/下移、填充模式（覆盖 / 包含 / 拉伸） |
| 吸附 | 可开关；吸附 **画布边缘** 与 **其它素材边/中线**，并显示辅助线 |
| 帧率（视频输出） | **30** 或 **60**；不足目标帧率时补帧 |
| Profile/Level（视频输出） | 可选，同上 |
| 时长 | 以最短视频轨为准（`shortest`）；静图 loop 铺满 |

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

1. 选择工作模式（图片 / 视频 / 比例裁剪 / 自定义裁剪 / 拼接 / 自定义拼接）  
2. 批量模式：勾选文件 → 配置选项 → **开始处理**  
3. 工作台模式：按界面提示选材、调整参数 → **导出**  
4. 结果写入输出目录；底部状态栏显示状态与局域网上传链接  

### 输出命名（设置页）

| 规则 | 说明 |
|------|------|
| 输出目录 | 设置中配置 |
| 通用命名 | 可选拼接：①原名 ②时间戳 ③标准时间 ④自定义文本 |
| 生效范围 | **除比例裁剪外** 的全部模式 |
| 比例裁剪 | 专用命名（见上文） |

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | [Tauri 2](https://tauri.app/) |
| 前端 | Vue 3、Vue Router、TypeScript、Vite、UnoCSS |
| 后端 | Rust |
| 媒体处理 | FFmpeg / FFprobe（视频）、ImageMagick（图片） |
| 更新 | `@tauri-apps/plugin-updater` + GitHub Releases |

### 关键命令（后端）

| 命令 | 用途 |
|------|------|
| `process_files` | 批量图片 / 视频处理 |
| `crop_by_ratios` / `crop_videos_by_ratios` | 比例裁剪 |
| `custom_crop` | 自定义像素裁剪 |
| `merge_videos` | 双路拼接 |
| `join_media` | 自定义拼接（自由画布） |
| `get_video_dimensions` / `get_image_dimensions` | 读取尺寸 |
| `extract_video_frame` / `load_image_preview` / `get_file_thumbnail` | 预览与缩略图 |
| `start_upload_server` | 局域网上传 |

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
│   │   ├── MergeWorkspace.vue    # 双路拼接
│   │   ├── JoinWorkspace.vue     # 自定义拼接
│   │   ├── CropWorkspace.vue     # 自定义裁剪
│   │   └── ...
│   ├── composables/          # 业务逻辑 hooks
│   ├── views/                # 页面（首页 / 设置）
│   └── types.ts
├── src-tauri/                # 桌面后端 (Rust)
│   ├── src/
│   │   ├── commands.rs       # Tauri 命令（含 merge / join）
│   │   ├── process.rs        # 图片 / 视频批量处理
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

产品信息架构与约束见：

- [docs/Ra.md](./docs/Ra.md)

---

## 路线图 / 后续

- [x] 拼接自由画布布局（自定义拼接）
- [x] 自定义拼接：图视频混排、边缘/素材吸附、帧率 30/60 与 Level
- [ ] 更多图片批量能力与预设
- [ ] Linux 构建与 sidecar 支持
- [ ] 拼接：光流插帧（更高质量升帧）、音频策略

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
