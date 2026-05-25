# double-image

一个 Rust 命令行工具，将图片横向扩展一倍（左右拼接两张相同的图）。

## 功能

- 输入 `400x300` → 输出 `800x300`
- 支持 PNG、JPG、JPEG、GIF、BMP、WebP、TIFF
- 输出文件名自动添加 `_double` 后缀
- 支持单文件处理和批量处理
- **Image-Cutout 模式 (`--ic`)**：调用 AI 对右半图进行结构挖孔，左半图保留原样作为练习参考

## 安装

### 从源码编译

```bash
git clone <仓库地址>
cd double-image
cargo build --release
```

编译完成后，二进制文件位于 `target/release/double-image`。

### 安装到系统 PATH（可选）

```bash
cargo install --path .
```

```bash
cargo install double-image
```

## 使用

### 处理单个文件

```bash
double-image photo.png
# 输出: photo_double.png
```

### 处理多个文件

```bash
double-image img1.jpg img2.png img3.bmp
```

### 批量处理当前目录

不指定任何文件时，工具会扫描当前目录下的所有图片文件，并询问是否全部处理：

```bash
cd ~/Pictures

$ double-image
找到 5 个图片文件，是否全部处理? [y/N] y
✓ vacation.jpg → vacation_double.jpg (1920x1080 → 3840x1080)
✓ screenshot.png → screenshot_double.png (800x600 → 1600x600)
...
处理完成: 5 成功, 0 失败
```

输入 `y` 或 `yes` 确认，其他任意键取消。

### Image-Cutout 模式 (`--ic`)

通过 `--ic` 参数启用结构挖孔模式，调用 [PackyAPI](https://www.packyapi.com) 的 GPT-Image-2 模型对图片进行智能结构挖孔：

- **左半部分**：保留原图，作为补全后的对比参考
- **右半部分**：AI 在关键解剖结构点（关节、面部特征、肌肉附着点等）挖出圆形孔洞，露出浅灰色背景
- 用于人体结构绘画练习

**前置条件**：需要设置 `PACKY_API_KEY` 环境变量（Sora 分组令牌）

```bash
export PACKY_API_KEY="你的 Sora 分组令牌"

double-image --ic pose-reference.jpg
# 输出: pose-reference_ic.jpg
# 左半 = 原图，右半 = 结构挖孔版本
```

支持批量处理：

```bash
double-image --ic pose1.jpg pose2.png pose3.bmp
```

## 输出位置

输出文件与输入文件位于**同一目录**。

| 输入 | 输出 |
|------|------|
| `/tmp/photo.png` | `/tmp/photo_double.png` |
| `./img/cat.jpg` | `./img/cat_double.jpg` |

## 支持的格式

- PNG
- JPEG / JPG
- GIF
- BMP
- WebP
- TIFF

## 依赖

- [image](https://crates.io/crates/image) — Rust 图像处理库
- [reqwest](https://crates.io/crates/reqwest) — HTTP 客户端（image-cutout 模式需要）
- [serde_json](https://crates.io/crates/serde_json) — JSON 解析（image-cutout 模式需要）

## 示例

```bash
# 单文件
double-image wallpaper.png
# → wallpaper_double.png (2560x1440 → 5120x1440)

# 批量处理（需确认）
cd ~/Downloads/images
double-image
# 找到 12 个图片文件，是否全部处理? [y/N] y
# → 全部生成 *_double.* 文件

# image-cutout 模式（需要 PACKY_API_KEY）
export PACKY_API_KEY="你的 Sora 分组令牌"
double-image --ic anatomy-reference.jpg
# → anatomy-reference_ic.jpg
# 左半 = 原图参考，右半 = 结构挖孔练习
```
