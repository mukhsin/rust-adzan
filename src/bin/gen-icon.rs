//! Tool dev: konversi assets/mosque.png menjadi data piksel RGBA yang dibutuhkan
//! aplikasi (ikon window/taskbar + tray). Ikon dipakai apa adanya, tanpa latar.
//! Output di assets/:
//!   - icon.rgba      (256×256 RGBA straight-alpha, di-include_bytes! untuk ikon window)
//!   - icon-tray.rgba (32×32 RGBA, untuk tray icon)
//!
//! Jalankan: cargo run --bin gen-icon

use image::imageops::FilterType;

fn main() {
    println!("Konversi assets/mosque.png → RGBA ikon aplikasi…");

    let img = image::open("assets/mosque.png")
        .expect("assets/mosque.png tidak ditemukan/gagal decode")
        .to_rgba8();
    let (w, h) = img.dimensions();
    println!("  sumber: {w}×{h}");

    if w != h {
        println!("  ⚠ gambar tidak persegi ({w}×{h}) — akan di-resize ke persegi");
    }

    // Ikon window/taskbar 256×256.
    let win = if (w, h) == (256, 256) {
        img.clone()
    } else {
        image::imageops::resize(&img, 256, 256, FilterType::Lanczos3)
    };
    std::fs::write("assets/icon.rgba", win.as_raw())
        .unwrap_or_else(|e| panic!("gagal tulis assets/icon.rgba: {e}"));
    println!("  ✓ assets/icon.rgba (256×256)");

    // Ikon tray 32×32 (Windows tray men-skala sendiri dari sini).
    let tray = image::imageops::resize(&img, 32, 32, FilterType::Lanczos3);
    std::fs::write("assets/icon-tray.rgba", tray.as_raw())
        .unwrap_or_else(|e| panic!("gagal tulis assets/icon-tray.rgba: {e}"));
    println!("  ✓ assets/icon-tray.rgba (32×32)");
}
