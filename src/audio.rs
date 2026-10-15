use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Adzan bawaan aplikasi (di-embed dari assets/ — selalu tersedia walau
/// exe dipindah tanpa folder assets).
pub static DEFAULT_ADZAN: &[u8] = include_bytes!("../assets/adzan.mp3");
pub static DEFAULT_ADZAN_SUBUH: &[u8] = include_bytes!("../assets/adzan-subuh.mp3");

/// Player yang sedang memutar tes/adzan. Play baru menghentikan yang lama,
/// dan bisa di-stop manual lewat [`stop_current`].
static CURRENT_PLAYER: Mutex<Option<Arc<rodio::Player>>> = Mutex::new(None);

/// Hentikan pemutaran yang sedang berjalan (jika ada).
pub fn stop_current() {
    if let Some(player) = CURRENT_PLAYER.lock().unwrap().take() {
        player.stop();
    }
}

/// Cari file adzan: pakai file dari settings jika ada; kalau tidak, cari
/// `assets/adzan.mp3` di dekat exe / dir config / cwd.
pub fn resolve_adzan(settings: &crate::settings::Settings) -> Option<PathBuf> {
    if let Some(p) = &settings.adzan_file
        && p.exists()
    {
        return Some(p.clone());
    }
    candidates("adzan.mp3").into_iter().find(|p| p.exists())
}

pub fn resolve_subuh(settings: &crate::settings::Settings) -> Option<PathBuf> {
    if let Some(p) = &settings.subuh_file
        && p.exists()
    {
        return Some(p.clone());
    }
    let fallback = resolve_adzan(settings);
    ["adzan-subuh.mp3", "adzan_subuh.mp3"]
        .iter()
        .map(|name| candidates(name))
        .flatten()
        .find(|p| p.exists())
        .or(fallback)
}

fn candidates(filename: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        out.push(dir.join("assets").join(filename));
        out.push(dir.join(filename));
    }
    out.push(crate::settings::config_dir().join(filename));
    out.push(PathBuf::from("assets").join(filename));
    out
}

/// Putar data audio dari memori (dipakai untuk adzan bawaan).
pub fn play_bytes(bytes: &'static [u8], volume: f32) -> Result<(), String> {
    stop_current();
    let sink = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|e| format!("perangkat audio tidak tersedia: {e}"))?;
    let player = Arc::new(
        rodio::play(sink.mixer(), Cursor::new(bytes))
            .map_err(|e| format!("gagal decode adzan bawaan: {e}"))?,
    );
    player.set_volume(volume.clamp(0.0, 1.0));
    *CURRENT_PLAYER.lock().unwrap() = Some(player.clone());

    std::thread::Builder::new()
        .name("adzan-audio".into())
        .spawn(move || {
            let _sink = sink;
            player.sleep_until_end();
            let mut current = CURRENT_PLAYER.lock().unwrap();
            if current.as_ref().is_some_and(|p| Arc::ptr_eq(p, &player)) {
                *current = None;
            }
        })
        .map_err(|e| format!("gagal spawn thread audio: {e}"))?;
    Ok(())
}

/// Putar file audio — hanya satu pemutaran pada satu waktu:
/// play baru otomatis menghentikan yang sedang berjalan, dan bisa
/// dihentikan kapan saja lewat [`stop_current`].
pub fn play_file(path: &Path, volume: f32) -> Result<(), String> {
    stop_current();
    let file =
        std::fs::File::open(path).map_err(|e| format!("gagal buka {}: {e}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let sink = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|e| format!("perangkat audio tidak tersedia: {e}"))?;
    let player = Arc::new(
        rodio::play(sink.mixer(), reader)
            .map_err(|e| format!("gagal decode {}: {e}", path.display()))?,
    );
    player.set_volume(volume.clamp(0.0, 1.0));
    *CURRENT_PLAYER.lock().unwrap() = Some(player.clone());

    std::thread::Builder::new()
        .name("adzan-audio".into())
        .spawn(move || {
            let _sink = sink; // jaga output stream tetap hidup
            player.sleep_until_end();
            // Bersihkan registrasi hanya jika masih player ini yang terdaftar.
            let mut current = CURRENT_PLAYER.lock().unwrap();
            if current.as_ref().is_some_and(|p| Arc::ptr_eq(p, &player)) {
                *current = None;
            }
        })
        .map_err(|e| format!("gagal spawn thread audio: {e}"))?;
    Ok(())
}

/// Pilih sumber audio untuk sholat tertentu dan putar:
/// file kustom (settings) → file di folder assets → adzan bawaan (embed).
/// Kembalikan deskripsi sumber untuk logging/status UI.
pub fn play_adzan(settings: &crate::settings::Settings, subuh: bool) -> Result<String, String> {
    let source = if subuh {
        resolve_subuh(settings)
    } else {
        resolve_adzan(settings)
    };
    match source {
        Some(path) => {
            play_file(&path, settings.volume)?;
            Ok(format!("memutar {}", path.display()))
        }
        None => {
            let bytes = if subuh {
                DEFAULT_ADZAN_SUBUH
            } else {
                DEFAULT_ADZAN
            };
            play_bytes(bytes, settings.volume)?;
            Ok("memutar adzan bawaan".to_string())
        }
    }
}
