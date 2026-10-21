//! Guard single-instance via mutex Windows bernama.
//!
//! Insiden v1.1: dua copy jalan bersamaan → dua scheduler → adzan bunyi
//! dua kali. Instance kedua kini menampilkan toast "sudah berjalan" lalu
//! langsung keluar; HANDLE mutex sengaja tidak ditutup agar terlepas
//! otomatis saat proses mati.

use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

const MUTEX_NAME: &str = "Local\\JadwalSholat.SingleInstance";

/// `true` bila ini instance pertama (pemilik mutex). Gagal membuat mutex
/// tidak memblokir app — guard boleh gagal, app tetap jalan.
fn acquire() -> bool {
    let mut name: Vec<u16> = MUTEX_NAME.encode_utf16().collect();
    name.push(0);
    match unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) } {
        // HANDLE sengaja di-leak: hidup selama proses, OS melepas saat mati.
        Ok(_handle) => {
            let sudah_ada = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
            !sudah_ada
        }
        Err(e) => {
            crate::logger::error(&format!("gagal membuat mutex single-instance: {e}"));
            true
        }
    }
}

/// Panggil di awal `main`: bila sudah ada instance lain, tampilkan toast
/// lalu keluar (tidak kembali).
pub fn exit_if_duplicate() {
    if !acquire() {
        crate::logger::error("instance lain sudah berjalan — instance ini keluar");
        let lang = crate::state::settings().language;
        crate::notify::show(
            crate::settings::APP_NAME,
            crate::i18n::tr(lang, "already_running"),
        );
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nama_mutex_utf16_terminated() {
        // Nama mutex di-encode UTF-16 + null terminator (invarian yang
        // dibutuhkan CreateMutexW).
        let mut name: Vec<u16> = MUTEX_NAME.encode_utf16().collect();
        name.push(0);
        assert!(name.last() == Some(&0));
        assert_eq!(name.len(), MUTEX_NAME.chars().count() + 1);
        assert!(MUTEX_NAME.starts_with("Local\\"));
    }
}
