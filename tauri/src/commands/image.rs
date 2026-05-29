use std::path::Path;
use std::process::Command;
use image::ImageReader;
use image::imageops::FilterType;
use uuid::Uuid;

const OUTPUT_DIR: &str = "/tmp/supertool-image";

fn ensure_output_dir() -> Result<(), String> {
    std::fs::create_dir_all(OUTPUT_DIR).map_err(|e| format!("创建输出目录失败: {}", e))
}

fn get_extension(path: &str) -> Result<String, String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| format!("无法获取文件扩展名: {}", path))
}

fn save_image(img: &image::DynamicImage, output_path: &str) -> Result<(), String> {
    let ext = get_extension(output_path)?;
    match ext.as_str() {
        "jpg" | "jpeg" => {
            let mut file =
                std::fs::File::create(output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            let rgb = img.to_rgb8();
            let (w, h) = rgb.dimensions();
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, 90);
            encoder
                .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG编码失败: {}", e))?;
        }
        "png" => {
            img.save(output_path)
                .map_err(|e| format!("保存PNG失败: {}", e))?;
        }
        "webp" => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let mut file =
                std::fs::File::create(output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            image::codecs::webp::WebPEncoder::new_lossless(&mut file)
                .encode(&rgba, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("WebP编码失败: {}", e))?;
        }
        "gif" => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let mut file =
                std::fs::File::create(output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut file);
            encoder
                .encode(&rgba, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("GIF编码失败: {}", e))?;
        }
        "bmp" => {
            img.save(output_path)
                .map_err(|e| format!("保存BMP失败: {}", e))?;
        }
        _ => return Err(format!("不支持的图片格式: {}", ext)),
    }
    Ok(())
}

/// 压缩图片为指定格式
#[tauri::command]
pub fn image_compress(path: String, quality: u8, format: String) -> Result<String, String> {
    ensure_output_dir()?;

    let img = ImageReader::open(&path)
        .map_err(|e| format!("打开图片失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码图片失败: {}", e))?;

    let fmt = format.to_lowercase();
    let ext = match fmt.as_str() {
        "jpeg" | "jpg" => "jpg",
        "png" => "png",
        "webp" => "webp",
        _ => return Err(format!("不支持的压缩格式: {}", format)),
    };
    let uuid = Uuid::new_v4();
    let output_path = format!("{}/compress_{}.{}", OUTPUT_DIR, uuid, ext);

    match fmt.as_str() {
        "jpeg" | "jpg" => {
            let rgb = img.to_rgb8();
            let (w, h) = rgb.dimensions();
            let q = quality.min(100).max(1);
            let mut file =
                std::fs::File::create(&output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, q);
            encoder
                .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG编码失败: {}", e))?;
        }
        "png" => {
            img.save(&output_path)
                .map_err(|e| format!("保存PNG失败: {}", e))?;
        }
        "webp" => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let mut file =
                std::fs::File::create(&output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            image::codecs::webp::WebPEncoder::new_lossless(&mut file)
                .encode(&rgba, w, h, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("WebP编码失败: {}", e))?;
        }
        _ => unreachable!(),
    }

    Ok(output_path)
}

/// 调整图片尺寸
#[tauri::command]
pub fn image_resize(
    path: String,
    width: Option<u32>,
    height: Option<u32>,
    percent: Option<f32>,
    keep_aspect: bool,
) -> Result<String, String> {
    ensure_output_dir()?;

    let img = ImageReader::open(&path)
        .map_err(|e| format!("打开图片失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码图片失败: {}", e))?;

    let (orig_w, orig_h) = (img.width(), img.height());

    let (new_w, new_h) = if let Some(pct) = percent {
        let ratio = pct / 100.0;
        (
            (orig_w as f64 * ratio as f64).round() as u32,
            (orig_h as f64 * ratio as f64).round() as u32,
        )
    } else {
        match (width, height) {
            (Some(w), Some(h)) => {
                if keep_aspect {
                    let ratio =
                        (w as f64 / orig_w as f64).min(h as f64 / orig_h as f64);
                    (
                        (orig_w as f64 * ratio).round() as u32,
                        (orig_h as f64 * ratio).round() as u32,
                    )
                } else {
                    (w, h)
                }
            }
            (Some(w), None) => {
                let ratio = w as f64 / orig_w as f64;
                (w, (orig_h as f64 * ratio).round() as u32)
            }
            (None, Some(h)) => {
                let ratio = h as f64 / orig_h as f64;
                ((orig_w as f64 * ratio).round() as u32, h)
            }
            (None, None) => return Err("请提供宽度、高度或百分比参数".to_string()),
        }
    };

    if new_w == 0 || new_h == 0 {
        return Err("目标尺寸不能为0".to_string());
    }

    let resized = image::imageops::resize(&img, new_w, new_h, FilterType::Lanczos3);

    let ext = get_extension(&path)?;
    let uuid = Uuid::new_v4();
    let output_path = format!("{}/resize_{}.{}", OUTPUT_DIR, uuid, ext);

    save_image(&image::DynamicImage::ImageRgba8(resized), &output_path)?;

    Ok(output_path)
}

/// 转换图片格式
#[tauri::command]
pub fn image_convert(path: String, target_format: String) -> Result<String, String> {
    ensure_output_dir()?;

    let img = ImageReader::open(&path)
        .map_err(|e| format!("打开图片失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码图片失败: {}", e))?;

    let fmt = target_format.to_lowercase();
    let ext = match fmt.as_str() {
        "jpeg" | "jpg" => "jpg",
        "png" => "png",
        "webp" => "webp",
        "gif" => "gif",
        "bmp" => "bmp",
        _ => return Err(format!("不支持的目标格式: {}", target_format)),
    };

    let uuid = Uuid::new_v4();
    let output_path = format!("{}/convert_{}.{}", OUTPUT_DIR, uuid, ext);

    save_image(&img, &output_path)?;

    Ok(output_path)
}

/// 裁剪图片
#[tauri::command]
pub fn image_crop(
    path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    ensure_output_dir()?;

    let img = ImageReader::open(&path)
        .map_err(|e| format!("打开图片失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码图片失败: {}", e))?;

    let cropped = image::imageops::crop_imm(&img, x, y, width, height).to_image();

    let ext = get_extension(&path)?;
    let uuid = Uuid::new_v4();
    let output_path = format!("{}/crop_{}.{}", OUTPUT_DIR, uuid, ext);

    save_image(&image::DynamicImage::ImageRgba8(cropped), &output_path)?;

    Ok(output_path)
}

/// 移除图片背景（调用 python3 -m rembg）
#[tauri::command]
pub fn image_remove_bg(path: String) -> Result<String, String> {
    ensure_output_dir()?;

    let uuid = Uuid::new_v4();
    let output_path = format!("{}/remove_bg_{}.png", OUTPUT_DIR, uuid);

    let output = Command::new("python3")
        .args(["-m", "rembg", "i", &path, &output_path])
        .output()
        .map_err(|e| format!("执行rembg失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rembg处理失败: {}", stderr));
    }

    Ok(output_path)
}
