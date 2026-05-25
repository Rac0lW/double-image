use std::env;
use std::fs;
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use image::{DynamicImage, GenericImage, ImageFormat, RgbaImage};

const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff"];
const PACKY_API_URL: &str = "https://www.packyapi.com/v1/images/edits";

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
    io::stdin().read_line(&mut input).expect("读取输入失败");

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn double_image_horizontal(input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let img = image::open(input_path)?;
    let (width, height) = (img.width(), img.height());

    let mut output_img = RgbaImage::new(width * 2, height);
    output_img.copy_from(&img.to_rgba8(), 0, 0)?;
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

fn format_error_chain(e: &dyn std::error::Error) -> String {
    let mut msg = e.to_string();
    let mut source = e.source();
    while let Some(s) = source {
        msg.push_str(&format!("\n  → {}", s));
        source = s.source();
    }
    msg
}

fn process_image_cutout(input_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let api_key = env::var("PACKY_API_KEY")
        .map_err(|_| "请设置环境变量 PACKY_API_KEY（Sora 分组令牌）")?;

    let img = image::open(input_path)?;
    let (width, height) = (img.width(), img.height());

    println!("  正在调用 PackyAPI GPT-Image-2 进行结构挖孔，请耐心等待...");

    let mut img_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut img_bytes), ImageFormat::Png)?;
    println!("  图片已编码为 PNG，大小: {} bytes", img_bytes.len());

    println!("  构建 HTTP 客户端...");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", format_error_chain(&e)))?;

    let prompt = concat!(
        "分析图片中的人物形象，在其关键解剖结构点和结构 landmark 处",
        "（如关节、面部特征、肌肉附着点、身体比例关键点、肢体转折点等）",
        "挖出圆形孔洞，露出干净的浅灰色背景。",
        "孔洞大小各异（直径约为画面中人物高度的 3% 到 12%），",
        "分布应覆盖全身多个关键结构位置但不过于密集，",
        "其余部分保持完全完整。用于人体结构绘画练习。"
    );

    let form = reqwest::blocking::multipart::Form::new()
        .text("model", "gpt-image-2")
        .text("prompt", prompt)
        .text("size", "auto")
        .text("quality", "high")
        .text("response_format", "b64_json")
        .text("output_format", "png")
        .text("input_fidelity", "high")
        .part(
            "image",
            reqwest::blocking::multipart::Part::bytes(img_bytes)
                .file_name("input.png")
                .mime_str("image/png")?,
        );

    println!("  发送请求到 {} ...", PACKY_API_URL);
    let response = client
        .post(PACKY_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .map_err(|e| format!("发送请求失败: {}", format_error_chain(&e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(format!("API 请求失败: {} - {}", status, text).into());
    }

    let json: serde_json::Value = response.json()?;

    let revised = json["data"][0]["revised_prompt"].as_str().unwrap_or("");
    if !revised.is_empty() {
        println!("  模型优化后的提示词: {}", revised);
    }

    println!("  正在解析 API 返回的图片数据...");
    let b64 = json["data"][0]["b64_json"]
        .as_str()
        .ok_or("API 返回格式错误：缺少 b64_json 字段")?;

    let edited_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        b64,
    ).map_err(|e| format!("Base64 解码失败: {}", e))?;
    println!("  图片数据解码完成: {} bytes", edited_bytes.len());

    let sig_len = edited_bytes.len().min(16);
    println!("  文件签名 (hex): {}",
        edited_bytes[..sig_len].iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "));

    let edited_img = image::load_from_memory_with_format(&edited_bytes, ImageFormat::Png)
        .or_else(|e| {
            println!("  内存直接解码失败 ({}), 尝试写入临时文件...", e);
            let tmp = std::env::temp_dir().join("double_image_ic_tmp.png");
            std::fs::write(&tmp, &edited_bytes)
                .map_err(|ioe| format!("写入临时文件失败: {}", ioe))?;
            let img = image::open(&tmp)
                .map_err(|e| format!("从临时文件解码失败: {}", e))?;
            let _ = std::fs::remove_file(&tmp);
            Ok::<DynamicImage, Box<dyn std::error::Error>>(img)
        })?;

    let mut output_img = RgbaImage::new(width * 2, height);
    output_img.copy_from(&img.to_rgba8(), 0, 0)?;
    output_img.copy_from(&edited_img.to_rgba8(), width, 0)?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");

    let output_name = format!("{}_ic.{}", stem, ext);
    let output_path = input_path.with_file_name(&output_name);

    let format = ImageFormat::from_path(input_path).unwrap_or(ImageFormat::Png);
    DynamicImage::ImageRgba8(output_img).save_with_format(&output_path, format)?;

    println!(
        "✓ {} → {} ({}x{} → {}x{}, image-cutout 模式)",
        input_path.display(),
        output_path.display(),
        width,
        height,
        width * 2,
        height
    );

    Ok(output_path)
}

fn print_usage() {
    println!("用法:");
    println!("  double-image [选项] <图片文件> [更多图片文件...]");
    println!();
    println!("选项:");
    println!("  --ic    启用 image-cutout 模式：右边图片进行结构挖孔，左边保留原图作对比参考");
    println!();
    println!("环境变量:");
    println!("  PACKY_API_KEY    image-cutout 模式必需的 API 令牌（Sora 分组）");
    println!();
    println!("示例:");
    println!("  double-image photo.png                  # 默认 double 模式");
    println!("  double-image --ic photo.png             # image-cutout 模式");
    println!("  double-image --ic img1.jpg img2.png     # 批量 IC 处理");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return;
    }

    let ic_mode = args.iter().any(|a| a == "--ic");
    let filtered_args: Vec<String> = args.into_iter().filter(|a| a != "--ic").collect();

    let input_paths: Vec<PathBuf> = if filtered_args.len() > 1 {
        filtered_args[1..].iter().map(PathBuf::from).collect()
    } else {
        let current_dir = env::current_dir().expect("无法获取当前目录");
        let image_files = get_image_files_in_dir(&current_dir);

        if image_files.is_empty() {
            println!("当前目录没有找到图片文件。");
            print_usage();
            return;
        }

        if !ask_user_confirmation(image_files.len()) {
            println!("已取消操作。");
            return;
        }

        image_files
    };

    if ic_mode {
        if env::var("PACKY_API_KEY").is_err() {
            eprintln!("错误: image-cutout 模式需要设置 PACKY_API_KEY 环境变量");
            eprintln!("      export PACKY_API_KEY=\"你的 Sora 分组令牌\"");
            return;
        }
        println!("已启用 image-cutout 模式，将调用 PackyAPI GPT-Image-2 进行结构挖孔...\n");

        if let Ok(proxy) = env::var("HTTPS_PROXY").or_else(|_| env::var("https_proxy")) {
            println!("[诊断] 检测到 HTTPS_PROXY: {}", proxy);
        }
        if let Ok(proxy) = env::var("HTTP_PROXY").or_else(|_| env::var("http_proxy")) {
            println!("[诊断] 检测到 HTTP_PROXY: {}", proxy);
        }
        if let Ok(no_proxy) = env::var("NO_PROXY").or_else(|_| env::var("no_proxy")) {
            println!("[诊断] 检测到 NO_PROXY: {}", no_proxy);
        }
    }

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

        let result = if ic_mode {
            process_image_cutout(path)
        } else {
            double_image_horizontal(path)
        };

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                eprintln!("✗ 处理失败 {}:\n{}", path.display(), format_error_chain(&*e));
                fail_count += 1;
            }
        }
    }

    println!();
    println!("处理完成: {} 成功, {} 失败", success_count, fail_count);
}
