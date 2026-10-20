//! Widget desktop — kartu tiket "Jadwal Sholat" (redesain dari design system
//! pen): kiri identitas + hitung mundur + status, kanan papan jadwal hari ini
//! dengan lonceng adzan per baris. Seluruh kartu bisa didrag native; klik
//! ganda membuka window pengaturan.

use super::icons;
use crate::hijri;
use crate::i18n::{day_name, month_name, tr};
use crate::prayer::{self, DayTimes, PrayerKind};
use crate::state;
use chrono::{Datelike, NaiveTime};
use dioxus::prelude::*;

/// Root komponen widget window — dom terpisah dari window utama,
/// state dibaca dari global `state::settings()` setiap tick.
/// Drag memakai mekanisme native (`window.drag()`, smooth seperti main
/// window); saat drag selesai posisi di-snap ke margin oleh
/// `crate::ui::clamp_widget_position`.
#[allow(non_snake_case)]
pub fn WidgetApp() -> Element {
    // Window widget dibuat tersembunyi; tampilkan setelah dom siap. Efek ini
    // berjalan pada poll setelah event initialize — yaitu SETELAH dioxus
    // menyetel ulang visibilitas window (set_visible(is_visible_before_start)
    // = false karena window utama mulai tersembunyi), jadi dipanggilannya
    // tidak tertimpa dan widget tampil dengan konten siap tanpa kedipan.
    use_effect(move || {
        dioxus::desktop::window().set_visible(true);
    });

    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });
    // Handle window per-closure (Rc, murah untuk di-clone).
    let win_drag = dioxus::desktop::window();

    let _ = tick();
    // Tema & settings dibaca langsung tiap render — tick 1 detik menjamin
    // perubahan dari dom lain (toggle tema di pengaturan) tampil maksimal
    // 1 detik kemudian.
    let settings = state::settings();
    let lang = settings.language;
    let dark = settings.theme.is_dark();
    let now = prayer::now_in_tz(&settings);
    let hari = day_name(lang, now.weekday().num_days_from_monday() as usize).to_string();
    let tanggal = format!(
        "{} {} {}",
        now.day(),
        month_name(lang, (now.month() - 1) as usize),
        now.year()
    );
    let tanggal_hijri = hijri::format(lang, now.date_naive());

    let mut root_class =
        String::from("h-screen w-screen overflow-hidden select-none font-sans text-ink");
    if dark {
        root_class.push_str(" dark");
    }

    rsx! {
        style { {crate::ui::TAILWIND_CSS} }
        div {
            class: "{root_class}",
            // Drag native — modal loop OS, kembali saat mouse dilepas.
            onmousedown: move |_| {
                win_drag.drag();
                // drag() di Windows async: clamp di sini jalan saat mousedown,
                // bukan saat drag selesai. Karena itu snap ditangguhkan ke
                // watcher yang memantau tombol mouse.
                crate::ui::watch_widget_drag_release();
            },
            // Klik ganda kartu = buka pengaturan (tak ada tombol di desain).
            ondoubleclick: move |_| state::send_main_command(state::MainCommand::ShowMain),

            match prayer::day_times(&settings, now.date_naive()) {
                Some(times) => rsx! {
                    Ticket {
                        times: times,
                        now: now.time(),
                        tz: now.format("%Z").to_string(),
                        hari: hari,
                        tanggal: tanggal,
                        tanggal_hijri: tanggal_hijri,
                    }
                },
                None => rsx! {
                    div { class: "w-full h-full flex items-center justify-center bg-pass rounded-[14px] widget-holes",
                        span { class: "text-sm text-ink-3", "{tr(lang, \"calc_failed\")}" }
                    }
                },
            }
        }
    }
}

/// Kartu tiket utuh: identitas + countdown + status (kiri), papan jadwal
/// (kanan), dipisah garis sobekan perforasi.
#[component]
fn Ticket(
    times: DayTimes,
    now: NaiveTime,
    tz: String,
    hari: String,
    tanggal: String,
    tanggal_hijri: String,
) -> Element {
    let settings = state::settings();
    let lang = settings.language;

    let (next_kind, target) = prayer::next_prayer(&times, now);
    let countdown = prayer::format_countdown(prayer::time_until(now, target));
    let kota = settings
        .city
        .clone()
        .unwrap_or_else(|| "Manual".to_string());
    let jam_target = format!(
        "{} {}",
        prayer::format_time(target, settings.time_format_24),
        tz
    );
    let menuju = format!("{} {}", tr(lang, "countdown_prefix"), next_kind.nama(lang));
    let masuk = tr(lang, "masuk_pukul").replace("{waktu}", &jam_target);

    rsx! {
        div { class: "w-full h-full flex bg-pass overflow-hidden widget-holes",
            // KARTU UTAMA — identitas, hitung mundur, status.
            div { class: "flex-1",
                div { class: "flex flex-col justify-between p-6 h-full",
                    div { class: "flex flex-col gap-1.5 min-w-0",
                        span {
                            class: "font-mono text-[10.5px] font-semibold tracking-[0.08em] uppercase text-ink-3 truncate",
                            "{tr(lang, \"app_name\")} · {kota}"
                        }
                    }
                    div { class: "flex flex-col gap-2",
                        div { class: "flex font-count items-baseline px-1 gap-2 min-w-0",
                            span { class: "text-[16px] font-semibold text-ink whitespace-nowrap", "{menuju}" }
                            span { class: "text-[12px] text-ink-3 truncate", "{masuk}" }
                        }
                        div { class: "font-count font-thin text-[72px] leading-none tracking-tight text-ink tabular-nums",
                            "{countdown}"
                        }
                    }
                    div { class: "flex flex-col",
                        div { class: "font-mono text-[9.5px] tracking-[0.18em] uppercase text-ink-3 truncate",
                            "{hari}"
                        }
                        div { class: "font-mono text-[9.5px] tracking-[0.18em] uppercase text-ink-3 truncate",
                            "{tanggal}"
                        }
                        div { class: "font-mono text-[9.5px] tracking-[0.18em] uppercase text-ink-3 truncate",
                            "{tanggal_hijri}"
                        }
                    }
                }
            }

            div { class: "w-6 shrink-0 flex flex-col items-center justify-between py-16",
                for _ in 0..5 {
                    span { class: "w-1 h-1 rounded-full bg-pass-line shrink-0" }
                }
            }

            // PAPAN JADWAL HARI INI.
            Board { times: times, now: now, next_kind: next_kind }
        }
    }
}

/// Satu baris jadwal untuk dirender.
struct Row {
    /// Kunci terjemahan nama (lihat `i18n::PHRASES`).
    key: &'static str,
    time: NaiveTime,
    /// Some(kind) untuk sholat wajib — bisa jadi "berikutnya" dan punya
    /// lonceng adzan interaktif. None (Imsak/Terbit) = baris penanda saja.
    kind: Option<PrayerKind>,
}

/// Papan jadwal: tujuh baris dengan status lewat / mendatang / berikutnya
/// (disorot amber) plus lonceng adzan di samping jam.
#[component]
fn Board(times: DayTimes, now: NaiveTime, next_kind: PrayerKind) -> Element {
    let lang = state::settings().language;
    let settings = state::settings();
    // Setelah Isya (menuju Subuh esok), sorotan tetap di baris Isya.
    let highlight_isya = next_kind == PrayerKind::Subuh && now > times.isya;

    let rows = [
        Row {
            key: "imsak",
            time: times.imsak,
            kind: None,
        },
        Row {
            key: "subuh",
            time: times.subuh,
            kind: Some(PrayerKind::Subuh),
        },
        Row {
            key: "terbit",
            time: times.terbit,
            kind: None,
        },
        Row {
            key: "dzuhur",
            time: times.dzuhur,
            kind: Some(PrayerKind::Dzuhur),
        },
        Row {
            key: "ashar",
            time: times.ashar,
            kind: Some(PrayerKind::Ashar),
        },
        Row {
            key: "maghrib",
            time: times.maghrib,
            kind: Some(PrayerKind::Maghrib),
        },
        Row {
            key: "isya",
            time: times.isya,
            kind: Some(PrayerKind::Isya),
        },
    ];

    // Index baris "berikutnya" (disorot amber) — dipakai untuk is_before_next.
    let next_index = rows.iter().position(|row| match row.kind {
        Some(k) => {
            (k == next_kind && !highlight_isya)
                || (row.kind == Some(PrayerKind::Isya) && highlight_isya)
        }
        None => false,
    });

    rsx! {
        div { class: "flex-1",
            div { class: "flex flex-col p-4 h-full",
                for (i, row) in rows.iter().enumerate() {
                    {
                        let is_next = match row.kind {
                            Some(k) => {
                                (k == next_kind && !highlight_isya)
                                    || (row.kind == Some(PrayerKind::Isya) && highlight_isya)
                            }
                            None => false,
                        };
                        // Baris terakhir tanpa garis bawah.
                        let is_last = i == rows.len() - 1;
                        // Baris tepat di atas sorotan amber — juga tanpa garis
                        // bawah supaya sorotan punya ruang bersih di atasnya.
                        let is_before_next = next_index.is_some_and(|nx| i + 1 == nx);
                        let is_lewat = !is_next && now > row.time;
                        let nama = tr(lang, row.key);

                        let name_class = if is_next {
                            "text-xs font-bold text-amber-ink"
                        } else if is_lewat {
                            "text-xs text-ink-3"
                        } else {
                            "text-xs text-ink-2"
                        };
                        let time_class = if is_next {
                            "font-mono text-[13px] font-semibold tabular-nums text-amber-ink"
                        } else if is_lewat {
                            "font-mono text-[13px] font-medium tabular-nums text-ink-3"
                        } else {
                            "font-mono text-[13px] font-medium tabular-nums text-board-text"
                        };
                        let bell_color: &'static str = if is_next {
                            "text-amber-ink"
                        } else if is_lewat {
                            "text-ink-3"
                        } else {
                            "text-board-text"
                        };
                        // State mute dibaca di sini (bukan di dalam BellRow):
                        // komponen dengan props yang sama tidak di-render ulang
                        // dioxus, jadi flip ikon butuh perubahan props.
                        let muted = row
                            .kind
                            .is_some_and(|k| settings.muted_prayers.contains(&k));

                        rsx! {
                            div {
                                key: "{row.key}",
                                class: if is_next {
                                    "flex-1 min-h-0 flex items-center justify-between gap-2.5 px-3.5 rounded-md bg-amber"
                                } else if is_last || is_before_next {
                                    "flex-1 min-h-0 flex items-center gap-2.5 px-3.5"
                                } else {
                                    "flex-1 min-h-0 flex items-center gap-2.5 px-3.5 border-b border-hairline"
                                },
                                span { class: "{name_class}", "{nama}" }
                                if !is_next {
                                    // Pengisi transparan: hanya mendorong jam ke kanan.
                                    div { class: "flex-1 h-px" }
                                }
                                div { class: "flex items-center gap-3 shrink-0",
                                    if let Some(kind) = row.kind {
                                        BellRow { kind: kind, color: bell_color, nama: nama, muted: muted }
                                    }
                                    span { class: "{time_class}", "{prayer::format_time(row.time, settings.time_format_24)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Lonceng satu baris jadwal — hanya sholat wajib (Imsak/Terbit tidak
/// dirender sama sekali). Klik = flip ikon INSTAN (optimistis), lalu
/// penulisan settings & persistensi berjalan di belakang.
#[component]
fn BellRow(kind: PrayerKind, color: &'static str, nama: String, muted: bool) -> Element {
    let lang = state::settings().language;
    // State tampilan lokal: sumber kebenaran ikon, disinkronkan dari prop
    // saat mount. Props di luar widget tidak mengubah mute, jadi aman.
    let mut shown = use_signal(|| muted);

    let title = if *shown.read() {
        tr(lang, "bell_off_title").replace("{nama}", &nama)
    } else {
        tr(lang, "bell_on_title").replace("{nama}", &nama)
    };

    rsx! {
        button {
            class: "p-1 -m-1 rounded-full cursor-pointer {color} hover:opacity-70 transition-opacity",
            title: "{title}",
            // Jangan ikut memicu drag kartu / buka pengaturan.
            onmousedown: move |e| e.stop_propagation(),
            ondoubleclick: move |e| e.stop_propagation(),
            onclick: move |_| {
                shown.toggle();
                spawn(async move {
                    toggle_adzan_mute(kind);
                });
            },
            if *shown.read() {
                icons::IconBellOff { class: "w-3 h-3" }
            } else {
                icons::IconBell { class: "w-3 h-3" }
            }
        }
    }
}

/// Toggle mute adzan satu sholat (dipanggil klik icon bell di papan jadwal).
fn toggle_adzan_mute(kind: PrayerKind) {
    let mut s = (*state::settings()).clone();
    if s.muted_prayers.contains(&kind) {
        s.muted_prayers.retain(|k| *k != kind);
    } else {
        s.muted_prayers.push(kind);
    }
    state::update_settings(s);
}
