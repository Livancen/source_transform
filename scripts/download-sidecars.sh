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

# ---------- macOS ----------
prepare_macos() {
  local triple="$1"
  local arch_npm
  if [[ "$triple" == "aarch64-apple-darwin" ]]; then
    arch_npm="arm64"
  else
    arch_npm="x64"
  fi

  local ffmpeg_out="$BIN_DIR/ffmpeg-$triple"
  local ffprobe_out="$BIN_DIR/ffprobe-$triple"
  local magick_out="$BIN_DIR/magick-$triple"

  if [[ -f "$ffmpeg_out" && -f "$ffprobe_out" && -f "$magick_out" ]]; then
    echo "    macOS sidecars already present for $triple"
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

  if [[ ! -f "$magick_out" ]]; then
    local host_arch
    host_arch="$(uname -m)"
    if ! command -v brew >/dev/null 2>&1; then
      echo "Homebrew required to install ImageMagick on macOS CI" >&2
      exit 1
    fi
    if { [[ "$arch_npm" == "arm64" && "$host_arch" == "arm64" ]] || \
         [[ "$arch_npm" == "x64" && "$host_arch" == "x86_64" ]]; }; then
      brew list imagemagick >/dev/null 2>&1 || brew install imagemagick
      local magick_src
      magick_src="$(command -v magick || true)"
      if [[ -z "$magick_src" ]]; then
        magick_src="$(brew --prefix imagemagick)/bin/magick"
      fi
      cp "$magick_src" "$magick_out"
      chmod +x "$magick_out"
      echo "    ok: magick (brew, native)"
    else
      # 交叉：尝试复制 brew 的 magick 并提示（arm runner 编 x64 时 magick 可能是 arm）
      # 更稳：x64 job 使用 brew 的 bottle 不一定匹配；建议用户只发 arm64，或提交预编译 magick
      echo "WARN: building $triple on host $host_arch — magick may be wrong arch"
      brew list imagemagick >/dev/null 2>&1 || brew install imagemagick
      cp "$(command -v magick)" "$magick_out"
      chmod +x "$magick_out"
      # 用 file 检查架构
      if command -v file >/dev/null 2>&1; then
        file "$magick_out" || true
      fi
      echo "    ok: magick (brew, possible arch mismatch)"
    fi
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
