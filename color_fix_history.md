# 图片压缩色彩失真修复历史

## 问题描述
压缩图片时出现色彩失真，红色图片压缩后变成紫色。

## 原始图片信息
```json
{
  "pix_fmt": "gbrap",
  "color_space": "bt470bg",
  "width": 2480,
  "height": 3508
}
```

## 根本原因
原始图片使用 `gbrap` 像素格式（GBR 通道顺序 + Alpha）和 `bt470bg` 色彩空间（PAL 制式），FFmpeg 在处理时色彩转换出现问题。

---

## 修改版本记录

### 版本 1：初始修复尝试
**修改内容**：根据文件格式设置不同的编码参数
- JPEG: `-c:v mjpeg -pix_fmt yuvj444p -color_range pc`
- PNG: `-pix_fmt rgb24`
- 其他格式特殊处理

**结果**：红色变紫色

---

### 版本 2：方案1 - 改用 yuvj420p
**修改内容**：将 `yuvj444p` 改为 `yuvj420p`
```
-pix_fmt yuvj420p
```

**结果**：还是变紫色

---

### 版本 3：方案2 - 移除 pix_fmt 参数
**修改内容**：移除 `-pix_fmt` 参数，让 FFmpeg 自动选择

**结果**：还是变紫色

---

### 版本 4：方案3 - format=rgb24 滤镜 + pix_fmt
**修改内容**：
```
-vf "format=rgb24" -pix_fmt yuvj420p
```

**结果**：还是变紫色

---

### 版本 5：方案E - 回退到最简配置
**修改内容**：移除所有色彩相关参数，只保留：
```
ffmpeg -i input.jpg [-vf 旋转/缩放] [-q:v 质量] -y output.jpg
```

**结果**：还是变紫色（说明问题不是我们添加的参数导致的，是 FFmpeg 本身的问题）

---

### 版本 6：方案B - colormatrix 滤镜
**修改内容**：
```
-vf "colormatrix=bt601:bt709"
```

**结果**：还是有偏色

---

### 版本 7：方案A - colorspace 滤镜
**修改内容**：
```
-vf "colorspace=all=bt709:iall=bt601"
```

**结果**：还是有偏色

---

### 版本 8：改回方案B
**修改内容**：改回 `colormatrix=bt601:bt709`

**结果**：还是有偏色

---

### 版本 9：-pix_fmt rgb24 输出参数
**修改内容**：
```
-pix_fmt rgb24
```

**结果**：还是变紫色

---

### 版本 10：-pix_fmt yuv444p
**修改内容**：
```
-pix_fmt yuv444p
```

**结果**：还是变紫色

---

### 版本 11：format=rgba 滤镜
**修改内容**：
```
-vf "format=rgba"
```

**目的**：修复 gbrap 格式的通道顺序问题

**结果**：还是变紫色

---

### 版本 12：shuffleplanes + format=rgb24
**修改内容**：
```
-vf "shuffleplanes=2:0:1:3,format=rgb24"
```

**目的**：手动重排 gbrap 的通道顺序 (G,B,R,A → R,G,B,A)

**结果**：图片变绿了（方向搞反了）

---

### 版本 13：只用 format=rgb24
**修改内容**：移除 shuffleplanes，只保留
```
-vf "format=rgb24"
```

**结果**：还是变紫色

---

### 版本 14：format=rgb24 + setparams
**修改内容**：
```
-vf "format=rgb24,setparams=colorspace=bt709:color_primaries=bt709:color_trc=bt709"
```

**目的**：重置色彩空间元数据为 bt709

**结果**：还是变紫色（setparams 只修改元数据，不实际转换）

---

### 版本 15：format=rgb24 + scale out_color_matrix
**修改内容**：
```
-vf "format=rgb24,scale=iw:ih:out_color_matrix=bt709"
```

**目的**：强制使用 bt709 色彩矩阵进行 RGB→YUV 转换

**结果**：还是变紫色

---

### 版本 16：scale in_color_matrix + out_color_matrix + colorspace 输出参数
**修改内容**：
```
-vf "scale=iw:ih:in_color_matrix=bt470bg:out_color_matrix=bt709" -colorspace bt709
```

**结果**：报错 - `bt470bg` 不是有效的 `in_color_matrix` 值

---

### 版本 17：修复版本16 - 使用 bt601 代替 bt470bg
**修改内容**：
```
-vf "scale=iw:ih:in_color_matrix=bt601:out_color_matrix=bt709" -colorspace bt709
```

**结果**：还是变紫色

---

### 版本 18：geq 滤镜 - 交换 R 和 B 通道
**修改内容**：
```
-vf "geq=r='b(X,Y)':g='g(X,Y)':b='r(X,Y)'"
```

**目的**：手动交换 R 和 B 通道

**结果**：偏紫蓝色

---

### 版本 19：geq 滤镜 - 重新映射通道 (r=b, g=r, b=g)
**修改内容**：
```
-vf "geq=r='b(X,Y)':g='r(X,Y)':b='g(X,Y)'"
```

**目的**：根据 gbrap 通道顺序重新映射

**结果**：图片变绿了

---

### 版本 20：geq 滤镜 - 排列 (r=g, g=b, b=r)
**修改内容**：
```
-vf "geq=r='g(X,Y)':g='b(X,Y)':b='r(X,Y)'"
```

**结果**：图片变蓝了

---

### 版本 21：geq 滤镜 - 排列 (r=g, g=r, b=b)
**修改内容**：
```
-vf "geq=r='g(X,Y)':g='r(X,Y)':b='b(X,Y)'"
```

**结果**：图片变绿了

---

### 版本 22：geq 滤镜 - 排列 (r=r, g=b, b=g)
**修改内容**：
```
-vf "geq=r='r(X,Y)':g='b(X,Y)':b='g(X,Y)'"
```

**结果**：偏黄色（最接近正确的结果）

---

### 版本 23：format=gbrp + format=rgb24
**修改内容**：
```
-vf "format=gbrp,format=rgb24"
```

**目的**：通过 gbrp 中间格式强制正确转换

**结果**：还是变紫色

---

### 版本 24：format=bgr24
**修改内容**：
```
-vf "format=bgr24"
```

**结果**：还是偏紫色

---

### 版本 25：format=rgb24 + colormatrix
**修改内容**：
```
-vf "format=rgb24,colormatrix=bt601:bt709"
```

**结果**：还是紫色，但亮度比上一版暗

---

### 版本 26：添加 sws_flags 参数
**修改内容**：
```
-sws_flags accurate_rnd+full_chroma_int
```

**目的**：使用更精确的色彩转换算法

**结果**：还是偏紫色

---

### 版本 27：format=yuv444p（当前版本）
**修改内容**：
```
-sws_flags accurate_rnd+full_chroma_int -vf "format=yuv444p"
```

**目的**：直接转换为 YUV 格式，跳过 RGB 中间步骤

**结果**：还是偏紫色

---

## 总计：27 个版本

## geq 滤镜通道排列测试结果汇总

| 排列 | 公式 | 结果 |
|------|------|------|
| #1 | r=r, g=g, b=b | 原始（紫色） |
| #2 | r=r, g=b, b=g | **偏黄**（最接近） |
| #3 | r=g, g=r, b=b | 变绿 |
| #4 | r=g, g=b, b=r | 变蓝 |
| #5 | r=b, g=r, b=g | 变绿 |
| #6 | r=b, g=g, b=r | 偏紫蓝 |

## 关键发现
1. 原始图片像素格式：`gbrap`（GBR 通道顺序，带 Alpha）
2. 原始图片色彩空间：`bt470bg`（PAL 制式）
3. 最简配置（版本5）也会变紫，说明问题在 FFmpeg 解码/编码过程中
4. geq 排列 `r=r, g=b, b=g` 得到偏黄结果，是最接近正确的
5. 问题核心：FFmpeg 在处理 `gbrap` + `bt470bg` 组合时存在色彩转换 bug

## 可能的解决方向
1. 更新 FFmpeg 版本，可能新版本已修复此 bug
2. 使用 ImageMagick 等其他工具处理这种特殊格式的图片
3. 在应用层检测 gbrap 格式，给用户警告或跳过处理
4. 尝试 FFmpeg 的 `-vf "lut"` 滤镜进行更精细的色彩调整
5. 使用两步处理：先转换为 PNG，再从 PNG 压缩为 JPEG

## 输出图片信息（压缩后）
```json
{
  "pix_fmt": "yuvj444p",
  "color_range": "pc",
  "color_space": "bt470bg"
}
```

---

## 版本 28：强制 yuvj420p 格式（基于其他工具分析）

### 背景
用户使用其他压缩工具压缩同一图片，结果没有偏色。对比 ffprobe 输出发现：

| 属性 | 问题图片 | 其他工具输出 |
|------|----------|--------------|
| pix_fmt | gbrap | **yuvj420p** |
| color_space | bt470bg | bt470bg |
| color_range | - | pc |

### 关键发现
**问题根源是 `gbrap` 像素格式，而非 `bt470bg` 色彩空间！**

- `gbrap` = GBR 通道顺序（Green-Blue-Red-Alpha），非标准
- `yuvj420p` = 标准 JPEG YUV 格式，full range

FFmpeg 处理 `gbrap` 时按 RGB 顺序解读，但实际是 GBR，导致 R 和 B 通道错位。

### 修改内容
```rust
// 滤镜：强制转换为标准 JPEG YUV 格式
filters.push("format=yuvj420p".to_string());

// 输出参数：强制像素格式
args.push("-pix_fmt".to_string());
args.push("yuvj420p".to_string());
```

完整 FFmpeg 参数：
```
-i input.jpg -sws_flags accurate_rnd+full_chroma_int -vf "format=yuvj420p" -pix_fmt yuvj420p -y output.jpg
```

### 预期结果
与其他压缩工具一致，输出标准 `yuvj420p` 格式，避免 gbrap 通道顺序问题。

**状态**：待测试
