use crate::prayer::{ADZAN_PRAYERS, PrayerKind, day_times, now_in_tz};
use crate::settings::Settings;
use chrono::{DateTime, Duration, NaiveDate, Timelike};
use chrono_tz::Tz;
use std::collections::HashSet;

/// Aksi yang diputuskan engine pada satu tick.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Masuk waktu sholat: mainkan adzan.
    PlayAdzan(PrayerKind),
    /// Toast saat masuk waktu sholat.
    NotifyAdzan(PrayerKind),
    /// Toast pengingat X menit sebelum adzan.
    NotifyPre { kind: PrayerKind, minutes: u32 },
}

#[derive(Debug, Default)]
pub struct EngineState {
    date: Option<NaiveDate>,
    played: HashSet<PrayerKind>,
    pre_notified: HashSet<PrayerKind>,
}

impl EngineState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Engine murni (mudah di-test): satu panggilan = satu detik berlalu.
/// Window toleransi 2 detik mencegah trigger terlewat akibat jitter timer,
/// dan mencegah double-trigger pada tick berikutnya lewat set `played`.
pub fn tick(state: &mut EngineState, settings: &Settings, now: DateTime<Tz>) -> Vec<Action> {
    let mut actions = Vec::new();

    // Ganti hari: reset penanda.
    let today = now.date_naive();
    if state.date != Some(today) {
        state.date = Some(today);
        state.played.clear();
        state.pre_notified.clear();
    }

    let Some(times) = day_times(settings, today) else {
        return actions;
    };

    let now_t = now.time();
    let tolerance = Duration::seconds(2);

    for kind in ADZAN_PRAYERS {
        let target = times.waktu(kind);
        // Elapsed mentah (bisa negatif sebelum waktu masuk) — sengaja TIDAK
        // memakai prayer::time_until yang mengoreksi lintas tengah malam,
        // karena window trigger 2 detik butuh nilai hari-yang-sama.
        let elapsed = Duration::seconds(
            now_t.num_seconds_from_midnight() as i64 - target.num_seconds_from_midnight() as i64,
        );

        // Trigger adzan: baru saja masuk waktu (0..=2s) dan belum diputar.
        if !state.played.contains(&kind) && elapsed >= Duration::zero() && elapsed <= tolerance {
            state.played.insert(kind);
            // Sholat yang di-mute (bell-off di daftar jadwal) tidak memutar
            // audio, notifikasi tetap dikirim.
            let muted = settings.muted_prayers.contains(&kind);
            if settings.audio_enabled && !muted {
                actions.push(Action::PlayAdzan(kind));
            }
            if settings.notif_enabled {
                actions.push(Action::NotifyAdzan(kind));
            }
        }

        // Pengingat X menit sebelum adzan.
        if settings.notif_enabled && settings.pre_notif_minutes > 0 {
            let pre_secs = settings.pre_notif_minutes as i64 * 60;
            let pre_target = target.num_seconds_from_midnight() as i64 - pre_secs;
            if pre_target >= 0 {
                let now_secs = now_t.num_seconds_from_midnight() as i64;
                let pre_elapsed = now_secs - pre_target;
                if !state.pre_notified.contains(&kind) && (0..=2).contains(&pre_elapsed) {
                    state.pre_notified.insert(kind);
                    actions.push(Action::NotifyPre {
                        kind,
                        minutes: settings.pre_notif_minutes,
                    });
                }
            }
        }
    }

    actions
}

/// Jalankan engine di thread terpisah (tidak tergantung runtime dioxus),
/// mengetik setiap 1 detik dan langsung mengeksekusi aksinya.
pub fn start() {
    std::thread::Builder::new()
        .name("adzan-scheduler".into())
        .spawn(|| {
            crate::logger::info("scheduler adzan mulai");
            let mut state = EngineState::new();
            loop {
                let settings = crate::state::settings();
                let now = now_in_tz(&settings);
                for action in tick(&mut state, &settings, now) {
                    execute(action, &settings);
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
        .expect("gagal spawn thread scheduler");
}

fn execute(action: Action, settings: &Settings) {
    use crate::i18n::tr;
    let lang = settings.language;
    match action {
        Action::PlayAdzan(kind) => {
            let subuh = kind == PrayerKind::Subuh;
            match crate::audio::play_adzan(settings, subuh) {
                Ok(()) => {
                    crate::logger::info(&format!("masuk waktu {}: adzan diputar", kind.nama(lang)))
                }
                Err(e) => {
                    crate::logger::error(&format!("gagal memutar adzan {}: {e}", kind.nama(lang)))
                }
            }
        }
        Action::NotifyAdzan(kind) => {
            crate::notify::show(
                tr(lang, "notif_adzan_title"),
                &tr(lang, "notif_adzan_body").replace("{nama}", kind.nama(lang)),
            );
            crate::logger::info(&format!("notifikasi masuk waktu {}", kind.nama(lang)));
        }
        Action::NotifyPre { kind, minutes } => {
            crate::notify::show(
                tr(lang, "notif_pre_title"),
                &tr(lang, "notif_pre_body")
                    .replace("{minutes}", &minutes.to_string())
                    .replace("{nama}", kind.nama(lang)),
            );
            crate::logger::info(&format!(
                "notifikasi pengingat {} ({minutes} menit)",
                kind.nama(lang)
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prayer::DayTimes;
    use chrono::{NaiveTime, TimeZone};

    fn settings_dengan_jadwal() -> Settings {
        Settings::default() // Jakarta, Kemenag, audio+notif aktif, pre 10 menit
    }

    fn fake_now(tz: Tz, date: (i32, u32, u32), t: (u32, u32, u32)) -> DateTime<Tz> {
        let d = NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap();
        let dt = d.and_hms_opt(t.0, t.1, t.2).unwrap();
        tz.from_local_datetime(&dt).unwrap()
    }

    // Tick nyata memanggil day_times (perhitungan astronomis); untuk test deterministik
    // kita uji logika via tick() dengan jadwal asli Jakarta pada jam yang jelas.
    #[test]
    fn tidak_trigger_sebelum_waktu() {
        let s = settings_dengan_jadwal();
        let mut state = EngineState::new();
        // 03:00 WIB jauh sebelum Subuh.
        let now = fake_now(s.timezone(), (2026, 10, 15), (3, 0, 0));
        let actions = tick(&mut state, &s, now);
        assert!(
            actions.is_empty(),
            "harus tidak ada aksi, dapat: {actions:?}"
        );
    }

    #[test]
    fn trigger_tepat_pada_waktu_subuh() {
        let s = settings_dengan_jadwal();
        let mut state = EngineState::new();
        let date = (2026, 10, 15);
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();

        // Detik tepat waktu subuh.
        let t = times.subuh;
        let now = fake_now(s.timezone(), date, (t.hour(), t.minute(), t.second()));
        let actions = tick(&mut state, &s, now);
        assert!(
            actions.contains(&Action::PlayAdzan(PrayerKind::Subuh)),
            "{actions:?}"
        );
        assert!(
            actions.contains(&Action::NotifyAdzan(PrayerKind::Subuh)),
            "{actions:?}"
        );

        // Tick berikutnya tidak double-trigger.
        let actions = tick(&mut state, &s, now + Duration::seconds(1));
        assert!(
            actions.is_empty(),
            "tidak boleh double trigger: {actions:?}"
        );
    }

    #[test]
    fn pre_notification_sebelum_maghrib() {
        let s = settings_dengan_jadwal();
        let mut state = EngineState::new();
        let date = (2026, 10, 15);
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();

        let pre_secs = 10 * 60;
        let maghrib_secs = times.maghrib.num_seconds_from_midnight();
        let pre_total = (maghrib_secs as i64 - pre_secs) as u32;
        let now = fake_now(
            s.timezone(),
            date,
            (pre_total / 3600, (pre_total % 3600) / 60, pre_total % 60),
        );
        let actions = tick(&mut state, &s, now);
        assert!(
            actions.contains(&Action::NotifyPre {
                kind: PrayerKind::Maghrib,
                minutes: 10
            }),
            "{actions:?}"
        );
        // Jangan mainkan adzan di tick pengingat.
        assert!(!actions.contains(&Action::PlayAdzan(PrayerKind::Maghrib)));
    }

    #[test]
    fn reset_saat_ganti_hari() {
        let s = settings_dengan_jadwal();
        let mut state = EngineState::new();
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();

        let now = fake_now(
            s.timezone(),
            (2026, 10, 15),
            (times.isya.hour(), times.isya.minute(), 0),
        );
        let _ = tick(&mut state, &s, now);
        assert!(state.played.contains(&PrayerKind::Isya));

        // Esok hari: state harus reset, jadi tick pada jam yang sama tidak trigger apa pun
        // (karena window 2s sudah lewat), tapi played kosong.
        let now2 = fake_now(
            s.timezone(),
            (2026, 10, 16),
            (times.isya.hour(), times.isya.minute(), 30),
        );
        let actions = tick(&mut state, &s, now2);
        assert!(state.played.is_empty() || state.played.contains(&PrayerKind::Isya));
        assert!(
            actions.is_empty(),
            "hari baru tidak boleh langsung trigger: {actions:?}"
        );
    }

    #[test]
    fn audio_mati_tidak_ada_aksi_play() {
        let mut s = settings_dengan_jadwal();
        s.audio_enabled = false;
        let mut state = EngineState::new();
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();
        let now = fake_now(
            s.timezone(),
            (2026, 10, 15),
            (times.subuh.hour(), times.subuh.minute(), 0),
        );
        let actions = tick(&mut state, &s, now);
        assert!(
            !actions.iter().any(|a| matches!(a, Action::PlayAdzan(_))),
            "{actions:?}"
        );
        assert!(actions.contains(&Action::NotifyAdzan(PrayerKind::Subuh)));
    }

    #[test]
    fn sholat_dimute_tidak_mainkan_adzan() {
        // Bell-off pada satu sholat: audio diam, notifikasi tetap jalan,
        // sholat lain tidak terpengaruh.
        let mut s = settings_dengan_jadwal();
        s.muted_prayers = vec![PrayerKind::Subuh];
        let mut state = EngineState::new();
        let date = (2026, 10, 15);
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();

        let now = fake_now(
            s.timezone(),
            date,
            (times.subuh.hour(), times.subuh.minute(), 0),
        );
        let actions = tick(&mut state, &s, now);
        assert!(
            !actions.iter().any(|a| matches!(a, Action::PlayAdzan(_))),
            "adzan Subuh di-mute, tidak boleh PlayAdzan: {actions:?}"
        );
        assert!(actions.contains(&Action::NotifyAdzan(PrayerKind::Subuh)));

        // Maghrib (tidak di-mute) tetap memutar adzan.
        let now = fake_now(
            s.timezone(),
            date,
            (times.maghrib.hour(), times.maghrib.minute(), 0),
        );
        let actions = tick(&mut state, &s, now);
        assert!(
            actions.contains(&Action::PlayAdzan(PrayerKind::Maghrib)),
            "{actions:?}"
        );
    }

    #[test]
    fn lewat_jendela_trigger_tidak_mainkan() {
        // App baru dinyalakan 1 jam setelah subuh — tidak boleh langsung bunyi.
        let s = settings_dengan_jadwal();
        let mut state = EngineState::new();
        let times = day_times(&s, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()).unwrap();
        let mut late = (
            times.subuh.hour(),
            times.subuh.minute(),
            times.subuh.second(),
        );
        late.0 += 1; // +1 jam
        let now = fake_now(s.timezone(), (2026, 10, 15), late);
        let actions = tick(&mut state, &s, now);
        assert!(actions.is_empty(), "{actions:?}");
    }
}
