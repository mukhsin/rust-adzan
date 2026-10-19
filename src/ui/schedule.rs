//! Komponen jadwal sholat bersama: header (lokasi, tanggal, countdown) dan
//! daftar jadwal lengkap. Dipakai widget desktop (mode kecil hanya header,
//! mode besar header + daftar); window utama kini khusus pengaturan.

use super::icons;
use crate::i18n::tr;
use crate::prayer::{self, DayTimes};
use crate::state;
use chrono::NaiveTime;
use dioxus::prelude::*;

/// Satu baris jadwal untuk dirender.
struct Row {
    /// Kunci terjemahan nama (lihat `i18n::PHRASES`).
    key: &'static str,
    time: NaiveTime,
    /// Some(kind) untuk sholat wajib yang bisa jadi "berikutnya".
    kind: Option<prayer::PrayerKind>,
}

#[component]
pub(crate) fn Header(lokasi: String, tanggal: String, times: DayTimes, now: NaiveTime) -> Element {
    let lang = state::settings().language;
    let (kind, target) = prayer::next_prayer(&times, now);
    let countdown = prayer::format_countdown(prayer::time_until(now, target));
    let menuju = format!("{} {}", tr(lang, "countdown_prefix"), kind.nama(lang));

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
                p { class: "text-zinc-500 dark:text-zinc-400 text-sm mb-2 font-medium",
                    "{menuju}"
                }
                h1 { class: "text-5xl font-bold tracking-tighter text-zinc-900 dark:text-zinc-50 tabular-nums",
                    "{countdown}"
                }
            }
        }
    }
}

#[component]
pub(crate) fn PrayerList(times: DayTimes, now: NaiveTime) -> Element {
    let lang = state::settings().language;
    let settings = state::settings();
    let (next_kind, _) = prayer::next_prayer(&times, now);
    // Setelah Isya (menuju Subuh esok), highlight tetap di baris Isya.
    let highlight_isya = next_kind == prayer::PrayerKind::Subuh && now > times.isya;

    let rows = [
        Row {
            key: "imsak",
            time: times.imsak,
            kind: None,
        },
        Row {
            key: "subuh",
            time: times.subuh,
            kind: Some(prayer::PrayerKind::Subuh),
        },
        Row {
            key: "terbit",
            time: times.terbit,
            kind: None,
        },
        Row {
            key: "dzuhur",
            time: times.dzuhur,
            kind: Some(prayer::PrayerKind::Dzuhur),
        },
        Row {
            key: "ashar",
            time: times.ashar,
            kind: Some(prayer::PrayerKind::Ashar),
        },
        Row {
            key: "maghrib",
            time: times.maghrib,
            kind: Some(prayer::PrayerKind::Maghrib),
        },
        Row {
            key: "isya",
            time: times.isya,
            kind: Some(prayer::PrayerKind::Isya),
        },
    ];

    rsx! {
        div { class: "flex flex-col gap-1.5",
            for row in rows {
                {
                    let is_next = match row.kind {
                        Some(k) => {
                            (k == next_kind && !highlight_isya)
                                || (row.kind == Some(prayer::PrayerKind::Isya) && highlight_isya)
                        }
                        None => false,
                    };
                    let nama = tr(lang, row.key);
                    // Ringkas: list hanya tampil di widget (lebar 300px).
                    let row_class = if is_next {
                        "flex items-center justify-between px-3 py-2 rounded-md transition-all duration-200 bg-zinc-900 text-zinc-50 dark:bg-zinc-100 dark:text-zinc-900 shadow-md ring-1 ring-zinc-900/5 dark:ring-white/10"
                    } else {
                        "flex items-center justify-between px-3 py-2 rounded-md transition-all duration-200 bg-transparent text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-900/50"
                    };
                    let name_class = if is_next { "text-sm font-semibold" } else { "text-sm font-medium" };
                    let time_class = if is_next {
                        "text-sm tabular-nums tracking-tight font-bold"
                    } else {
                        "text-sm tabular-nums tracking-tight font-medium text-zinc-500 dark:text-zinc-400"
                    };
                    let bell_wrap_class = if is_next {
                        "p-1 rounded-full hover:bg-zinc-800 dark:hover:bg-zinc-200 cursor-pointer"
                    } else {
                        "p-1 rounded-full hover:bg-zinc-200 dark:hover:bg-zinc-800 cursor-pointer"
                    };
                    let bell_class = if is_next {
                        "w-3.5 h-3.5 text-zinc-300 dark:text-zinc-600"
                    } else {
                        "w-3.5 h-3.5 text-zinc-400 dark:text-zinc-500"
                    };
                    // Adzan sholat ini di-mute? (bell-off; scheduler tidak memutar audionya)
                    let muted = row
                        .kind
                        .is_some_and(|k| settings.muted_prayers.contains(&k));
                    let bell_title = if muted {
                        tr(lang, "bell_off_title").replace("{nama}", nama)
                    } else {
                        tr(lang, "bell_on_title").replace("{nama}", nama)
                    };
                    let bell_icon_class = if muted {
                        format!("{bell_class} opacity-60")
                    } else {
                        bell_class.to_string()
                    };
                    rsx! {
                        div { key: "{row.key}", class: "{row_class}",
                            span { class: "{name_class}", "{nama}" }
                            div { class: "flex items-center gap-2",
                                span { class: "{time_class}", "{prayer::format_time(row.time)}" }
                                if let Some(kind) = row.kind {
                                    span {
                                        class: "{bell_wrap_class}",
                                        title: "{bell_title}",
                                        onclick: move |_| toggle_adzan_mute(kind),
                                        if muted {
                                            icons::IconBellOff { class: "{bell_icon_class}" }
                                        } else {
                                            icons::IconBell { class: "{bell_icon_class}" }
                                        }
                                    }
                                } else {
                                    // Placeholder agar kolom jam sejajar antar baris.
                                    span { class: "{bell_wrap_class} invisible pointer-events-none",
                                        icons::IconBell { class: "{bell_icon_class}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Toggle mute adzan satu sholat (dipanggil klik icon bell di daftar jadwal).
fn toggle_adzan_mute(kind: prayer::PrayerKind) {
    let mut s = (*state::settings()).clone();
    if s.muted_prayers.contains(&kind) {
        s.muted_prayers.retain(|k| *k != kind);
    } else {
        s.muted_prayers.push(kind);
    }
    state::update_settings(s);
}
