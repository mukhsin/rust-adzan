//! Papan jadwal hari ini — tujuh baris (Imsak..Isya) dengan status lewat /
//! mendatang / berikutnya (disorot amber) plus lonceng adzan interaktif di
//! samping jam untuk sholat wajib.

use super::ui::icons;
use crate::i18n::tr;
use crate::prayer::{self, DayTimes, PrayerKind};
use crate::state;
use chrono::NaiveTime;
use dioxus::prelude::*;

/// Satu baris jadwal untuk dirender.
struct Row {
    /// Kunci terjemahan nama (lihat `i18n::PHRASES`).
    key: &'static str,
    time: NaiveTime,
    /// Some(kind) untuk sholat wajib — bisa jadi "berikutnya" dan punya
    /// lonceng adzan interaktif. None (Imsak/Terbit) = baris penanda saja.
    kind: Option<PrayerKind>,
}

#[component]
pub fn PrayerTable(times: DayTimes, now: NaiveTime, next_kind: PrayerKind) -> Element {
    let lang = state::settings().language;
    let settings = state::settings();
    // Setelah Isya (menuju Subuh esok), sorotan tetap di baris Isya.
    let highlight_isya = next_kind == PrayerKind::Subuh && now > times.isya;
    // "Lewati adzan berikutnya" (tray) aktif → lonceng sholat berikutnya
    // tampil off, konsisten dengan centang di menu tray.
    let skip_aktif = state::skip_next_armed();

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
            div { class: "flex flex-col p-4 pl-6 h-full",
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
                            "text-xs uppercase font-count font-bold text-amber-ink"
                        } else if is_lewat {
                            "text-xs uppercase font-count font-medium text-ink-3"
                        } else {
                            "text-xs uppercase font-count font-medium text-ink-2"
                        };
                        let time_class = if is_next {
                            "text-[13px] font-count font-semibold tabular-nums text-amber-ink"
                        } else if is_lewat {
                            "text-[13px] font-count font-medium tabular-nums text-ink-3"
                        } else {
                            "text-[13px] font-count font-medium tabular-nums text-board-text"
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
                        // dioxus, jadi flip ikon butuh perubahan props. Lonceng
                        // sholat berikutnya ikut tampil off saat skip armed —
                        // dipisah dari `muted` karena kliknya beda makna.
                        let muted = row
                            .kind
                            .is_some_and(|k| settings.muted_prayers.contains(&k));
                        let skip = skip_aktif && row.kind.is_some_and(|k| k == next_kind);

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
                                        BellRow { kind: kind, color: bell_color, nama: nama, muted: muted, skip: skip }
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
/// penulisan settings & persistensi berjalan di belakang. `skip` true saat
/// adzan baris ini ditarget "lewati adzan berikutnya" (tray): ikon tampil
/// off dan kliknya MEMBATALKAN skip, bukan mengubah mute.
#[component]
fn BellRow(
    kind: PrayerKind,
    color: &'static str,
    nama: String,
    muted: bool,
    skip: bool,
) -> Element {
    let lang = state::settings().language;
    // State tampilan lokal: sumber kebenaran ikon, disinkronkan dari prop
    // saat mount. Props di luar widget tidak mengubah mute, jadi aman.
    let mut shown = use_signal(|| muted);

    let title = if skip {
        tr(lang, "bell_skip_title").replace("{nama}", &nama)
    } else if *shown.read() {
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
                if skip {
                    // Klik lonceng yang off karena skip = batalkan skip.
                    shown.set(false);
                    spawn(async move {
                        crate::state::cancel_skip_next_adzan();
                        // Item tray hanya bisa disentuh main thread — lewat
                        // channel perintah dom utama.
                        crate::state::send_main_command(
                            crate::state::MainCommand::SkipInactive,
                        );
                        crate::logger::info("lewati adzan dibatalkan (papan jadwal)");
                    });
                } else {
                    shown.toggle();
                    spawn(async move {
                        toggle_adzan_mute(kind);
                    });
                }
            },
            if *shown.read() || skip {
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
