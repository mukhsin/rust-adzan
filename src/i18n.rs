//! Terjemahan teks UI (Indonesia / English).
//!
//! Semua string user-facing diambil lewat [`tr`] agar mudah ditambah bahasa
//! baru: cukup tambah kolom di tabel [`PHRASES`]. Kunci tanpa padanan Inggris
//! jatuh kembali ke Indonesia; kunci yang sama sekali tidak dikenal
//! dikembalikan apa adanya (mudah terdeteksi saat review).
//!
//! Placeholder `{nama}` / `{minutes}` diganti di tempat pemakaian via
//! `String::replace`.

use crate::settings::Language;

/// (kunci, Indonesia, English)
const PHRASES: &[(&str, &str, &str)] = &[
    // Umum / widget
    ("app_name", "Jadwal Sholat", "Prayer Times"),
    ("settings_title", "Pengaturan", "Settings"),
    (
        "close_hint",
        "Tutup (tetap berjalan di tray)",
        "Close (keeps running in tray)",
    ),
    ("menu_show", "Tampilkan Jendela", "Show Window"),
    ("menu_widget", "Widget Desktop", "Desktop Widget"),
    ("menu_quit", "Keluar", "Quit"),
    (
        "calc_failed",
        "Gagal menghitung jadwal.",
        "Failed to calculate the schedule.",
    ),
    ("countdown_prefix", "Menuju", "Until"),
    // Kartu tiket widget
    ("masuk_pukul", "— masuk pukul {waktu}", "— at {waktu}"),
    // Nama baris jadwal (termasuk sholat wajib, lihat PrayerKind::key)
    ("imsak", "Imsak", "Imsak"),
    ("subuh", "Subuh", "Fajr"),
    ("terbit", "Terbit", "Sunrise"),
    ("dzuhur", "Dzuhur", "Dhuhr"),
    ("ashar", "Ashar", "Asr"),
    ("maghrib", "Maghrib", "Maghrib"),
    ("isya", "Isya", "Isha"),
    // Toggle bell per sholat
    ("bell_on_title", "Mute adzan {nama}", "Mute {nama} adhan"),
    (
        "bell_off_title",
        "Bunyikan adzan {nama}",
        "Unmute {nama} adhan",
    ),
    // Settings — lokasi
    ("sec_location", "Lokasi", "Location"),
    ("f_city", "Kota", "City"),
    (
        "city_manual",
        "— Koordinat manual —",
        "— Manual coordinates —",
    ),
    ("f_lat", "Latitude", "Latitude"),
    ("f_lng", "Longitude", "Longitude"),
    ("f_tz", "Zona Waktu", "Timezone"),
    // Settings — perhitungan
    ("sec_calc", "Perhitungan", "Calculation"),
    ("f_method", "Metode", "Method"),
    (
        "m_kemenag",
        "Kemenag (Subuh 20°, Isya 18°)",
        "Kemenag (Fajr 20°, Isha 18°)",
    ),
    ("m_mwl", "Muslim World League", "Muslim World League"),
    (
        "m_egyptian",
        "Egyptian General Authority",
        "Egyptian General Authority",
    ),
    ("m_karachi", "Universitas Karachi", "University of Karachi"),
    (
        "m_umm_al_qura",
        "Umm al-Qura, Makkah",
        "Umm al-Qura, Makkah",
    ),
    ("m_isna", "ISNA (Amerika Utara)", "ISNA (North America)"),
    ("f_madhab", "Mazhab (Ashar)", "Madhab (Asr)"),
    (
        "madhab_shafi",
        "Syafi'i / Maliki / Hambali",
        "Shafi'i / Maliki / Hanbali",
    ),
    ("madhab_hanafi", "Hanafi", "Hanafi"),
    // Settings — audio
    ("sec_audio", "Audio Adzan", "Adhan Audio"),
    (
        "f_audio_enabled",
        "Mainkan adzan saat masuk waktu",
        "Play adhan at prayer time",
    ),
    ("f_volume", "Volume", "Volume"),
    ("f_adzan_file", "File adzan", "Adhan file"),
    ("f_subuh_file", "File khusus Subuh", "Fajr adhan file"),
    ("pick", "Pilih…", "Browse…"),
    ("stop_play", "Hentikan pemutaran", "Stop playback"),
    ("test_play", "Tes putar adzan", "Test adhan"),
    (
        "test_play_subuh",
        "Tes putar adzan Subuh",
        "Test Fajr adhan",
    ),
    ("pick_audio_title", "Pilih file adzan", "Choose adhan file"),
    ("src_custom", "Kustom: {nama}", "Custom: {nama}"),
    ("src_file", "File: {nama}", "File: {nama}"),
    ("src_embedded", "Bawaan (ter-embed)", "Built-in (embedded)"),
    // Settings — notifikasi
    ("sec_notif", "Notifikasi", "Notifications"),
    (
        "f_notif_enabled",
        "Tampilkan toast notifikasi",
        "Show toast notifications",
    ),
    (
        "f_pre_notif",
        "Pengingat sebelum adzan",
        "Remind before adhan",
    ),
    (
        "hint_pre_notif",
        "Menit sebelum waktu masuk",
        "Minutes before prayer time",
    ),
    ("dec", "Kurangi", "Decrease"),
    ("inc", "Tambah", "Increase"),
    // Settings — lainnya
    ("sec_misc", "Lainnya", "Other"),
    ("f_language", "Bahasa", "Language"),
    (
        "f_autostart",
        "Jalankan saat Windows menyala",
        "Run at Windows startup",
    ),
    ("reset_btn", "Reset semua pengaturan", "Reset all settings"),
    ("version_label", "Versi {versi}", "Version {versi}"),
    (
        "modal_reset_msg",
        "Kembalikan SEMUA pengaturan ke bawaan? (lokasi, audio, notifikasi, tema, auto-start)",
        "Reset ALL settings to defaults? (location, audio, notifications, theme, auto-start)",
    ),
    ("confirm_reset", "Ya, reset", "Yes, reset"),
    ("cancel", "Batal", "Cancel"),
    ("autostart_title", "Auto-start", "Auto-start"),
    (
        "err_reg_open",
        "gagal buka registry Run",
        "failed to open Run registry key",
    ),
    (
        "err_exe_path",
        "gagal dapat path exe",
        "failed to get exe path",
    ),
    (
        "err_reg_write",
        "gagal tulis registry",
        "failed to write registry",
    ),
    // Notifikasi toast (scheduler)
    (
        "notif_adzan_title",
        "Masuk waktu sholat",
        "It's prayer time",
    ),
    (
        "notif_adzan_body",
        "Telah masuk waktu sholat {nama}. Mari sholat. 🕌",
        "It is time for {nama} now. Let's pray. 🕌",
    ),
    ("notif_pre_title", "Pengingat adzan", "Adhan reminder"),
    (
        "notif_pre_body",
        "{minutes} menit lagi masuk waktu sholat {nama}.",
        "{nama} in {minutes} minutes.",
    ),
];

/// Ambil terjemahan `key` untuk `lang`.
pub fn tr(lang: Language, key: &'static str) -> &'static str {
    PHRASES
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|&(_, id, en)| match lang {
            Language::Id => id,
            Language::En => en,
        })
        .unwrap_or(key)
}

/// Nama hari, indeks 0 = Senin (`weekday().num_days_from_monday()`).
pub fn day_name(lang: Language, idx: usize) -> &'static str {
    const ID: [&str; 7] = [
        "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu",
    ];
    const EN: [&str; 7] = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    match lang {
        Language::Id => ID[idx],
        Language::En => EN[idx],
    }
}

/// Nama bulan hijri, indeks 0 = Muharam.
pub fn hijri_month_name(lang: Language, idx: usize) -> &'static str {
    const ID: [&str; 12] = [
        "Muharam",
        "Safar",
        "Rabiul Awal",
        "Rabiul Akhir",
        "Jumadil Awal",
        "Jumadil Akhir",
        "Rajab",
        "Sya'ban",
        "Ramadhan",
        "Syawal",
        "Zulkaidah",
        "Zulhijah",
    ];
    const EN: [&str; 12] = [
        "Muharram",
        "Safar",
        "Rabi' al-Awwal",
        "Rabi' al-Thani",
        "Jumada al-Ula",
        "Jumada al-Akhirah",
        "Rajab",
        "Sha'ban",
        "Ramadan",
        "Shawwal",
        "Dhu al-Qi'dah",
        "Dhu al-Hijjah",
    ];
    match lang {
        Language::Id => ID[idx],
        Language::En => EN[idx],
    }
}

/// Nama bulan, indeks 0 = Januari.
pub fn month_name(lang: Language, idx: usize) -> &'static str {
    const ID: [&str; 12] = [
        "Januari",
        "Februari",
        "Maret",
        "April",
        "Mei",
        "Juni",
        "Juli",
        "Agustus",
        "September",
        "Oktober",
        "November",
        "Desember",
    ];
    const EN: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    match lang {
        Language::Id => ID[idx],
        Language::En => EN[idx],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terjemahan_dua_bahasa() {
        assert_eq!(tr(Language::Id, "app_name"), "Jadwal Sholat");
        assert_eq!(tr(Language::En, "app_name"), "Prayer Times");
        assert_eq!(tr(Language::Id, "subuh"), "Subuh");
        assert_eq!(tr(Language::En, "subuh"), "Fajr");
    }

    #[test]
    fn kunci_tak_dikenal_kembali_mentah() {
        assert_eq!(tr(Language::En, "kunci_ngawur"), "kunci_ngawur");
    }

    #[test]
    fn nama_hari_dan_bulan() {
        assert_eq!(day_name(Language::Id, 0), "Senin");
        assert_eq!(day_name(Language::En, 6), "Sunday");
        assert_eq!(month_name(Language::Id, 0), "Januari");
        assert_eq!(month_name(Language::En, 11), "December");
        assert_eq!(hijri_month_name(Language::Id, 0), "Muharam");
        assert_eq!(hijri_month_name(Language::En, 4), "Jumada al-Ula");
    }
}
