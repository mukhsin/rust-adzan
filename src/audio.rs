use std::path::{Path, PathBuf};
use std::time::Duration;

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
    candidates("adzan_subuh.mp3")
        .into_iter()
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

/// Putar file audio di thread terpisah agar UI tidak pernah terblok.
/// Stream & player dijaga hidup oleh thread sampai selesai.
pub fn play_file(path: &Path, volume: f32) -> Result<(), String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("gagal buka {}: {e}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let path_display = path.display().to_string();
    std::thread::Builder::new()
        .name("adzan-audio".into())
        .spawn(move || -> Result<(), String> {
            let sink = rodio::DeviceSinkBuilder::open_default_sink()
                .map_err(|e| format!("perangkat audio tidak tersedia: {e}"))?;
            let player = rodio::play(sink.mixer(), reader)
                .map_err(|e| format!("gagal decode {path_display}: {e}"))?;
            player.set_volume(volume.clamp(0.0, 1.0));
            player.sleep_until_end();
            Ok(())
        })
        .map_err(|e| format!("gagal spawn thread audio: {e}"))?;
    Ok(())
}

/// Fallback: bunyi "beep" 3x jika tidak ada file adzan sama sekali,
/// supaya aplikasi tetap memberi tanda saat masuk waktu sholat.
pub fn play_fallback_beep(volume: f32) -> Result<(), String> {
    let vol = volume.clamp(0.0, 1.0);
    std::thread::Builder::new()
        .name("adzan-beep".into())
        .spawn(move || -> Result<(), String> {
            use rodio::source::{SineWave, Source};
            let sink = rodio::DeviceSinkBuilder::open_default_sink()
                .map_err(|e| format!("perangkat audio tidak tersedia: {e}"))?;
            let player = rodio::Player::connect_new(sink.mixer());
            player.set_volume(vol * 0.6);
            for _ in 0..3 {
                let beep = SineWave::new(880.0)
                    .take_duration(Duration::from_millis(350))
                    .amplify(1.0);
                player.append(beep);
                std::thread::sleep(Duration::from_millis(200));
            }
            player.sleep_until_end();
            Ok(())
        })
        .map_err(|e| format!("gagal spawn thread audio: {e}"))?;
    Ok(())
}

/// Pilih sumber audio untuk sholat tertentu dan putar.
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
            play_fallback_beep(settings.volume)?;
            Ok("memutar beep fallback (file adzan belum diatur)".to_string())
        }
    }
}
