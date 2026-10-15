use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Adzan bawaan aplikasi (di-embed dari assets/ — selalu tersedia walau
/// exe dipindah tanpa folder assets).
pub static DEFAULT_ADZAN: &[u8] = include_bytes!("../assets/adzan.mp3");
pub static DEFAULT_ADZAN_SUBUH: &[u8] = include_bytes!("../assets/adzan-subuh.mp3");

/// Jalur pemutaran yang independen satu sama lain — tiap slot punya tombol
/// play/stop-nya sendiri (baris adzan umum & baris Subuh di Pengaturan).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Adzan umum (Dzuhur, Ashar, Maghrib, Isya).
    Umum,
    /// Adzan khusus Subuh.
    Subuh,
}

fn slot_idx(slot: Slot) -> usize {
    match slot {
        Slot::Umum => 0,
        Slot::Subuh => 1,
    }
}

/// Player aktif per slot. Play di satu slot tidak mengganggu slot lain;
/// play ulang di slot yang sama menghentikan pemutaran sebelumnya.
static SLOTS: Mutex<[Option<Arc<rodio::Player>>; 2]> = Mutex::new([None, None]);

/// Hentikan pemutaran pada satu slot.
pub fn stop_slot(slot: Slot) {
    if let Some(player) = SLOTS.lock().unwrap()[slot_idx(slot)].take() {
        player.stop();
    }
}

/// Apakah slot ini sedang memutar.
pub fn is_playing(slot: Slot) -> bool {
    SLOTS.lock().unwrap()[slot_idx(slot)].is_some()
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

/// Daftarkan player ke slot-nya dan jalankan thread penjaga stream.
fn register_player(
    slot: Slot,
    player: Arc<rodio::Player>,
    sink: rodio::MixerDeviceSink,
) -> Result<(), String> {
    // Play ulang di slot yang sama menghentikan pemutaran sebelumnya.
    if let Some(old) = SLOTS.lock().unwrap()[slot_idx(slot)].take() {
        old.stop();
    }
    SLOTS.lock().unwrap()[slot_idx(slot)] = Some(player.clone());

    std::thread::Builder::new()
        .name("adzan-audio".into())
        .spawn(move || {
            let _sink = sink; // jaga output stream tetap hidup
            player.sleep_until_end();
            // Bersihkan registrasi hanya jika masih player ini yang terdaftar.
            let mut slots = SLOTS.lock().unwrap();
            if slots[slot_idx(slot)]
                .as_ref()
                .is_some_and(|p| Arc::ptr_eq(p, &player))
            {
                slots[slot_idx(slot)] = None;
            }
        })
        .map_err(|e| format!("gagal spawn thread audio: {e}"))?;
    Ok(())
}

/// Putar file audio pada slot tertentu.
pub fn play_file(path: &Path, volume: f32, slot: Slot) -> Result<(), String> {
    stop_slot(slot);
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
    register_player(slot, player, sink)
}

/// Putar data audio dari memori (dipakai untuk adzan bawaan).
pub fn play_bytes(bytes: &'static [u8], volume: f32, slot: Slot) -> Result<(), String> {
    stop_slot(slot);
    let sink = rodio::DeviceSinkBuilder::open_default_sink()
        .map_err(|e| format!("perangkat audio tidak tersedia: {e}"))?;
    let player = Arc::new(
        rodio::play(sink.mixer(), Cursor::new(bytes))
            .map_err(|e| format!("gagal decode adzan bawaan: {e}"))?,
    );
    player.set_volume(volume.clamp(0.0, 1.0));
    register_player(slot, player, sink)
}

/// Pilih sumber audio untuk sholat tertentu dan putar di slot yang sesuai:
/// file kustom (settings) → file di folder assets → adzan bawaan (embed).
pub fn play_adzan(settings: &crate::settings::Settings, subuh: bool) -> Result<(), String> {
    let slot = if subuh { Slot::Subuh } else { Slot::Umum };
    let source = if subuh {
        resolve_subuh(settings)
    } else {
        resolve_adzan(settings)
    };
    match source {
        Some(path) => play_file(&path, settings.volume, slot),
        None => {
            let bytes = if subuh {
                DEFAULT_ADZAN_SUBUH
            } else {
                DEFAULT_ADZAN
            };
            play_bytes(bytes, settings.volume, slot)
        }
    }
}
