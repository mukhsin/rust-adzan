//! Kelas gaya tombol (Tailwind) — varian tombol atomik ala komponen
//! "Tombol Sekunder" design system: pass + garis pass-line, radius 8.
//!
//! Dipakai pada elemen `<button>`: `<button class="{BTN}" …>`.

/// Tombol sekunder dengan teks (Pilih…, Batal, …).
pub const BTN: &str = "flex items-center gap-2 bg-pass border border-pass-line rounded-lg px-3 py-2 text-[13px] font-semibold text-ink hover:border-ink-3 transition-colors cursor-pointer";

/// Tombol ikon saja (play/stop adzan).
pub const BTN_ICON: &str = "p-2 bg-pass border border-pass-line rounded-lg text-ink-2 hover:border-ink-3 transition-colors cursor-pointer";

/// Tombol konfirmasi utama (isian ink, teks on-ink).
pub const BTN_CONFIRM: &str = "px-3 py-2 rounded-lg text-[13px] font-semibold bg-ink text-on-ink hover:opacity-90 transition-opacity cursor-pointer";

/// Tombol aksi merah (Ya, reset).
pub const BTN_DANGER: &str = "px-3 py-2 rounded-lg text-[13px] font-semibold bg-red-500 text-white hover:bg-red-600 transition-colors cursor-pointer";

/// Tombol aksi aksen amber (Perbarui & Instal) — sorotan ala "Penanda
/// Hari Ini" design system.
pub const BTN_AMBER: &str = "flex items-center gap-2 px-3 py-2 rounded-lg text-[13px] font-semibold bg-amber text-amber-ink hover:brightness-105 transition-[filter] cursor-pointer disabled:opacity-60 disabled:cursor-wait";
