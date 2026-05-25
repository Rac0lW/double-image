# double-image

A Rust CLI tool that horizontally doubles an image by stitching two identical copies side by side. Also supports AI-powered anatomical cutout mode for figure drawing practice.

## Features

- `400x300` → `800x300`
- Supports PNG, JPG, JPEG, GIF, BMP, WebP, TIFF
- Output filename automatically gets a `_double` suffix
- Single-file and batch processing modes
- **Image-Cutout mode (`--ic`)**: Uses AI to create strategic anatomical cutouts on the right half, left half preserved as reference

## Installation

### Build from source

```bash
git clone <repo-url>
cd double-image
cargo build --release
```

The compiled binary will be at `target/release/double-image`.

### Install to system PATH (optional)

```bash
cargo install --path .
```

Or install from crates.io (if published):

```bash
cargo install double-image
```

## Usage

### Process a single file

```bash
double-image photo.png
# Output: photo_double.png
```

### Process multiple files

```bash
double-image img1.jpg img2.png img3.bmp
```

### Batch process current directory

When no files are specified, the tool scans the current directory for images and asks for confirmation before processing:

```bash
cd ~/Pictures

$ double-image
Found 5 image files, process all? [y/N] y
✓ vacation.jpg → vacation_double.jpg (1920x1080 → 3840x1080)
✓ screenshot.png → screenshot_double.png (800x600 → 1600x600)
...
Done: 5 succeeded, 0 failed
```

Type `y` or `yes` to confirm, any other key to cancel.

### Image-Cutout mode (`--ic`)

Enable the `--ic` flag to use AI-powered anatomical cutout mode via [PackyAPI](https://www.packyapi.com) GPT-Image-2:

- **Left half**: Original image preserved as reference for comparison
- **Right half**: AI creates strategic circular cutouts at key anatomical landmarks (joints, facial features, muscle attachments, etc.) revealing a light gray background
- Designed for figure drawing / anatomy practice

**Prerequisite**: Set the `PACKY_API_KEY` environment variable (Sora group token)

```bash
export PACKY_API_KEY="your-sora-group-token"

double-image --ic pose-reference.jpg
# Output: pose-reference_ic.jpg
# Left = original, Right = structural cutout version
```

Batch processing is also supported:

```bash
double-image --ic pose1.jpg pose2.png pose3.bmp
```

## Output Location

Output files are placed in the **same directory** as the input files.

| Input | Output |
|-------|--------|
| `/tmp/photo.png` | `/tmp/photo_double.png` |
| `./img/cat.jpg` | `./img/cat_double.jpg` |

In `--ic` mode, the suffix is `_ic` instead of `_double`.

| Input | `--ic` Output |
|-------|---------------|
| `/tmp/photo.png` | `/tmp/photo_ic.png` |

## Supported Formats

- PNG
- JPEG / JPG
- GIF
- BMP
- WebP
- TIFF

## Dependencies

- [image](https://crates.io/crates/image) — Rust image processing library
- [reqwest](https://crates.io/crates/reqwest) — HTTP client (required for image-cutout mode)
- [serde_json](https://crates.io/crates/serde_json) — JSON parsing (required for image-cutout mode)

## Examples

```bash
# Single file
double-image wallpaper.png
# → wallpaper_double.png (2560x1440 → 5120x1440)

# Batch processing (requires confirmation)
cd ~/Downloads/images
double-image
# Found 12 image files, process all? [y/N] y
# → Generates *_double.* files for all images

# image-cutout mode (requires PACKY_API_KEY)
export PACKY_API_KEY="your-sora-group-token"
double-image --ic anatomy-reference.jpg
# → anatomy-reference_ic.jpg
# Left = original reference, Right = structural cutout practice
```
