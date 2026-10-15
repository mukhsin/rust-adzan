use tauri_winrt_notification::{Sound, Toast};

/// AUMID PowerShell — trik standar agar toast tampil tanpa registrasi app
/// (direkomendasikan di README tauri-winrt-notification).
const APP_ID: &str =
    "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

/// Tampilkan toast notification; gagal secara diam-diam (notifikasi bukan fitur kritis).
pub fn show(title: &str, body: &str) {
    let result = Toast::new(APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .show();
    if let Err(e) = result {
        eprintln!("Notifikasi gagal: {e}");
    }
}
