//! Lapisan UI aplikasi.
//!
//! Susunan modul:
//! - [`app`] — root dom window utama: tab, tema, perintah lintas window.
//! - [`tray`] — ikon system tray + menunya.
//! - [`window`] — manajemen window (widget window, clamp posisi, show/hide).
//! - [`components`] — komponen komposabel (tiket, papan jadwal, bar atas…).
//!   - `components::ui` — komponen atomik (tombol, modal, sakelar, …).
//! - [`screens`] — layar tab window utama (Harian, Pengaturan, Tentang).

pub mod app;
pub mod components;
pub mod screens;
pub mod tray;
pub mod window;

pub use app::app;

/// CSS hasil kompilasi Tailwind (assets/tailwind.css, dibangun via
/// tools/build-css.sh). Dibangun ulang SEBELUM cargo build — di-embed saat
/// kompilasi.
pub static TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");
/// Data piksel RGBA (256×256) untuk ikon window/taskbar.
pub static WINDOW_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon.rgba");
/// Data piksel RGBA (32×32) untuk tray icon.
pub static TRAY_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon-tray.rgba");
