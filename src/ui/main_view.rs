use super::icons;
use crate::prayer::{self, DayTimes};
use crate::state;
use chrono::{Datelike, NaiveTime};
use dioxus::prelude::*;

pub(crate) const HARI: [&str; 7] = [
    "Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu",
];
pub(crate) const BULAN: [&str; 12] = [
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

/// Satu baris jadwal untuk dirender.
struct Row {
    name: &'static str,
    time: NaiveTime,
    /// Some(kind) untuk sholat wajib yang bisa jadi "berikutnya".
    kind: Option<prayer::PrayerKind>,
}

#[component]
pub fn MainView() -> Element {
    // Tick tiap detik → countdown & highlight dihitung ulang.
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });

    let _ = tick();
    let settings = state::settings();
    let now = prayer::now_in_tz(&settings);
    let times = prayer::day_times(&settings, now.date_naive());

    let lokasi = match &settings.city {
        Some(kota) => format!("{kota}, Indonesia"),
        None => format!("({:.4}, {:.4})", settings.lat, settings.lng),
    };
    let tanggal = format!(
        "{}, {} {} {}",
        HARI[now.weekday().num_days_from_monday() as usize],
        now.day(),
        BULAN[(now.month() - 1) as usize],
        now.year()
    );

    rsx! {
        div { class: "flex-1 p-5 flex flex-col gap-6 overflow-y-auto",
            {match times {
                Some(times) => rsx! {
                    Header { lokasi: lokasi.clone(), tanggal: tanggal.clone(), times: times, now: now.time() }
                    PrayerList { times: times, now: now.time() }
                },
                None => rsx! {
                    div { class: "flex-1 flex items-center justify-center text-sm text-zinc-500",
                        "Gagal menghitung jadwal. Periksa koordinat di Pengaturan."
                    }
                },
            }}
        }
    }
}

#[component]
pub(crate) fn Header(lokasi: String, tanggal: String, times: DayTimes, now: NaiveTime) -> Element {
    let (kind, target) = prayer::next_prayer(&times, now);
    let countdown = prayer::format_countdown(prayer::time_until(now, target));

    rsx! {
        div { class: "flex flex-col items-center text-center space-y-4 mt-2",
            div { class: "flex flex-col gap-1 items-center",
                div { class: "flex items-center gap-1.5 text-zinc-500 dark:text-zinc-400 text-sm font-medium",
                    icons::IconMapPin { class: "w-4 h-4" }
                    span { "{lokasi}" }
                }
                div { class: "flex items-center gap-1.5 text-zinc-400 dark:text-zinc-500 text-xs",
                    icons::IconCalendar { class: "w-3.5 h-3.5" }
                    span { "{tanggal}" }
                }
            }

            div { class: "w-full bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl py-6 px-4 flex flex-col items-center justify-center shadow-sm",
                p { class: "text-zinc-500 dark:text-zinc-400 text-sm mb-2 font-medium", "Menuju {kind.nama()}" }
                h1 { class: "text-5xl font-bold tracking-tighter text-zinc-900 dark:text-zinc-50 tabular-nums",
                    "{countdown}"
                }
            }
        }
    }
}

#[component]
fn PrayerList(times: DayTimes, now: NaiveTime) -> Element {
    let (next_kind, _) = prayer::next_prayer(&times, now);
    // Setelah Isya (menuju Subuh esok), highlight tetap di baris Isya.
    let highlight_isya = next_kind == prayer::PrayerKind::Subuh && now > times.isya;

    let rows = [
        Row {
            name: "Imsak",
            time: times.imsak,
            kind: None,
        },
        Row {
            name: "Subuh",
            time: times.subuh,
            kind: Some(prayer::PrayerKind::Subuh),
        },
        Row {
            name: "Terbit",
            time: times.terbit,
            kind: None,
        },
        Row {
            name: "Dzuhur",
            time: times.dzuhur,
            kind: Some(prayer::PrayerKind::Dzuhur),
        },
        Row {
            name: "Ashar",
            time: times.ashar,
            kind: Some(prayer::PrayerKind::Ashar),
        },
        Row {
            name: "Maghrib",
            time: times.maghrib,
            kind: Some(prayer::PrayerKind::Maghrib),
        },
        Row {
            name: "Isya",
            time: times.isya,
            kind: Some(prayer::PrayerKind::Isya),
        },
    ];

    rsx! {
        div { class: "flex flex-col gap-2",
            for row in rows {
                {
                    let is_next = match row.kind {
                        Some(k) => {
                            (k == next_kind && !highlight_isya)
                                || (row.kind == Some(prayer::PrayerKind::Isya) && highlight_isya)
                        }
                        None => false,
                    };
                    let row_class = if is_next {
                        "flex items-center justify-between p-3 rounded-xl transition-all duration-200 bg-zinc-900 text-zinc-50 dark:bg-zinc-100 dark:text-zinc-900 shadow-md ring-1 ring-zinc-900/5 dark:ring-white/10 scale-[1.02]"
                    } else {
                        "flex items-center justify-between p-3 rounded-xl transition-all duration-200 bg-transparent text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-900/50"
                    };
                    let name_class = if is_next { "text-sm font-semibold" } else { "text-sm font-medium" };
                    let time_class = if is_next {
                        "text-sm tabular-nums tracking-tight font-bold"
                    } else {
                        "text-sm tabular-nums tracking-tight font-medium text-zinc-500 dark:text-zinc-400"
                    };
                    let bell_wrap_class = if is_next {
                        "p-1 rounded-full hover:bg-zinc-800 dark:hover:bg-zinc-200 cursor-default"
                    } else {
                        "p-1 rounded-full hover:bg-zinc-200 dark:hover:bg-zinc-800 cursor-default"
                    };
                    let bell_class = if is_next {
                        "w-3.5 h-3.5 text-zinc-300 dark:text-zinc-600"
                    } else {
                        "w-3.5 h-3.5 text-zinc-400 dark:text-zinc-500"
                    };
                    rsx! {
                        div { key: "{row.name}", class: "{row_class}",
                            span { class: "{name_class}", "{row.name}" }
                            div { class: "flex items-center gap-3",
                                span { class: "{time_class}", "{prayer::format_time(row.time)}" }
                                span { class: "{bell_wrap_class}",
                                    icons::IconBell { class: bell_class }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
