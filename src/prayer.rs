use crate::settings::{CalcMethod, MadhabChoice, Settings};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime};
use salah::prelude::*;
use std::sync::OnceLock;

/// Offset jam palsu (dari env JADWAL_FAKE_TIME) — dihitung sekali.
static FAKE_OFFSET: OnceLock<Option<Duration>> = OnceLock::new();

/// Lima sholat wajib yang memicu adzan (urutan hari).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrayerKind {
    Subuh,
    Dzuhur,
    Ashar,
    Maghrib,
    Isya,
}

pub const ADZAN_PRAYERS: [PrayerKind; 5] = [
    PrayerKind::Subuh,
    PrayerKind::Dzuhur,
    PrayerKind::Ashar,
    PrayerKind::Maghrib,
    PrayerKind::Isya,
];

impl PrayerKind {
    /// Kunci terjemahan nama sholat (lihat `i18n::PHRASES`).
    pub fn key(&self) -> &'static str {
        match self {
            PrayerKind::Subuh => "subuh",
            PrayerKind::Dzuhur => "dzuhur",
            PrayerKind::Ashar => "ashar",
            PrayerKind::Maghrib => "maghrib",
            PrayerKind::Isya => "isya",
        }
    }

    /// Nama sholat sesuai bahasa UI.
    pub fn nama(&self, lang: crate::settings::Language) -> &'static str {
        crate::i18n::tr(lang, self.key())
    }
}

/// Jadwal satu hari (waktu lokal kota, `NaiveTime`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayTimes {
    /// Imsak: 10 menit sebelum Subuh (konvensi jadwal Kemenag).
    pub imsak: NaiveTime,
    pub subuh: NaiveTime,
    pub terbit: NaiveTime,
    pub dzuhur: NaiveTime,
    pub ashar: NaiveTime,
    pub maghrib: NaiveTime,
    pub isya: NaiveTime,
    /// Subuh esok hari — dipakai untuk countdown setelah Isya.
    pub subuh_esok: NaiveTime,
}

impl DayTimes {
    pub fn waktu(&self, kind: PrayerKind) -> NaiveTime {
        match kind {
            PrayerKind::Subuh => self.subuh,
            PrayerKind::Dzuhur => self.dzuhur,
            PrayerKind::Ashar => self.ashar,
            PrayerKind::Maghrib => self.maghrib,
            PrayerKind::Isya => self.isya,
        }
    }
}

pub fn build_parameters(method: CalcMethod, madhab: MadhabChoice) -> Parameters {
    let salah_method = match method {
        CalcMethod::Kemenag => Method::Singapore, // Subuh 20°, Isya 18° = metode Kemenag
        CalcMethod::MuslimWorldLeague => Method::MuslimWorldLeague,
        CalcMethod::Egyptian => Method::Egyptian,
        CalcMethod::Karachi => Method::Karachi,
        CalcMethod::UmmAlQura => Method::UmmAlQura,
        CalcMethod::NorthAmerica => Method::NorthAmerica,
    };
    let madhab = match madhab {
        MadhabChoice::Shafi => Madhab::Shafi,
        MadhabChoice::Hanafi => Madhab::Hanafi,
    };
    Configuration::with(salah_method, madhab)
}

/// Hitung jadwal satu hari untuk lokasi & metode pada settings.
pub fn day_times(settings: &Settings, date: NaiveDate) -> Option<DayTimes> {
    let tz = settings.timezone();
    let coords = Coordinates::new(settings.lat, settings.lng);
    let params = build_parameters(settings.method, settings.madhab);

    let schedule = PrayerSchedule::new()
        .on(date)
        .for_location(coords)
        .with_configuration(params)
        .calculate()
        .ok()?;

    let tomorrow = date.succ_opt()?;
    let schedule_tomorrow = PrayerSchedule::new()
        .on(tomorrow)
        .for_location(coords)
        .with_configuration(params)
        .calculate()
        .ok()?;

    let local =
        |t: Prayer, sched: &PrayerTimes| -> NaiveTime { sched.time(t).with_timezone(&tz).time() };
    // Koreksi manual per sholat (settings → masjid setempat). Imsak mengikuti
    // Subuh yang sudah terkoreksi; subuh_esok juga, agar countdown lintas
    // tengah malam memakai jadwal yang sama dengan trigger adzan.
    let geser = |kind: PrayerKind, t: NaiveTime| -> NaiveTime {
        t + Duration::minutes(settings.offset_of(kind) as i64)
    };
    let subuh = geser(PrayerKind::Subuh, local(Prayer::Fajr, &schedule));

    Some(DayTimes {
        imsak: subuh - Duration::minutes(10),
        subuh,
        terbit: local(Prayer::Sunrise, &schedule),
        dzuhur: geser(PrayerKind::Dzuhur, local(Prayer::Dhuhr, &schedule)),
        ashar: geser(PrayerKind::Ashar, local(Prayer::Asr, &schedule)),
        maghrib: geser(PrayerKind::Maghrib, local(Prayer::Maghrib, &schedule)),
        isya: geser(PrayerKind::Isya, local(Prayer::Isha, &schedule)),
        subuh_esok: geser(PrayerKind::Subuh, local(Prayer::Fajr, &schedule_tomorrow)),
    })
}

/// Sholat berikutnya dari `now`; kembalikan `(nama, waktu target)`.
/// Setelah Isya, target adalah Subuh esok hari.
pub fn next_prayer(times: &DayTimes, now: NaiveTime) -> (PrayerKind, NaiveTime) {
    for kind in ADZAN_PRAYERS {
        let t = times.waktu(kind);
        if t > now {
            return (kind, t);
        }
    }
    (PrayerKind::Subuh, times.subuh_esok)
}

/// Selisih `target - now` sebagai Duration untuk COUNTDOWN.
/// Kalau target < now, target dianggap jatuh esok hari (lintas tengah
/// malam) dan ditambah 24 jam — sehingga countdown ke Subuh esok setelah
/// Isya selalu positif, bukan mentok 00:00:00.
pub fn time_until(now: NaiveTime, target: NaiveTime) -> Duration {
    let now_secs = now.num_seconds_from_midnight() as i64;
    let mut target_secs = target.num_seconds_from_midnight() as i64;
    if target_secs < now_secs {
        target_secs += 86_400;
    }
    Duration::seconds(target_secs - now_secs)
}

/// Format Duration menjadi HH:MM:SS (tanpa tanda negatif).
pub fn format_countdown(d: Duration) -> String {
    let secs = d.num_seconds().max(0);
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Format jam sesuai setting: 24 jam ("04:11") atau 12 jam ("4:11 AM").
pub fn format_time(t: NaiveTime, h24: bool) -> String {
    if h24 {
        t.format("%H:%M").to_string()
    } else {
        t.format("%-I:%M %p").to_string()
    }
}

/// Waktu "sekarang" dalam zona waktu kota.
///
/// Untuk testing: set env `JADWAL_FAKE_TIME=HH:MM[:SS]` saat menjalankan app
/// agar jam internal digeser ke waktu tersebut (tetap berjalan/berdetak).
/// Offset dihitung sekali saat pemanggilan pertama. Tanpa env var = jam asli.
pub fn now_in_tz(settings: &Settings) -> DateTime<chrono_tz::Tz> {
    let real = chrono::Utc::now().with_timezone(&settings.timezone());
    let offset = *FAKE_OFFSET.get_or_init(|| {
        std::env::var("JADWAL_FAKE_TIME")
            .ok()
            .and_then(|raw| {
                let fake = NaiveTime::parse_from_str(&raw, "%H:%M:%S")
                    .or_else(|_| NaiveTime::parse_from_str(&raw, "%H:%M"))
                    .ok()?;
                let fake_dt = real
                    .timezone()
                    .from_local_datetime(&real.date_naive().and_time(fake))
                    .single()?;
                Some(fake_dt - real)
            })
            .inspect(|o| {
                crate::logger::info(&format!("JADWAL_FAKE_TIME aktif — jam digeser {o}"));
            })
    });
    match offset {
        Some(o) => real + o,
        None => real,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    fn jakarta() -> Settings {
        Settings::default() // default = Jakarta, Kemenag
    }

    #[test]
    fn format_time_24_dan_12_jam() {
        let sore = NaiveTime::from_hms_opt(16, 5, 0).unwrap();
        assert_eq!(format_time(sore, true), "16:05");
        assert_eq!(format_time(sore, false), "4:05 PM");
        let dini_hari = NaiveTime::from_hms_opt(0, 11, 0).unwrap();
        assert_eq!(format_time(dini_hari, false), "12:11 AM");
    }

    // Referensi kalibrasi: Aladhan API method=20 (Kemenag) untuk Jakarta
    // 15-10-2026: Subuh 04:13, Terbit 05:31, Dzuhur 11:38, Ashar 14:45,
    // Maghrib 17:46, Isya 18:56. Range test ±~7 menit dari referensi.
    #[test]
    fn jadwal_jakarta_cocok_referensi_kemenag() {
        let s = jakarta();
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&s, date).expect("perhitungan berhasil");

        let hm = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        assert!(
            t.subuh > hm(4, 5) && t.subuh < hm(4, 22),
            "subuh: {:?}",
            t.subuh
        );
        assert!(
            t.terbit > hm(5, 25) && t.terbit < hm(5, 40),
            "terbit: {:?}",
            t.terbit
        );
        assert!(
            t.dzuhur > hm(11, 32) && t.dzuhur < hm(11, 47),
            "dzuhur: {:?}",
            t.dzuhur
        );
        assert!(
            t.ashar > hm(14, 38) && t.ashar < hm(14, 53),
            "ashar: {:?}",
            t.ashar
        );
        assert!(
            t.maghrib > hm(17, 40) && t.maghrib < hm(17, 55),
            "maghrib: {:?}",
            t.maghrib
        );
        assert!(
            t.isya > hm(18, 50) && t.isya < hm(19, 5),
            "isya: {:?}",
            t.isya
        );

        // Urutan harus monoton.
        assert!(t.subuh < t.terbit && t.terbit < t.dzuhur);
        assert!(t.dzuhur < t.ashar && t.ashar < t.maghrib && t.maghrib < t.isya);
    }

    #[test]
    fn metode_kemenag_subuh_lebih_awal_dari_mwl() {
        // Kemenag (Subuh 20°, Isya 18°) vs MWL (Subuh 18°, Isya 17°):
        // Subuh Kemenag lebih awal, Isya Kemenag lebih lambat.
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let mut kemenag = jakarta();
        kemenag.method = CalcMethod::Kemenag;
        let mut mwl = jakarta();
        mwl.method = CalcMethod::MuslimWorldLeague;
        let a = day_times(&kemenag, date).unwrap();
        let b = day_times(&mwl, date).unwrap();
        assert!(
            a.subuh < b.subuh,
            "Subuh Kemenag ({:?}) harus < MWL ({:?})",
            a.subuh,
            b.subuh
        );
        assert!(
            a.isya > b.isya,
            "Isya Kemenag ({:?}) harus > MWL ({:?})",
            a.isya,
            b.isya
        );
    }

    #[test]
    fn next_prayer_siang_hari() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&jakarta(), date).unwrap();
        let pukul = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        let (kind, _) = next_prayer(&t, pukul(10, 0));
        assert_eq!(kind, PrayerKind::Dzuhur);
    }

    #[test]
    fn offset_menggeser_semua_konsumen_jadwal() {
        // Koreksi +3 menit Maghrib & −2 menit Subuh harus terlihat di
        // DayTimes (scheduler, countdown, widget semuanya baca field ini).
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let mut s = jakarta();
        s.prayer_offsets.insert(PrayerKind::Maghrib, 3);
        s.prayer_offsets.insert(PrayerKind::Subuh, -2);
        let murni = day_times(&jakarta(), date).unwrap();
        let dikoreksi = day_times(&s, date).unwrap();

        let menit = |a: NaiveTime, b: NaiveTime| -> i64 {
            (a.num_seconds_from_midnight() as i64 - b.num_seconds_from_midnight() as i64) / 60
        };
        assert_eq!(menit(dikoreksi.maghrib, murni.maghrib), 3);
        assert_eq!(menit(dikoreksi.subuh, murni.subuh), -2);
        assert_eq!(menit(dikoreksi.subuh_esok, murni.subuh_esok), -2);
        // Imsak mengikuti Subuh terkoreksi, tetap 10 menit sebelumnya.
        assert_eq!(menit(dikoreksi.imsak, dikoreksi.subuh), -10);
        // Terbit & sholat tanpa offset tidak bergeser.
        assert_eq!(dikoreksi.terbit, murni.terbit);
        assert_eq!(dikoreksi.dzuhur, murni.dzuhur);
    }

    #[test]
    fn next_prayer_setelah_isya_menuju_subuh_esok() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&jakarta(), date).unwrap();
        let pukul = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        let (kind, target) = next_prayer(&t, pukul(23, 30));
        assert_eq!(kind, PrayerKind::Subuh);
        assert_eq!(target, t.subuh_esok);
    }

    #[test]
    fn countdown_format() {
        assert_eq!(format_countdown(Duration::seconds(3661)), "01:01:01");
        assert_eq!(format_countdown(Duration::seconds(-5)), "00:00:00");
    }

    #[test]
    fn countdown_lintas_tengah_malam_positif() {
        // Regresi: setelah Isya, countdown ke Subuh esok mentok 00:00:00
        // karena selisih detik-dalam-sehari negatif tanpa koreksi +24 jam.
        let hm = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();

        // 23:30 -> 04:14 esok = 4 jam 44 menit.
        let d = time_until(hm(23, 30), hm(4, 14));
        assert_eq!(d, Duration::seconds(17_040));
        assert_eq!(format_countdown(d), "04:44:00");

        // Countdown sama-hari tetap benar.
        assert_eq!(time_until(hm(10, 0), hm(11, 58)), Duration::seconds(7_080));

        // Tepat pada waktunya = nol.
        assert_eq!(time_until(hm(17, 50), hm(17, 50)), Duration::zero());
    }

    /// Helper debug: cetak jadwal hari ini memakai settings bawaan (Jakarta,
    /// tanpa offset). Jalankan:
    /// `cargo test debug_cetak_jadwal -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn debug_cetak_jadwal_hari_ini() {
        let s = Settings::default();
        let today = chrono::Utc::now().with_timezone(&s.timezone()).date_naive();
        if let Some(t) = day_times(&s, today) {
            println!(
                "jadwal {today} (kota {:?}, offset aktif: {:?}):",
                s.city, s.prayer_offsets
            );
            for kind in ADZAN_PRAYERS {
                println!("  {:8} = {}", kind.nama(s.language), t.waktu(kind));
            }
            println!("  imsak    = {}", t.imsak);
        }
    }
}
