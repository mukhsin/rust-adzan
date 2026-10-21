use std::borrow::Cow;
use std::path::Path;
use tauri_winrt_notification::{Sound, Toast};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, REG_EXPAND_SZ};

/// AUMID aplikasi sendiri — didaftarkan ke registry (HKCU) saat startup agar
/// toast tampil sebagai "Jadwal Sholat" dengan ikon aplikasi, bukan meminjam
/// identitas PowerShell (jebakan bawaan WinRT untuk app tanpa AUMID).
const APP_ID: &str = "JadwalSholat";

/// Path kunci registry tempat AUMID untuk app non-UWP didaftarkan.
const REG_PATH: &str = r"Software\Classes\AppUserModelId";

/// Ikon toast: icon.ico ter-embed di binary, disalin ke folder config agar
/// IconUri di registry selalu menunjuk file yang pasti ada (portable maupun
/// hasil install).
const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.ico");

/// Daftarkan identitas notifikasi ke registry (HKCU\...\AppUserModelId).
/// Idempoten; ditulis ulang tiap startup agar path ikon yang berpindah
/// ikut terbarui (pola sama dengan sinkronisasi auto-start).
pub fn ensure_aumid_registered() {
    if let Err(e) = daftarkan_aumid() {
        crate::logger::error(&format!("gagal daftarkan AUMID notifikasi: {e}"));
    }
}

fn daftarkan_aumid() -> Result<(), Box<dyn std::error::Error>> {
    let icon_path = crate::settings::config_dir().join("icon.ico");
    let perlu_tulis = match std::fs::read(&icon_path) {
        Ok(isi) => isi != ICON_BYTES,
        Err(_) => true,
    };
    if perlu_tulis {
        std::fs::write(&icon_path, ICON_BYTES)?;
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(Path::new(REG_PATH).join(APP_ID))?;
    // DisplayName & IconUri memakai REG_EXPAND_SZ sesuai dokumentasi Windows
    // untuk kunci AppUserModelId.
    set_expand_sz(&key, "DisplayName", crate::settings::APP_NAME)?;
    set_expand_sz(&key, "IconUri", &file_uri(&icon_path))?;
    Ok(())
}

/// Tulis nilai string REG_EXPAND_SZ (winreg 0.56 menyimpan RegValue mentah
/// sebagai bytes UTF-16LE + terminator null).
fn set_expand_sz(key: &RegKey, name: &str, value: &str) -> std::io::Result<()> {
    let mut bytes: Vec<u8> = value.encode_utf16().flat_map(u16::to_le_bytes).collect();
    bytes.extend_from_slice(&[0, 0]);
    key.set_raw_value(
        name,
        &winreg::RegValue {
            bytes: Cow::Owned(bytes),
            vtype: REG_EXPAND_SZ,
        },
    )
}

/// Konversi path Windows ke `file:///` URI (slash maju + percent-encoding)
/// untuk nilai IconUri.
fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file:///");
    for &b in path.to_string_lossy().replace('\\', "/").as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' | b'/' => {
                uri.push(b as char);
            }
            _ => uri.push_str(&format!("%{b:02X}")),
        }
    }
    uri
}

/// Tampilkan toast notification; gagal secara diam-diam (notifikasi bukan fitur kritis).
pub fn show(title: &str, body: &str) {
    let result = Toast::new(APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .show();
    if let Err(e) = result {
        crate::logger::error(&format!("notifikasi gagal: {e}"));
    }
}

/// Toast yang badannya bisa diklik — dipakai toast pembaruan: klik membuka
/// `url` (halaman rilis) di browser bawaan.
pub fn show_clickable(title: &str, body: &str, url: String) {
    let result = Toast::new(APP_ID)
        .title(title)
        .text1(body)
        .sound(Some(Sound::Default))
        .on_activated(move |_| {
            open_url(&url);
            Ok(())
        })
        .show();
    if let Err(e) = result {
        crate::logger::error(&format!("notifikasi gagal: {e}"));
    }
}

/// Buka URL di browser bawaan via `cmd /C start`. CREATE_NO_WINDOW mencegah
/// jendela console berkedip pada build GUI.
fn open_url(url: &str) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
    if let Err(e) = result {
        crate::logger::error(&format!("gagal membuka {url}: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aumid_terdaftar_dengan_nama_dan_ikon() {
        ensure_aumid_registered();

        let icon_path = crate::settings::config_dir().join("icon.ico");
        assert!(
            icon_path.exists(),
            "icon.ico harus tersalin ke folder config"
        );

        use winreg::enums::KEY_READ;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu
            .open_subkey_with_flags(Path::new(REG_PATH).join(APP_ID), KEY_READ)
            .expect("kunci AUMID harus terdaftar");
        let nama: String = key.get_value("DisplayName").expect("DisplayName ada");
        assert_eq!(nama, crate::settings::APP_NAME);
        let uri: String = key.get_value("IconUri").expect("IconUri ada");
        assert!(uri.starts_with("file:///"), "IconUri harus file URI: {uri}");
        assert!(!uri.contains(' '), "spasi harus di-encode: {uri}");
    }

    #[test]
    fn file_uri_encode_path_dengan_spasi() {
        assert_eq!(
            file_uri(Path::new(r"C:\Dir Nama\icon.ico")),
            "file:///C:/Dir%20Nama/icon.ico"
        );
    }
}
