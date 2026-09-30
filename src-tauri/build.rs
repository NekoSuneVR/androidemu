use std::{fs, path::Path};

const ICON_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41,
    0x54, 0x08, 0xD7, 0x63, 0x60, 0x60, 0x60, 0xF8,
    0x0F, 0x00, 0x01, 0x04, 0x01, 0x00, 0x5F, 0xE5,
    0xC3, 0x4B, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn ensure_temporary_icons() {
    let icon_dir = Path::new("icons");
    let png_path = icon_dir.join("icon.png");
    let ico_path = icon_dir.join("icon.ico");

    fs::create_dir_all(icon_dir).expect("failed to create Tauri icon directory");

    if !png_path.exists() {
        fs::write(&png_path, ICON_PNG).expect("failed to write temporary Tauri PNG icon");
    }

    if !ico_path.exists() {
        // ICO header + one directory entry + the PNG payload.
        // Modern Windows ICO files may embed PNG image data directly.
        let mut ico = Vec::with_capacity(22 + ICON_PNG.len());
        ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // image type: icon
        ico.extend_from_slice(&1u16.to_le_bytes()); // image count

        ico.push(1); // width
        ico.push(1); // height
        ico.push(0); // palette
        ico.push(0); // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // color planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        ico.extend_from_slice(&(ICON_PNG.len() as u32).to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes()); // image offset
        ico.extend_from_slice(ICON_PNG);

        fs::write(&ico_path, ico).expect("failed to write temporary Tauri ICO icon");
    }
}

fn main() {
    ensure_temporary_icons();
    tauri_build::build()
}
