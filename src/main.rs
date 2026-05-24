use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use image::{DynamicImage, GenericImage, ImageFormat, RgbaImage};

const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff"];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

fn get_image_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("无法读取目录: {}", dir.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && is_image_file(path))
        .collect()
}

fn ask_user_confirmation(count: usize) -> bool {
    print!("找到 {} 个图片文件，是否全部处理? [y/N] ", count);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("读取输入失败");

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn double_image_horizontal(input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let img = image::open(input_path)?;
    let (width, height) = (img.width(), img.height());

    let mut output_img = RgbaImage::new(width * 2, height);

    // 左边放原图
    output_img.copy_from(&img.to_rgba8(), 0, 0)?;
    // 右边放原图
    output_img.copy_from(&img.to_rgba8(), width, 0)?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");

    let output_name = format!("{}_double.{}", stem, ext);
    let output_path = input_path.with_file_name(&output_name);

    let format = ImageFormat::from_path(input_path).unwrap_or(ImageFormat::Png);
    DynamicImage::ImageRgba8(output_img).save_with_format(&output_path, format)?;

    println!(
        "✓ {} → {} ({}x{} → {}x{})",
        input_path.display(),
        output_path.display(),
        width,
        height,
        width * 2,
        height
    );

    Ok(output_path)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_paths: Vec<PathBuf> = if args.len() > 1 {
        // 指定了输入文件
        args[1..].iter().map(PathBuf::from).collect()
    } else {
        // 没有指定，扫描当前目录
        let current_dir = env::current_dir().expect("无法获取当前目录");
        let image_files = get_image_files_in_dir(&current_dir);

        if image_files.is_empty() {
            println!("当前目录没有找到图片文件。");
            println!("用法: double-image <图片文件> [更多图片文件...]");
            return;
        }

        if !ask_user_confirmation(image_files.len()) {
            println!("已取消操作。");
            return;
        }

        image_files
    };

    let mut success_count = 0;
    let mut fail_count = 0;

    for path in &input_paths {
        if !path.exists() {
            eprintln!("✗ 文件不存在: {}", path.display());
            fail_count += 1;
            continue;
        }

        if !is_image_file(path) {
            eprintln!("✗ 不支持的文件格式: {}", path.display());
            fail_count += 1;
            continue;
        }

        match double_image_horizontal(path) {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!("✗ 处理失败 {}: {}", path.display(), e);
                fail_count += 1;
            }
        }
    }

    println!();
    println!("处理完成: {} 成功, {} 失败", success_count, fail_count);
}
