//! 打包时把项目根目录 `.env` 中的 `CLX_BASE_URL` 编译进二进制（`CLX_EMBEDDED_BASE_URL`），
//! 运行时无需任何配置文件。Windows 下同时从 `assets/icon.png` 生成 ICO 并嵌入 exe。

fn main() {
    println!("cargo:rustc-env=CLX_APP_VERSION={}", env!("CARGO_PKG_VERSION"));
    embed_base_url_from_env_file();

    #[cfg(windows)]
    embed_windows_icon();
}

/// 读取仓库根目录 `.env`（与 Tauri 共用）中的 `CLX_BASE_URL`，
/// 通过 `cargo:rustc-env` 编译进二进制；`.env` 变化时自动触发重新构建。
fn embed_base_url_from_env_file() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let env_file = std::path::Path::new(&manifest_dir).join("../.env");
    println!("cargo:rerun-if-changed={}", env_file.display());
    let Ok(content) = std::fs::read_to_string(&env_file) else {
        return;
    };
    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "CLX_BASE_URL" {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            println!("cargo:rustc-env=CLX_EMBEDDED_BASE_URL={value}");
        }
        return; // 找到键即结束
    }
}

#[cfg(windows)]
fn embed_windows_icon() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let png = manifest_dir.join("assets/icon.png");
    println!("cargo:rerun-if-changed={}", png.display());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = out_dir.join("icon.ico");
    png_to_ico(&png, &ico).expect("生成应用图标失败");

    let wix_icon = manifest_dir.join("wix/Product.ico");
    if let Some(parent) = wix_icon.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::copy(&ico, &wix_icon).expect("复制 MSI 图标到 wix/Product.ico 失败");

    let mut res = winres::WindowsResource::new();
    res.set_icon(ico.to_str().expect("icon path utf-8"));
    res.compile().expect("嵌入 Windows 图标资源失败");
}

#[cfg(windows)]
fn png_to_ico(png_path: &std::path::Path, ico_path: &std::path::Path) -> Result<(), String> {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
    use image::imageops::FilterType;

    let img = image::open(png_path).map_err(|e| format!("读取 {png_path:?}: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();

    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for size in [256u32, 128, 64, 48, 32, 16] {
        let icon_rgba = if w == size && h == size {
            rgba.clone()
        } else {
            image::imageops::resize(&rgba, size, size, FilterType::Lanczos3)
        };
        let icon_image = IconImage::from_rgba_data(size, size, icon_rgba.into_raw());
        let entry = IconDirEntry::encode(&icon_image)
            .map_err(|e| format!("编码 {size}x{size} 图标: {e}"))?;
        icon_dir.add_entry(entry);
    }

    let file = std::fs::File::create(ico_path).map_err(|e| format!("创建 {ico_path:?}: {e}"))?;
    icon_dir
        .write(file)
        .map_err(|e| format!("写入 {ico_path:?}: {e}"))?;
    Ok(())
}
