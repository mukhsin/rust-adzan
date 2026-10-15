use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "JadwalSholat";
const LEGACY_VALUE_NAME: &str = "RustAdzan";

/// Set/hapus auto-start via registry HKCU Run (tidak butuh hak admin).
pub fn set_enabled(enable: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE | KEY_QUERY_VALUE)
        .map_err(|e| format!("gagal buka registry Run: {e}"))?;

    if enable {
        let exe = std::env::current_exe().map_err(|e| format!("gagal dapat path exe: {e}"))?;
        let quoted = format!("\"{}\"", exe.display());
        key.set_value(VALUE_NAME, &quoted)
            .map_err(|e| format!("gagal tulis registry: {e}"))?;
    } else {
        // Tidak masalah jika value tidak ada.
        let _ = key.delete_value(VALUE_NAME);
    }
    // Bersihkan value dari nama aplikasi lama (jika ada).
    let _ = key.delete_value(LEGACY_VALUE_NAME);
    Ok(())
}
