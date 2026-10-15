use crate::settings::{CalcMethod, MadhabChoice, Settings};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime};
use salah::prelude::*;

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
    pub fn nama(&self) -> &'static str {
        match self {
            PrayerKind::Subuh => "Subuh",
            PrayerKind::Dzuhur => "Dzuhur",
            PrayerKind::Ashar => "Ashar",
            PrayerKind::Maghrib => "Maghrib",
            PrayerKind::Isya => "Isya",
        }
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

    Some(DayTimes {
        imsak: local(Prayer::Fajr, &schedule) - Duration::minutes(10),
        subuh: local(Prayer::Fajr, &schedule),
        terbit: local(Prayer::Sunrise, &schedule),
        dzuhur: local(Prayer::Dhuhr, &schedule),
        ashar: local(Prayer::Asr, &schedule),
        maghrib: local(Prayer::Maghrib, &schedule),
        isya: local(Prayer::Isha, &schedule),
        subuh_esok: local(Prayer::Fajr, &schedule_tomorrow),
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

/// Selisih `target - now` sebagai Duration (bisa negatif kalau sudah lewat).
pub fn time_until(now: NaiveTime, target: NaiveTime) -> Duration {
    let now_secs = now.num_seconds_from_midnight() as i64;
    let target_secs = target.num_seconds_from_midnight() as i64;
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

pub fn format_time(t: NaiveTime) -> String {
    t.format("%H:%M").to_string()
}

pub fn now_in_tz(settings: &Settings) -> DateTime<chrono_tz::Tz> {
    chrono::Utc::now().with_timezone(&settings.timezone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    fn jakarta() -> Settings {
        Settings::default() // default = Jakarta, Kemenag
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
}
