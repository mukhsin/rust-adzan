use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metode perhitungan jadwal sholat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalcMethod {
    /// Kemenag Indonesia: Subuh 20°, Isya 18° (preset "Singapore" di crate salah).
    Kemenag,
    MuslimWorldLeague,
    Egyptian,
    Karachi,
    UmmAlQura,
    NorthAmerica,
}

impl Default for CalcMethod {
    fn default() -> Self {
        Self::Kemenag
    }
}

/// Mazhab untuk perhitungan waktu Ashar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MadhabChoice {
    #[default]
    Shafi,
    Hanafi,
}

/// Tema UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn is_dark(&self) -> bool {
        *self == Theme::Dark
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Nama kota dari daftar bawaan; `None` jika memakai koordinat manual.
    pub city: Option<String>,
    pub lat: f64,
    pub lng: f64,
    /// Nama zona waktu chrono-tz, mis. "Asia/Jakarta".
    pub timezone: String,
    pub method: CalcMethod,
    pub madhab: MadhabChoice,

    pub audio_enabled: bool,
    /// File MP3 adzan umum; `None` = cari di lokasi default (lihat `audio::resolve_*`).
    pub adzan_file: Option<PathBuf>,
    /// File MP3 khusus Subuh; `None` = pakai `adzan_file`.
    pub subuh_file: Option<PathBuf>,
    /// Volume 0.0–1.0
    pub volume: f32,

    pub notif_enabled: bool,
    /// Berapa menit sebelum adzan dikirim notifikasi pengingat.
    pub pre_notif_minutes: u32,

    pub widget_enabled: bool,
    pub autostart: bool,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        let jakarta = crate::cities::find_city("Jakarta").expect("Jakarta ada di daftar kota");
        Self {
            city: Some(jakarta.name.to_string()),
            lat: jakarta.lat,
            lng: jakarta.lng,
            timezone: jakarta.timezone.name().to_string(),
            method: CalcMethod::Kemenag,
            madhab: MadhabChoice::Shafi,
            audio_enabled: true,
            adzan_file: None,
            subuh_file: None,
            volume: 0.9,
            notif_enabled: true,
            pre_notif_minutes: 10,
            widget_enabled: false,
            autostart: false,
            theme: Theme::Dark,
        }
    }
}

impl Settings {
    pub fn timezone(&self) -> Tz {
        self.timezone.parse().unwrap_or(chrono_tz::Asia::Jakarta)
    }

    /// Zona waktu pilihan untuk input koordinat manual.
    pub fn set_timezone_name(&mut self, name: &str) {
        self.timezone = name.to_string();
    }
}

/// Direktori config: %APPDATA%\rust-adzan
pub fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("rust-adzan");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load() -> Settings {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("Gagal membaca config ({e}); memakai default.");
            Settings::default()
        }),
        Err(_) => Settings::default(),
    }
}

pub fn save(settings: &Settings) {
    let path = config_path();
    match serde_json::to_string_pretty(settings) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("Gagal menyimpan config: {e}");
            }
        }
        Err(e) => eprintln!("Gagal serialisasi config: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serde_roundtrip() {
        let mut s = Settings::default();
        s.city = Some("Makassar".into());
        s.lat = -5.1477;
        s.lng = 119.4327;
        s.timezone = "Asia/Makassar".into();
        s.method = CalcMethod::Karachi;
        s.madhab = MadhabChoice::Hanafi;
        s.volume = 0.42;
        s.pre_notif_minutes = 15;
        s.widget_enabled = true;

        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn settings_default_terparse_dari_json_tanpa_field() {
        // Config lama/parsial tetap terbaca berkat #[serde(default)].
        let back: Settings = serde_json::from_str(r#"{"lat": -7.0}"#).unwrap();
        assert_eq!(back.lat, -7.0);
        assert_eq!(back.method, CalcMethod::Kemenag);
        assert!(back.audio_enabled);
    }
}
