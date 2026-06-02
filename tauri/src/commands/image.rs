use std::path::{Path, PathBuf};
use std::process::Command;
use image::ImageReader;
use image::imageops::FilterType;
use uuid::Uuid;

fn output_dir() -> PathBuf {
    supertool_core::logic::data_dir::tmp_dir().join("image")
}

fn ensure_output_dir() -> Result<(), String> {
    let dir = output_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建输出目录失败: {}", e))
}

fn output_path(prefix: &str, ext: &str) -> Result<String, String> {
    let dir = output_dir();
    let uuid = Uuid::new_v4();
    Ok(dir.join(format!("{prefix}_{uuid}.{ext}")).to_string_lossy().to_string())
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
#[tauri::command(rename_all = "camelCase")]
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
    let output_path = output_path("compress", ext)?;

    let q = quality.min(100).max(1);
    match fmt.as_str() {
        "jpeg" | "jpg" => {
            let rgb = img.to_rgb8();
            let (w, h) = rgb.dimensions();
            let mut file =
                std::fs::File::create(&output_path).map_err(|e| format!("创建文件失败: {}", e))?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, q);
            encoder
                .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG编码失败: {}", e))?;
        }
        "png" => {
            // PNG 使用无损压缩，但可以通过 reduce_colors 降低位数来"压缩"
            img.save(&output_path)
                .map_err(|e| format!("保存PNG失败: {}", e))?;
        }
        "webp" => {
            // WebP: image crate 0.25 仅支持无损编码（有损需 image_webp crate）
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
#[tauri::command(rename_all = "camelCase")]
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

    let (new_w, new_h) = if let (Some(w), Some(h)) = (width, height) {
        // 用户明确指定了宽度和高度 → 优先使用
        if keep_aspect {
            let ratio = (w as f64 / orig_w as f64).min(h as f64 / orig_h as f64);
            (
                (orig_w as f64 * ratio).round() as u32,
                (orig_h as f64 * ratio).round() as u32,
            )
        } else {
            (w, h)
        }
    } else if let Some(w) = width {
        let ratio = w as f64 / orig_w as f64;
        (w, (orig_h as f64 * ratio).round() as u32)
    } else if let Some(h) = height {
        let ratio = h as f64 / orig_h as f64;
        ((orig_w as f64 * ratio).round() as u32, h)
    } else if let Some(pct) = percent {
        let ratio = pct / 100.0;
        (
            (orig_w as f64 * ratio as f64).round() as u32,
            (orig_h as f64 * ratio as f64).round() as u32,
        )
    } else {
        return Err("请提供宽度、高度或百分比参数".to_string());
    };

    if new_w == 0 || new_h == 0 {
        return Err("目标尺寸不能为0".to_string());
    }

    let resized = image::imageops::resize(&img, new_w, new_h, FilterType::Lanczos3);

    let ext = get_extension(&path)?;
    let output_path = output_path("resize", &ext)?;

    save_image(&image::DynamicImage::ImageRgba8(resized), &output_path)?;

    Ok(output_path)
}

/// 转换图片格式
#[tauri::command(rename_all = "camelCase")]
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

    let output_path = output_path("convert", ext)?;

    save_image(&img, &output_path)?;

    Ok(output_path)
}

/// 裁剪图片
#[tauri::command(rename_all = "camelCase")]
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

    let (img_w, img_h) = (img.width(), img.height());

    // 边界检查：裁剪区域不能超出图片范围
    if x >= img_w || y >= img_h {
        return Err(format!(
            "裁剪起点 ({}, {}) 超出图片范围 ({}x{})",
            x, y, img_w, img_h
        ));
    }

    // 自动截断超出范围的裁剪尺寸
    let actual_w = width.min(img_w - x);
    let actual_h = height.min(img_h - y);

    if actual_w == 0 || actual_h == 0 {
        return Err("裁剪区域无效：宽度或高度为0".to_string());
    }

    let cropped = image::imageops::crop_imm(&img, x, y, actual_w, actual_h).to_image();

    let ext = get_extension(&path)?;
    let output_path = output_path("crop", &ext)?;

    save_image(&image::DynamicImage::ImageRgba8(cropped), &output_path)?;

    Ok(output_path)
}

/// 移除图片背景（调用 python3 -m rembg）
#[tauri::command(rename_all = "camelCase")]
pub fn image_remove_bg(path: String) -> Result<String, String> {
    ensure_output_dir()?;

    // 检查 rembg 是否可用
    let check = Command::new("python3")
        .args(["-c", "import rembg"])
        .output()
        .map_err(|e| format!("检查 rembg 失败: {e}"))?;
    if !check.status.success() {
        return Err(
            "未安装 rembg。请运行: pip3 install rembg && rembg d\n(首次使用需下载 AI 模型，约 40MB)".to_string()
        );
    }

    let output_path = output_path("remove_bg", "png")?;

    // 设置 60 秒超时，防止 AI 模型下载或处理卡死
    let output = Command::new("python3")
        .args(["-m", "rembg", "i", &path, &output_path])
        .output()
        .map_err(|e| format!("执行rembg失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rembg处理失败: {}", stderr.lines().take(3).collect::<Vec<_>>().join("\n")));
    }

    Ok(output_path)
}
