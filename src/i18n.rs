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
    ("tab_daily", "Harian", "Daily"),
    ("tab_monthly", "Bulanan", "Monthly"),
    ("tab_kalender", "Kalender", "Calendar"),
    ("tab_tentang", "Tentang", "About"),
    ("coming_soon", "Belum tersedia", "Not available yet"),
    ("col_tgl", "Tgl", "Date"),
    ("today", "Hari ini", "Today"),
    ("day_ahad", "Ahad", "Sunday"),
    ("prev_month", "Bulan sebelumnya", "Previous month"),
    ("next_month", "Bulan berikutnya", "Next month"),
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
    ("unit_minutes", "menit", "min"),
    ("win_minimize", "Minimalkan", "Minimize"),
    ("win_maximize", "Maksimalkan", "Maximize"),
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
        "hint_notif_toast",
        "toast saat waktu masuk",
        "toast when prayer time starts",
    ),
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
    ("f_imsak_notif", "Notifikasi imsak", "Imsak notification"),
    (
        "hint_imsak_notif",
        "toast saat masuk waktu imsak",
        "toast when imsak time starts",
    ),
    // Settings — koreksi waktu per sholat
    (
        "offset_hint",
        "waktu bawaan {waktu}",
        "default time {waktu}",
    ),
    ("dec", "Kurangi", "Decrease"),
    ("inc", "Tambah", "Increase"),
    // Settings — tampilan
    ("sec_appearance", "Tampilan", "Appearance"),
    ("f_theme", "Tema", "Theme"),
    ("theme_system", "Sistem", "System"),
    ("theme_dark", "Gelap", "Dark"),
    ("theme_light", "Terang", "Light"),
    ("f_time_format", "Format waktu", "Time format"),
    ("fmt_24", "24 jam", "24-hour"),
    ("fmt_12", "12 jam", "12-hour"),
    // Settings — lainnya
    ("sec_misc", "Lainnya", "Other"),
    // Layar Tentang
    (
        "about_tagline",
        "Pengingat jadwal sholat & adzan untuk Windows — 100% offline.",
        "Prayer times & adhan reminder for Windows — 100% offline.",
    ),
    ("about_info", "Informasi", "Information"),
    ("about_hijri", "Hijriah", "Hijri calendar"),
    ("about_adzan", "Adzan", "Adhan"),
    (
        "about_adzan_value",
        "Tersemat · bisa kustom",
        "Embedded · customizable",
    ),
    ("about_source", "Sumber Kode", "Source code"),
    ("about_astronomis", "Astronomis", "Astronomical"),
    (
        "about_built",
        "Dibangun dengan Rust · Dioxus · Tailwind",
        "Built with Rust · Dioxus · Tailwind",
    ),
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
    ("notif_imsak_title", "Masuk waktu imsak", "It's imsak time"),
    (
        "notif_imsak_body",
        "Telah masuk waktu imsak\nBersiap menuju waktu Subuh. 🌙",
        "Imsak has started\nFajr is coming up soon. 🌙",
    ),
    // Pembaruan (toast + settings)
    ("f_update_check", "Periksa pembaruan", "Update check"),
    (
        "hint_update_check",
        "cek rilis baru di GitHub secara otomatis",
        "automatically check GitHub for new releases",
    ),
    (
        "update_check_row",
        "Cek pembaruan manual",
        "Manual update check",
    ),
    ("update_check_btn", "Cek sekarang", "Check now"),
    ("update_title", "Pembaruan tersedia", "Update available"),
    (
        "update_body",
        "Jadwal Sholat {versi} sudah rilis — klik untuk mengunduh.",
        "Jadwal Sholat {versi} is out — click to download.",
    ),
    (
        "update_latest_title",
        "Versi sudah terbaru",
        "You're up to date",
    ),
    (
        "update_latest_body",
        "Kamu memakai versi terbaru, v{versi}.",
        "You're on the latest version, v{versi}.",
    ),
    (
        "update_failed_title",
        "Gagal memeriksa pembaruan",
        "Update check failed",
    ),
    (
        "update_failed_body",
        "Tidak bisa menghubungi GitHub — periksa koneksi internet.",
        "Could not reach GitHub — check your internet connection.",
    ),
    // Layar Harian — tanggal penting & kiblat
    ("sec_dates", "Tanggal Penting", "Important Dates"),
    (
        "days_until_ramadan",
        "{n} hari menuju Ramadhan",
        "{n} days until Ramadan",
    ),
    (
        "ramadan_day",
        "Hari ke-{n} Ramadhan {year} H",
        "Day {n} of Ramadan {year} AH",
    ),
    ("in_days", "{n} hari lagi", "in {n} days"),
    ("sec_qibla", "Arah Kiblat", "Qibla Direction"),
    ("qibla_from_north", "dari utara sejati", "from true north"),
    ("dir_n", "Utara", "North"),
    ("dir_ne", "Timur Laut", "Northeast"),
    ("dir_e", "Timur", "East"),
    ("dir_se", "Tenggara", "Southeast"),
    ("dir_s", "Selatan", "South"),
    ("dir_sw", "Barat Daya", "Southwest"),
    ("dir_w", "Barat", "West"),
    ("dir_nw", "Barat Laut", "Northwest"),
    // Nama peristiwa penting kalender Islam
    ("ev_tahun_baru", "Tahun Baru Islam", "Islamic New Year"),
    ("ev_maulid", "Maulid Nabi", "Mawlid an-Nabi"),
    ("ev_isra_miraj", "Isra Mi'raj", "Isra and Mi'raj"),
    ("ev_awal_ramadan", "Awal Ramadhan", "First of Ramadan"),
    ("ev_idul_fitri", "Idul Fitri", "Eid al-Fitr"),
    ("ev_idul_adha", "Idul Adha", "Eid al-Adha"),
    // Tray — lewati adzan berikutnya
    (
        "menu_skip_next",
        "Lewati adzan berikutnya",
        "Skip next adhan",
    ),
    (
        "skip_next_toast",
        "Adzan berikutnya akan dilewati.",
        "The next adhan will be skipped.",
    ),
    (
        "skip_next_cancel_toast",
        "Lewati adzan dibatalkan.",
        "Adhan skip cancelled.",
    ),
    (
        "bell_skip_title",
        "Adzan {nama} akan dilewati — klik untuk membatalkan",
        "{nama} adhan will be skipped — click to cancel",
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
        assert!(tr(Language::En, "update_title").len() > 0);
        assert!(tr(Language::Id, "notif_imsak_title").len() > 0);
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
