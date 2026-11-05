#!/usr/bin/env bash
# 按目标平台下载/准备 ffmpeg、ffprobe、magick sidecar
# 用法:
#   bash scripts/download-sidecars.sh [target-triple]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$ROOT/src-tauri/binaries"
mkdir -p "$BIN_DIR"
TMP_DIR="${RUNNER_TEMP:-${TEMP:-/tmp}}/source_transform_sidecars_$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

detect_target() {
  if [[ -n "${1:-}" ]]; then
    echo "$1"
    return
  fi
  case "$(uname -s 2>/dev/null || echo Windows)" in
    MINGW*|MSYS*|CYGWIN*|Windows_NT|Windows)
      echo "x86_64-pc-windows-msvc"
      ;;
    Darwin)
      if [[ "$(uname -m)" == "arm64" ]]; then
        echo "aarch64-apple-darwin"
      else
        echo "x86_64-apple-darwin"
      fi
      ;;
    Linux)
      if [[ "$(uname -m)" == "aarch64" ]]; then
        echo "aarch64-unknown-linux-gnu"
      else
        echo "x86_64-unknown-linux-gnu"
      fi
      ;;
    *)
      echo "unsupported-os" >&2
      exit 1
      ;;
  esac
}

TARGET="$(detect_target "${1:-}")"
echo "==> Preparing sidecars for target: $TARGET"

download_ok() {
  local url="$1"
  local out="$2"
  echo "    try: $url"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 --retry-delay 2 -o "$out" "$url" && return 0
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$out" "$url" && return 0
  fi
  return 1
}

download_first() {
  local out="$1"
  shift
  local url
  for url in "$@"; do
    if download_ok "$url" "$out"; then
      return 0
    fi
  done
  echo "Failed to download: $out" >&2
  return 1
}

# ---------- Windows ----------
prepare_windows() {
  local suffix="x86_64-pc-windows-msvc.exe"
  local ffmpeg_out="$BIN_DIR/ffmpeg-$suffix"
  local ffprobe_out="$BIN_DIR/ffprobe-$suffix"
  local magick_out="$BIN_DIR/magick-$suffix"

  if [[ -f "$ffmpeg_out" && -f "$ffprobe_out" && -f "$magick_out" ]]; then
    echo "    Windows sidecars already present, skip download"
    ls -la "$BIN_DIR"/*windows* 2>/dev/null || true
    return
  fi

  command -v unzip >/dev/null 2>&1 || {
    echo "unzip is required" >&2
    exit 1
  }

  if [[ ! -f "$ffmpeg_out" || ! -f "$ffprobe_out" ]]; then
    local ffmpeg_zip="$TMP_DIR/ffmpeg-win64.zip"
    download_first "$ffmpeg_zip" \
      "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
    unzip -qo "$ffmpeg_zip" -d "$TMP_DIR/ffmpeg"
    local ffdir
    ffdir="$(find "$TMP_DIR/ffmpeg" -type d -name "bin" | head -n1)"
    cp "$ffdir/ffmpeg.exe" "$ffmpeg_out"
    cp "$ffdir/ffprobe.exe" "$ffprobe_out"
    echo "    ok: ffmpeg / ffprobe"
  fi

  if [[ ! -f "$magick_out" ]]; then
    local im_zip="$TMP_DIR/imagemagick.zip"
    # 多个版本/镜像，命中一个即可
    download_first "$im_zip" \
      "https://imagemagick.org/archive/binaries/ImageMagick-7.1.1-47-portable-Q16-HDRI-x64.zip" \
      "https://imagemagick.org/archive/binaries/ImageMagick-7.1.1-43-portable-Q16-HDRI-x64.zip" \
      "https://download.imagemagick.org/ImageMagick/download/binaries/ImageMagick-7.1.1-43-portable-Q16-HDRI-x64.zip" \
      || {
        echo "ImageMagick portable download failed; place magick-x86_64-pc-windows-msvc.exe manually" >&2
        exit 1
      }
    unzip -qo "$im_zip" -d "$TMP_DIR/magick"
    local magick_bin
    magick_bin="$(find "$TMP_DIR/magick" -type f -iname "magick.exe" | head -n1)"
    if [[ -z "$magick_bin" ]]; then
      echo "magick.exe not found in ImageMagick archive" >&2
      exit 1
    fi
    local magick_dir
    magick_dir="$(dirname "$magick_bin")"
    cp "$magick_bin" "$magick_out"
    # portable 依赖同目录 dll
    find "$magick_dir" -maxdepth 1 -type f \( -iname "*.dll" \) -exec cp {} "$BIN_DIR/" \;
    echo "    ok: magick (+ dlls)"
  fi
}

# 将 brew magick 及其非系统 dylib 打成自包含包，供 Tauri sidecar 使用。
# 兼容 macOS 自带 Bash 3.2（不用关联数组 / process substitution）。
# 产物：
#   binaries/magick-<triple>
#   binaries/magick-libs/*.dylib  （打包进 Contents/Resources/magick-libs/）
bundle_magick_macos() {
  local magick_src="$1"
  local magick_out="$2"
  local libs_dir="$BIN_DIR/magick-libs"
  local rel_prefix="@executable_path/../Resources/magick-libs"
  local seen_file="$TMP_DIR/magick_seen_deps.txt"
  local queue_file="$TMP_DIR/magick_dep_queue.txt"

  command -v otool >/dev/null 2>&1 || {
    echo "otool is required to bundle ImageMagick" >&2
    exit 1
  }
  command -v install_name_tool >/dev/null 2>&1 || {
    echo "install_name_tool is required to bundle ImageMagick" >&2
    exit 1
  }

  rm -rf "$libs_dir"
  mkdir -p "$libs_dir"
  : > "$seen_file"
  : > "$queue_file"
  cp "$magick_src" "$magick_out"
  chmod +x "$magick_out"

  enqueue_dep() {
    local lib="$1"
    # 系统库与已带 install path 的引用不收集
    case "$lib" in
      /System/*|/usr/lib/*|/usr/X11*|@*) return 0 ;;
    esac
    # 其余（Homebrew / Cellar / 官方包 lib/ 等）全部收集
    [[ -f "$lib" ]] || return 0
    if grep -Fxq "$lib" "$seen_file" 2>/dev/null; then
      return 0
    fi
    echo "$lib" >> "$seen_file"
    echo "$lib" >> "$queue_file"
  }

  collect_deps_from() {
    local bin="$1"
    local line lib
    otool -L "$bin" | tail -n +2 | while IFS= read -r line; do
      lib="$(echo "$line" | awk '{print $1}')"
      enqueue_dep "$lib"
    done
  }

  # 注意：管道子 shell 无法改父级文件以外的状态；enqueue 写文件即可
  collect_deps_from "$magick_out"

  local qpos=1
  local dep base
  while true; do
    dep="$(sed -n "${qpos}p" "$queue_file")"
    [[ -n "$dep" ]] || break
    qpos=$((qpos + 1))
    base="$(basename "$dep")"
    if [[ ! -f "$libs_dir/$base" ]]; then
      cp "$dep" "$libs_dir/$base"
      chmod +w "$libs_dir/$base" || true
    fi
    collect_deps_from "$dep"
  done

  rewrite_deps() {
    local file="$1"
    local id_path="$2"
    local line lib base
    chmod +w "$file" || true
    install_name_tool -id "$id_path" "$file" 2>/dev/null || true
    otool -L "$file" | tail -n +2 | while IFS= read -r line; do
      lib="$(echo "$line" | awk '{print $1}')"
      case "$lib" in
        /System/*|/usr/lib/*) continue ;;
      esac
      base="$(basename "$lib")"
      if [[ -f "$libs_dir/$base" ]]; then
        if [[ "$file" == "$magick_out" ]]; then
          install_name_tool -change "$lib" "$rel_prefix/$base" "$file" || true
        else
          install_name_tool -change "$lib" "@loader_path/$base" "$file" || true
        fi
      fi
    done
  }

  local lib_file
  for lib_file in "$libs_dir"/*.dylib; do
    [[ -f "$lib_file" ]] || continue
    rewrite_deps "$lib_file" "@loader_path/$(basename "$lib_file")"
  done
  rewrite_deps "$magick_out" "$(basename "$magick_out")"

  # install_name_tool 会使原签名失效，改为 ad-hoc，最终由 Tauri 再签 app
  if command -v codesign >/dev/null 2>&1; then
    codesign --force --sign - "$magick_out" >/dev/null 2>&1 || true
    for lib_file in "$libs_dir"/*.dylib; do
      [[ -f "$lib_file" ]] || continue
      codesign --force --sign - "$lib_file" >/dev/null 2>&1 || true
    done
  fi

  # 校验：magick 不应再引用 Cellar / Homebrew 绝对路径
  if otool -L "$magick_out" | grep -E '/opt/homebrew/|/usr/local/Cellar/|/Homebrew/' >/dev/null 2>&1; then
    echo "magick still has Homebrew absolute dylib paths:" >&2
    otool -L "$magick_out" >&2
    exit 1
  fi

  local lib_count
  lib_count="$(find "$libs_dir" -name '*.dylib' | wc -l | tr -d ' ')"
  echo "    ok: magick self-contained ($lib_count dylibs -> magick-libs/)"
}

magick_is_self_contained() {
  local magick_bin="$1"
  local libs_dir="$BIN_DIR/magick-libs"
  [[ -f "$magick_bin" ]] || return 1
  [[ -d "$libs_dir" ]] || return 1
  find "$libs_dir" -name '*.dylib' | grep -q . || return 1
  if otool -L "$magick_bin" 2>/dev/null | grep -E '/opt/homebrew/|/usr/local/Cellar/|/Homebrew/' >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

resolve_brew_magick() {
  local host_arch expected_arch brew_bin magick_src
  host_arch="$(uname -m)"
  expected_arch="$1" # arm64 | x86_64

  if ! command -v brew >/dev/null 2>&1; then
    echo "Homebrew required to install ImageMagick on macOS" >&2
    exit 1
  fi

  if [[ "$host_arch" != "$expected_arch" ]]; then
    if [[ "$expected_arch" == "x86_64" && "$host_arch" == "arm64" ]]; then
      echo "    host is arm64, trying Rosetta x86_64 Homebrew for magick..." >&2
      if [[ -x /usr/local/bin/brew ]]; then
        brew_bin=/usr/local/bin/brew
      else
        echo "x86_64 Homebrew not found at /usr/local/bin/brew (need Rosetta brew for Intel magick)" >&2
        exit 1
      fi
      arch -x86_64 "$brew_bin" list imagemagick >/dev/null 2>&1 || \
        arch -x86_64 "$brew_bin" install imagemagick
      magick_src="$(arch -x86_64 "$brew_bin" --prefix imagemagick)/bin/magick"
    else
      echo "Cannot prepare magick for $expected_arch on host $host_arch" >&2
      exit 1
    fi
  else
    brew list imagemagick >/dev/null 2>&1 || brew install imagemagick
    magick_src="$(command -v magick || true)"
    if [[ -z "$magick_src" ]]; then
      magick_src="$(brew --prefix imagemagick)/bin/magick"
    fi
  fi

  if [[ ! -f "$magick_src" ]]; then
    echo "magick binary not found: $magick_src" >&2
    exit 1
  fi

  if command -v file >/dev/null 2>&1; then
    if ! file "$magick_src" | grep -q "$expected_arch"; then
      echo "magick arch mismatch, expected $expected_arch:" >&2
      file "$magick_src" >&2
      exit 1
    fi
  fi

  # 仅 stdout 输出路径，供命令替换捕获
  printf '%s\n' "$magick_src"
}

# ---------- macOS ----------
prepare_macos() {
  local triple="$1"
  local arch_npm expected_arch
  if [[ "$triple" == "aarch64-apple-darwin" ]]; then
    arch_npm="arm64"
    expected_arch="arm64"
  else
    arch_npm="x64"
    expected_arch="x86_64"
  fi

  local ffmpeg_out="$BIN_DIR/ffmpeg-$triple"
  local ffprobe_out="$BIN_DIR/ffprobe-$triple"
  local magick_out="$BIN_DIR/magick-$triple"

  if [[ -f "$ffmpeg_out" && -f "$ffprobe_out" ]] && magick_is_self_contained "$magick_out"; then
    echo "    macOS sidecars already present for $triple (magick self-contained)"
    chmod +x "$ffmpeg_out" "$ffprobe_out" "$magick_out" || true
    return
  fi

  if [[ ! -f "$ffmpeg_out" ]]; then
    download_first "$ffmpeg_out" \
      "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-darwin-${arch_npm}" \
      "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.0/ffmpeg-darwin-${arch_npm}" \
      "https://cdn.jsdelivr.net/npm/@ffmpeg-installer/darwin-${arch_npm}@4.1.5/ffmpeg" \
      "https://cdn.jsdelivr.net/npm/@ffmpeg-installer/darwin-${arch_npm}@4.1.0/ffmpeg"
    chmod +x "$ffmpeg_out"
    echo "    ok: ffmpeg"
  fi

  if [[ ! -f "$ffprobe_out" ]]; then
    if ! download_first "$ffprobe_out" \
      "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffprobe-darwin-${arch_npm}" \
      "https://cdn.jsdelivr.net/npm/@ffprobe-installer/darwin-${arch_npm}@5.1.0/ffprobe" \
      "https://cdn.jsdelivr.net/npm/@ffprobe-installer/darwin-${arch_npm}@4.1.0/ffprobe"; then
      # brew 回退（仅 host arch 匹配）
      if command -v brew >/dev/null 2>&1; then
        local host_arch
        host_arch="$(uname -m)"
        if { [[ "$arch_npm" == "arm64" && "$host_arch" == "arm64" ]] || \
             [[ "$arch_npm" == "x64" && "$host_arch" == "x86_64" ]]; }; then
          brew list ffmpeg >/dev/null 2>&1 || brew install ffmpeg
          cp "$(brew --prefix ffmpeg)/bin/ffprobe" "$ffprobe_out"
        else
          echo "Cannot get ffprobe for $triple (cross-arch)" >&2
          exit 1
        fi
      else
        echo "Cannot download ffprobe for $triple" >&2
        exit 1
      fi
    fi
    chmod +x "$ffprobe_out"
    echo "    ok: ffprobe"
  fi

  if ! magick_is_self_contained "$magick_out"; then
    local magick_src
    magick_src="$(resolve_brew_magick "$expected_arch")"
    bundle_magick_macos "$magick_src" "$magick_out"
  fi
}

case "$TARGET" in
  x86_64-pc-windows-msvc)
    prepare_windows
    ;;
  aarch64-apple-darwin|x86_64-apple-darwin)
    prepare_macos "$TARGET"
    ;;
  *)
    echo "Unsupported target: $TARGET" >&2
    exit 1
    ;;
esac

echo "==> Sidecars ready in $BIN_DIR"
ls -la "$BIN_DIR"
