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

/// Nama sholat sesuai konteks tanggal — Dzuhur pada Jumat tampil sebagai
/// "Sholat Jumat" (konvensi imsakiyah; crate `salah` melakukan hal serupa).
pub fn nama_hari_ini(
    kind: PrayerKind,
    lang: crate::settings::Language,
    tanggal: NaiveDate,
) -> &'static str {
    use chrono::Datelike as _;
    if kind == PrayerKind::Dzuhur && tanggal.weekday() == chrono::Weekday::Fri {
        crate::i18n::tr(lang, "sholat_jumat")
    } else {
        kind.nama(lang)
    }
}

/// Jadwal satu hari (waktu lokal kota, `NaiveTime`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayTimes {
    /// Imsak: 10 menit sebelum Subuh (konvensi jadwal Kemenag).
    pub imsak: NaiveTime,
    pub subuh: NaiveTime,
    pub terbit: NaiveTime,
    /// Dhuha: 20 menit setelah Terbit (konvensi imsakiyah).
    pub dhuha: NaiveTime,
    pub dzuhur: NaiveTime,
    pub ashar: NaiveTime,
    pub maghrib: NaiveTime,
    pub isya: NaiveTime,
    /// Tengah malam syar'i: titik tengah interval Maghrib → Subuh esok.
    pub tengah_malam: NaiveTime,
    /// Sepertiga malam akhir: ⅔ interval Maghrib → Subuh esok (waktu qiyam).
    pub sepertiga_malam: NaiveTime,
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
    let terbit =
        local(Prayer::Sunrise, &schedule) + Duration::minutes(settings.offset_terbit() as i64);
    let maghrib = geser(PrayerKind::Maghrib, local(Prayer::Maghrib, &schedule));
    let subuh_esok = geser(PrayerKind::Subuh, local(Prayer::Fajr, &schedule_tomorrow));

    Some(DayTimes {
        imsak: subuh - Duration::minutes(10),
        subuh,
        terbit,
        // Dhuha mengikuti Terbit (memang tanpa offset). Tengah malam &
        // sepertiga malam dihitung dari Maghrib & Subuh esok terkoreksi
        // agar konsisten dengan jadwal yang tampil di layar.
        dhuha: terbit + Duration::minutes(20),
        dzuhur: geser(PrayerKind::Dzuhur, local(Prayer::Dhuhr, &schedule)),
        ashar: geser(PrayerKind::Ashar, local(Prayer::Asr, &schedule)),
        maghrib,
        isya: geser(PrayerKind::Isya, local(Prayer::Isha, &schedule)),
        tengah_malam: bagi_malam(maghrib, subuh_esok, 0.5),
        sepertiga_malam: bagi_malam(maghrib, subuh_esok, 2.0 / 3.0),
        subuh_esok,
    })
}

/// Bagi malam Maghrib → Subuh esok pada proporsi `porsi` (0.5 = tengah
/// malam syar'i, 2.0/3.0 = sepertiga malam akhir / waktu qiyam). Rumus sama
/// dengan internal crate `salah` (`calculate_qiyam`; field-nya tidak
/// diekspos publik). Hasil dibulatkan ke menit terdekat.
fn bagi_malam(maghrib: NaiveTime, subuh_esok: NaiveTime, porsi: f64) -> NaiveTime {
    let malam = time_until(maghrib, subuh_esok).num_seconds() as f64;
    let total = maghrib.num_seconds_from_midnight() as f64 + malam * porsi;
    let bulat = (((total / 60.0).round() as i64) * 60).rem_euclid(86_400) as u32;
    NaiveTime::from_num_seconds_from_midnight_opt(bulat, 0).expect("detik dalam sehari")
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

/// Target countdown harian: sholat biasa, atau imsak saat Ramadan.
///
/// Sengaja belum terhubung UI: label countdown spesial Ramadan ("Menuju
/// Imsak") dikeluarkan dari desain v1.4 — fungsi disiapkan untuk
/// menghidupkannya kembali di rilis berikutnya.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetHarian {
    /// Sholat berikutnya (perilaku non-Ramadan; Maghrib saat Ramadan
    /// tetap varian ini — label "Menuju Berbuka" diganti oleh UI).
    Sholat(PrayerKind, NaiveTime),
    /// Menuju imsak (pra-Subuh saat Ramadan, atau setelah Isya → imsak esok).
    Imsak(NaiveTime),
}

/// Target countdown dengan penyesuaian Ramadan:
/// - pra-imsak (`now < imsak`) → imsak hari ini (lintas tengah malam pun
///   benar karena `DayTimes` sudah rolover ke hari baru);
/// - antara imsak dan Isya → perilaku normal (`next_prayer`);
/// - setelah Isya → imsak esok (Subuh esok − 10 menit, konvensi Kemenag).
#[allow(dead_code)]
pub fn target_countdown(ramadan: bool, times: &DayTimes, now: NaiveTime) -> TargetHarian {
    let (kind, target) = next_prayer(times, now);
    if !ramadan {
        return TargetHarian::Sholat(kind, target);
    }
    match kind {
        PrayerKind::Subuh if now < times.imsak => TargetHarian::Imsak(times.imsak),
        PrayerKind::Subuh if now >= times.isya => {
            TargetHarian::Imsak(times.subuh_esok - Duration::minutes(10))
        }
        _ => TargetHarian::Sholat(kind, target),
    }
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

/// Format jam sesuai setting: 24 jam ("04:11") atau 12 jam ("04:11 AM").
/// Keduanya berawalan nol — lebar teks konstan agar kolom jam (dan lonceng
/// di sebelahnya) rata di papan jadwal.
pub fn format_time(t: NaiveTime, h24: bool) -> String {
    if h24 {
        t.format("%H:%M").to_string()
    } else {
        t.format("%I:%M %p").to_string()
    }
}

/// Tanggal masehi "efektif" untuk konteks HIJRIAH: hari hijriah D berjalan
/// dari Maghrib D−1 sampai Maghrib D, jadi begitu lewat Maghrib hari ini
/// tanggal hijriah sudah milik besok. Dipakai untuk TAMPILAN — label
/// tanggal hijriah, badge Ramadan, pil puasa sunnah, kartu Tanggal
/// Penting — bukan untuk trigger adzan/imsak yang tetap berbasis tanggal
/// masehi (puasa Senin tetap puasa siang Senin masehi).
pub fn tanggal_efektif_hijriah(times: &DayTimes, now: NaiveTime, today: NaiveDate) -> NaiveDate {
    if now >= times.maghrib {
        today.succ_opt().unwrap_or(today)
    } else {
        today
    }
}

/// Waktu "sekarang" dalam zona waktu kota.
///
/// Untuk testing, dua env var (bisa dipakai sendiri-sendiri atau digabung):
/// - `JADWAL_FAKE_TIME=HH:MM[:SS]` — jam internal digeser ke waktu tersebut
///   (tetap berjalan/berdetak), tanggal tetap hari asli;
/// - `JADWAL_FAKE_DATE=YYYY-MM-DD` — tanggal dipalsukan (mis. untuk uji
///   mode Ramadan: `2027-02-12`), jam mengikuti jam asli (atau
///   `JADWAL_FAKE_TIME` bila juga disetel).
///
/// Offset dihitung sekali saat pemanggilan pertama. Tanpa env var = jam
/// asli. Karena cuma offset Duration, seluruh tampilan (countdown, badge
/// Ramadan, kalender, penanda hari ini) ikut bergeser konsisten.
pub fn now_in_tz(settings: &Settings) -> DateTime<chrono_tz::Tz> {
    let real = chrono::Utc::now().with_timezone(&settings.timezone());
    let offset = *FAKE_OFFSET.get_or_init(|| {
        let fake_jam = std::env::var("JADWAL_FAKE_TIME").ok().and_then(|raw| {
            NaiveTime::parse_from_str(&raw, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(&raw, "%H:%M"))
                .ok()
        });
        let fake_tanggal = std::env::var("JADWAL_FAKE_DATE")
            .ok()
            .and_then(|raw| NaiveDate::parse_from_str(&raw, "%Y-%m-%d").ok());

        let geser = |tgl: NaiveDate, jam: NaiveTime| -> Option<Duration> {
            let dt = real
                .timezone()
                .from_local_datetime(&tgl.and_time(jam))
                .single()?;
            Some(dt - real)
        };

        let offset = match (fake_tanggal, fake_jam) {
            // Tanggal palsu + (jam asli atau jam palsu).
            (Some(tgl), jam) => geser(tgl, jam.unwrap_or(real.time())),
            // Hanya jam palsu — perilaku lama.
            (None, Some(jam)) => geser(real.date_naive(), jam),
            (None, None) => None,
        };
        match (fake_tanggal, fake_jam, offset) {
            (Some(t), Some(j), Some(o)) => {
                crate::logger::info(&format!(
                    "JADWAL_FAKE_DATE={t} + JADWAL_FAKE_TIME aktif — jam palsu {j}, offset {o}"
                ));
                Some(o)
            }
            (Some(t), None, Some(o)) => {
                crate::logger::info(&format!(
                    "JADWAL_FAKE_DATE={t} aktif — tanggal dipalsukan, offset {o}"
                ));
                Some(o)
            }
            (None, Some(_), Some(o)) => {
                crate::logger::info(&format!("JADWAL_FAKE_TIME aktif — jam digeser {o}"));
                Some(o)
            }
            _ => {
                if fake_tanggal.is_some() || fake_jam.is_some() {
                    crate::logger::error("env JADWAL_FAKE_* tak terbaca — jam asli dipakai");
                }
                None
            }
        }
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
        // 12 jam tetap berawalan nol agar lebar kolom jam konstan
        // (lonceng di papan jadwal tetap rata).
        assert_eq!(format_time(sore, false), "04:05 PM");
        let dini_hari = NaiveTime::from_hms_opt(0, 11, 0).unwrap();
        assert_eq!(format_time(dini_hari, false), "12:11 AM");
    }

    #[test]
    fn tanggal_efektif_hijriah_berganti_di_maghrib() {
        let s = jakarta();
        let date = NaiveDate::from_ymd_opt(2026, 10, 23).unwrap(); // Jumat
        let besok = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        let t = day_times(&s, date).expect("perhitungan berhasil");

        let hm = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        // Sebelum Maghrib — termasuk dini hari — masih tanggal hari ini.
        assert_eq!(tanggal_efektif_hijriah(&t, hm(1, 0), date), date);
        assert_eq!(
            tanggal_efektif_hijriah(&t, t.maghrib - chrono::Duration::minutes(1), date),
            date
        );
        // Tepat & setelah Maghrib — hari hijriah sudah milik besok.
        assert_eq!(tanggal_efektif_hijriah(&t, t.maghrib, date), besok);
        assert_eq!(tanggal_efektif_hijriah(&t, hm(23, 30), date), besok);
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
    fn offset_terbit_menggeser_terbit_dan_dhuha() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let mut s = jakarta();
        s.terbit_offset = 5;
        let murni = day_times(&jakarta(), date).unwrap();
        let geser = day_times(&s, date).unwrap();

        let menit = |a: NaiveTime, b: NaiveTime| -> i64 {
            (a.num_seconds_from_midnight() as i64 - b.num_seconds_from_midnight() as i64) / 60
        };
        assert_eq!(menit(geser.terbit, murni.terbit), 5);
        // Dhuha mengikuti Terbit.
        assert_eq!(menit(geser.dhuha, murni.dhuha), 5);
        // Sholat & imsak tidak terpengaruh.
        assert_eq!(geser.subuh, murni.subuh);
        assert_eq!(geser.imsak, murni.imsak);
        assert_eq!(geser.dzuhur, murni.dzuhur);
    }

    #[test]
    fn dzuhur_jumat_berlabel_sholat_jumat() {
        let s = jakarta();
        // 2027-02-12 = Jumat; 13-an = Sabtu.
        let jumat = NaiveDate::from_ymd_opt(2027, 2, 12).unwrap();
        let sabtu = NaiveDate::from_ymd_opt(2027, 2, 13).unwrap();

        assert_eq!(
            nama_hari_ini(PrayerKind::Dzuhur, s.language, jumat),
            "Sholat Jumat"
        );
        assert_eq!(
            nama_hari_ini(PrayerKind::Dzuhur, s.language, sabtu),
            "Dzuhur"
        );
        // Sholat lain tak berubah di Jumat.
        assert_eq!(nama_hari_ini(PrayerKind::Ashar, s.language, jumat), "Ashar");
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

    #[test]
    fn dhuha_dan_bagi_malam_mengikuti_jadwal() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&jakarta(), date).unwrap();

        // Dhuha = terbit + 20 menit.
        assert_eq!(t.dhuha, t.terbit + Duration::minutes(20));

        // Tengah malam & sepertiga malam berada di dalam malam
        // (maghrib → subuh esok), berurutan, dan dibulatkan ke menit.
        // Pakai time_until agar urutan tetap benar lintas tengah malam.
        let malam = time_until(t.maghrib, t.subuh_esok);
        let setengah = time_until(t.maghrib, t.tengah_malam);
        let sepertiga = time_until(t.maghrib, t.sepertiga_malam);
        assert!(setengah < sepertiga && sepertiga < malam);
        assert_eq!(t.tengah_malam.second(), 0);
        assert_eq!(t.sepertiga_malam.second(), 0);
    }

    #[test]
    fn bagi_malam_rumus_dan_pembulatan() {
        let hm = |h: u32, m: u32, s: u32| NaiveTime::from_hms_opt(h, m, s).unwrap();
        // Malam 17:46:20 → 04:13:40 = 10j27m20s. Tengah = +5j13m40s = 23:00:00;
        // ⅔ = +6j58m13,3s = 00:44:33,3 → dibulatkan 00:45 (lintas tengah malam).
        let tengah = bagi_malam(hm(17, 46, 20), hm(4, 13, 40), 0.5);
        let sepertiga = bagi_malam(hm(17, 46, 20), hm(4, 13, 40), 2.0 / 3.0);
        assert_eq!(tengah, hm(23, 0, 0));
        assert_eq!(sepertiga, hm(0, 45, 0));
    }

    #[test]
    fn target_countdown_non_ramadan_beperilaku_lama() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&jakarta(), date).unwrap();
        let hm = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();

        let (kind, target) = match target_countdown(false, &t, hm(10, 0)) {
            TargetHarian::Sholat(kind, target) => (kind, target),
            TargetHarian::Imsak(_) => panic!("bukan imsak"),
        };
        assert_eq!(kind, PrayerKind::Dzuhur);
        assert_eq!(target, t.dzuhur);
    }

    #[test]
    fn target_countdown_ramadan_menuju_imsak() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let t = day_times(&jakarta(), date).unwrap();
        let imsak_esok = t.subuh_esok - Duration::minutes(10);

        // Pra-imsak pagi ini.
        let pra_imsak = t.imsak - Duration::minutes(60);
        assert_eq!(
            target_countdown(true, &t, pra_imsak),
            TargetHarian::Imsak(t.imsak)
        );

        // Lintas tengah malam (00:30) tetap menuju imsak hari itu.
        let dini = NaiveTime::from_hms_opt(0, 30, 0).unwrap();
        assert_eq!(
            target_countdown(true, &t, dini),
            TargetHarian::Imsak(t.imsak)
        );

        // Jendela imsak–subuh: tetap tampil Subuh (bukan loncat ke imsak esok).
        let antara = t.imsak + Duration::minutes(5);
        assert_eq!(
            target_countdown(true, &t, antara),
            TargetHarian::Sholat(PrayerKind::Subuh, t.subuh)
        );

        // Setelah Isya → imsak esok (subuh esok − 10 menit).
        let larut = NaiveTime::from_hms_opt(23, 30, 0).unwrap();
        assert_eq!(
            target_countdown(true, &t, larut),
            TargetHarian::Imsak(imsak_esok)
        );

        // Siang hari (antara Ashar–Maghrib) → Maghrib tetap varian sholat
        // (label berbuka oleh UI).
        assert_eq!(
            target_countdown(true, &t, NaiveTime::from_hms_opt(16, 0, 0).unwrap()),
            TargetHarian::Sholat(PrayerKind::Maghrib, t.maghrib)
        );
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
