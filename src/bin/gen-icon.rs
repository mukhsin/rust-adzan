//! Tool dev: konversi assets/mosque.png menjadi aset ikon yang dibutuhkan
//! aplikasi. Output di assets/:
//!   - icon.png       (256×256 PNG — referensi & sumber ICO)
//!   - icon.ico       (ICO 256×256 — di-embed ke exe via build.rs/winresource)
//!   - icon.rgba      (256×256 RGBA straight-alpha, untuk ikon window)
//!   - icon-tray.rgba (32×32 RGBA, untuk tray icon)
//!
//! Jalankan: cargo run --bin gen-icon

use image::imageops::FilterType;

fn main() {
    println!("Konversi assets/mosque.png → aset ikon aplikasi…");

    let img = image::open("assets/mosque.png")
        .expect("assets/mosque.png tidak ditemukan/gagal decode")
        .to_rgba8();
    let (w, h) = img.dimensions();
    println!("  sumber: {w}×{h}");

    if w != h {
        println!("  ⚠ gambar tidak persegi ({w}×{h}) — akan di-resize ke persegi");
    }

    // PNG 256×256 + ICO (kontainer 22-byte membungkus PNG yang sama).
    let win = if (w, h) == (256, 256) {
        img.clone()
    } else {
        image::imageops::resize(&img, 256, 256, FilterType::Lanczos3)
    };
    let mut png = Vec::new();
    win.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("gagal encode PNG");
    std::fs::write("assets/icon.png", &png)
        .unwrap_or_else(|e| panic!("gagal tulis assets/icon.png: {e}"));
    write_ico(&png, "assets/icon.ico");
    println!("  ✓ assets/icon.png + assets/icon.ico (256×256)");

    // RGBA untuk ikon window (di-embed via include_bytes!).
    std::fs::write("assets/icon.rgba", win.as_raw())
        .unwrap_or_else(|e| panic!("gagal tulis assets/icon.rgba: {e}"));
    println!("  ✓ assets/icon.rgba (256×256)");

    // Ikon tray 32×32 (Windows tray men-skala sendiri dari sini).
    let tray = image::imageops::resize(&img, 32, 32, FilterType::Lanczos3);
    std::fs::write("assets/icon-tray.rgba", tray.as_raw())
        .unwrap_or_else(|e| panic!("gagal tulis assets/icon-tray.rgba: {e}"));
    println!("  ✓ assets/icon-tray.rgba (32×32)");
}

/// Tulis file .ico berisi satu entri PNG (ukuran 256 = 0 di header ICO).
fn write_ico(png: &[u8], path: &str) {
    let mut out = Vec::with_capacity(22 + png.len());
    out.extend_from_slice(&0u16.to_le_bytes()); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    out.extend_from_slice(&1u16.to_le_bytes()); // jumlah entri
    out.push(0); // width  (0 = 256)
    out.push(0); // height (0 = 256)
    out.push(0); // palet
    out.push(0); // reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // color planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    out.extend_from_slice(&(png.len() as u32).to_le_bytes()); // ukuran entri
    out.extend_from_slice(&22u32.to_le_bytes()); // offset entri
    out.extend_from_slice(png);
    std::fs::write(path, out).unwrap_or_else(|e| panic!("gagal tulis {path}: {e}"));
    println!("  ✓ {path} ({} bytes)", 22 + png.len());
}
