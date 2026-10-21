//! Layar Kalender — grid bulan campuran Masehi–Hijriah–Jawa ala frame
//! "Kalender" design system pen:
//! - tiap sel: tanggal hijriah (angka Arab, kanan-atas), tanggal masehi
//!   besar (tengah), dan baris weton Jawa — Tgl Jawa · Pasaran · Neptu;
//! - Ahad disorot merah (`ahad`), Jum'at hijau (`jumat`), hari ini amber;
//! - kepala kolom dwibahasa: Latin + Arab.
//!
//! Kalender Jawa mengikuti kalender hijriah (Tgl Jawa = tanggal hijriah),
//! dan Neptu = neptu hari + neptu pasaran.

use crate::components::ui::icons;
use crate::hijri;
use crate::i18n::{day_name, hijri_month_name, month_name, tr};
use crate::prayer;
use crate::state;
use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;

/// Nama hari kolom (Ahad..Sabtu) — indeks chrono `num_days_from_sunday`.
const KOLOM: [&str; 7] = ["ahad", "senin", "selasa", "rabu", "kamis", "jumat", "sabtu"];

/// Nama hari kolom dalam Arab.
const KOLOM_ARAB: [&str; 7] = [
    "الأحد",
    "الإثنين",
    "الثلاثاء",
    "الأربعاء",
    "الخميس",
    "الجمعة",
    "السبت",
];

/// Nama pasaran siklus 5 hari Jawa.
const PASARAN: [&str; 5] = ["Legi", "Pahing", "Pon", "Wage", "Kliwon"];

/// Neptu hari: indeks = `num_days_from_sunday` (Ahad 5 … Sabtu 9).
const NEPTU_HARI: [u32; 7] = [5, 4, 3, 7, 8, 6, 9];

/// Neptu pasaran: indeks = siklus [`PASARAN`] (Legi 5 … Kliwon 8).
const NEPTU_PASARAN: [u32; 5] = [5, 9, 7, 4, 8];

#[component]
pub fn CalendarView() -> Element {
    let settings = state::settings();
    let lang = settings.language;
    let today = prayer::now_in_tz(&settings).date_naive();

    // Bulan aktif (tahun, bulan) — awal: bulan hari ini.
    let mut bulan = use_signal(|| (today.year(), today.month()));
    let (year, month) = *bulan.read();

    let kota = settings
        .city
        .clone()
        .unwrap_or_else(|| "Manual".to_string());
    let bulan_label = format!(
        "{} {} · {}",
        month_name(lang, (month - 1) as usize),
        year,
        kota
    );

    // Rentang hijriah GRID tampil (bukan cuma bulan): sel pertama bisa
    // beberapa hari bulan sebelumnya — "16 Rabiul Akhir – 20 Jumadil Awal H".
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("tanggal valid");
    let last_day = days_in_month(year, month);
    let last = NaiveDate::from_ymd_opt(year, month, last_day).expect("tanggal valid");
    let lead = first.weekday().num_days_from_sunday();
    let grid_first = first - chrono::Duration::days(lead as i64);
    let (_h1, m1, d1) = hijri::to_hijri(grid_first);
    let (h2, m2, d2) = hijri::to_hijri(last);
    let rentang = format!(
        "{} {} – {} {} {} H",
        d1,
        hijri_month_name(lang, (m1 - 1) as usize),
        d2,
        hijri_month_name(lang, (m2 - 1) as usize),
        h2
    );

    // Sel kalender: 4 minggu–6 minggu, diawali Ahad; sisa diisi out-month.
    let total_cells = (lead + last_day) as usize;
    let weeks = total_cells.div_ceil(7);
    let bulan_tampil = (year, month) == (today.year(), today.month());

    rsx! {
        div { class: "flex-1 p-4 flex flex-col gap-[14px] overflow-y-auto",

            // Kepala kalender: navigasi + judul + rentang hijriah + hari ini.
            div { class: "flex items-center gap-4 px-1 py-1",
                div { class: "flex items-center gap-3.5",
                    button {
                        class: "p-1 -m-1 rounded-md text-ink-2 hover:text-ink transition-colors cursor-pointer",
                        title: tr(lang, "prev_month"),
                        onclick: move |_| {
                            let (y, m) = bulan();
                            bulan.set(if m == 1 { (y - 1, 12) } else { (y, m - 1) });
                        },
                        icons::IconChevronLeft { class: "w-5 h-5" }
                    }
                    span { class: "text-[21px] font-bold tracking-wide uppercase text-ink",
                        "{bulan_label}"
                    }
                    button {
                        class: "p-1 -m-1 rounded-md text-ink-2 hover:text-ink transition-colors cursor-pointer",
                        title: tr(lang, "next_month"),
                        onclick: move |_| {
                            let (y, m) = bulan();
                            bulan.set(if m == 12 { (y + 1, 1) } else { (y, m + 1) });
                        },
                        icons::IconChevronRight { class: "w-5 h-5" }
                    }
                }
                div { class: "flex-1" }
                span { class: "font-mono text-[10.5px] tracking-[0.08em] uppercase text-ink-3",
                    "{rentang}"
                }
                if bulan_tampil {
                    div { class: "flex items-center gap-2 bg-amber rounded-full px-3 py-1.5",
                        span { class: "w-[7px] h-[7px] rounded-full bg-amber-ink" }
                        span { class: "font-mono text-[10.5px] font-semibold tracking-[0.08em] uppercase text-amber-ink",
                            "{today.day()} · {tr(lang, \"today\")}"
                        }
                    }
                }
            }

            // Papan kalender: kepala kolom + baris minggu (grid 7 kolom).
            div { class: "flex-1 bg-board border border-pass-line rounded-xl flex flex-col overflow-hidden",
                div { class: "grid grid-cols-7 h-12 border-b border-hairline",
                    for (i, (latin_key, arab)) in KOLOM.iter().zip(KOLOM_ARAB).enumerate() {
                            {
                                // Indeks chrono: Senin=0..Minggu=6; kolom 0 = Ahad.
                                let chrono_idx = if i == 0 { 6 } else { i - 1 };
                                let latin = match *latin_key {
                                    // Konvensi kalender Islam: kolom pertama
                                    // "Ahad", bukan "Minggu".
                                    "ahad" => match lang {
                                        crate::settings::Language::Id => "Ahad",
                                        crate::settings::Language::En => "Sunday",
                                    },
                                    "jumat" => day_name(lang, 4),
                                    _ => day_name(lang, chrono_idx),
                                };
                            let warna = if *latin_key == "ahad" {
                                "text-ahad"
                            } else if *latin_key == "jumat" {
                                "text-jumat"
                            } else {
                                "text-board-dim"
                            };
                            rsx! {
                                div { class: "flex flex-col items-center justify-center gap-[3px]",
                                    span { class: "font-mono text-[9.5px] font-semibold uppercase {warna}",
                                        "{latin}"
                                    }
                                    span { class: "font-[Noto_Naskh_Arabic] text-[11px] text-ink-3",
                                        dir: "rtl",
                                        "{arab}"
                                    }
                                }
                            }
                        }
                    }
                }
                for minggu in 0..weeks {
                    div { class: "grid grid-cols-7 flex-1 min-h-0",
                        for col in 0..7 {
                            {
                                let idx = minggu * 7 + col;
                                let date = first - chrono::Duration::days(lead as i64)
                                    + chrono::Duration::days(idx as i64);
                                let in_month = date.month() == month;
                                let is_today = date == today;
                                let weekday = date.weekday().num_days_from_sunday() as usize;
                                let (h_y, h_m, h_d) = hijri::to_hijri(date);
                                // Penanda tanggal penting Islam (1 Ramadhan,
                                // Idul Fitri, dst.) dari tanggal hijriah sel.
                                let acara = crate::dates::by_hijri(h_m, h_d)
                                    .map(|kind| tr(lang, kind.key()));
                                let _ = h_y;
                                let weton_tgl = h_d;
                                let pasaran = PASARAN[pasaran_index(date)];
                                let neptu = NEPTU_HARI[weekday]
                                    + NEPTU_PASARAN[pasaran_index(date)];

                                // Pewarnaan: Ahad merah, Jum'at hijau — hanya
                                // tanggal dalam bulan; luar bulan selalu redup.
                                let masehi_class: String = if is_today {
                                    "font-mono text-[20px] font-bold text-amber-ink".into()
                                } else if !in_month {
                                    "font-mono text-[20px] text-ink-3".into()
                                } else if weekday == 0 {
                                    "font-mono text-[20px] font-bold text-ahad".into()
                                } else if weekday == 5 {
                                    "font-mono text-[20px] font-bold text-jumat".into()
                                } else {
                                    "font-mono text-[20px] font-bold text-board-text".into()
                                };
                                let weton_class: String = if is_today {
                                    "font-mono text-[9px] font-semibold text-amber-ink".into()
                                } else if !in_month {
                                    "font-mono text-[9px] font-semibold text-board-dim".into()
                                } else if weekday == 0 {
                                    "font-mono text-[9px] font-semibold text-ahad".into()
                                } else if weekday == 5 {
                                    "font-mono text-[9px] font-semibold text-jumat".into()
                                } else {
                                    "font-mono text-[9px] font-semibold text-board-dim".into()
                                };
                                let mut cell_class = String::from("flex flex-col p-3 px-3.5");
                                if col < 6 {
                                    cell_class.push_str(" border-r border-hairline");
                                }
                                if minggu + 1 < weeks {
                                    cell_class.push_str(" border-b border-hairline");
                                }
                                if is_today {
                                    cell_class.push_str(" rounded-md bg-amber");
                                }
                                let hijriah = arabic_numerals(h_d);
                                let pasaran_nama = pasaran.to_uppercase();
                                // Label peristiwa: amber saat dalam bulan
                                // (dibaca), redup saat out-month. Pakai
                                // text-amber (bukan amber-ink) agar terbaca
                                // di tema gelap.
                                let acara_class = if in_month {
                                    "text-[9px] font-bold uppercase tracking-wide text-amber truncate max-w-full"
                                } else {
                                    "text-[9px] font-bold uppercase tracking-wide text-board-dim truncate max-w-full"
                                };

                                rsx! {
                                    div { class: "{cell_class}",
                                        div { class: "h-5 text-right",
                                            span { class: "font-[Noto_Naskh_Arabic] text-[11.5px] text-ink-3",
                                                dir: "rtl",
                                                "{hijriah}"
                                            }
                                        }
                                        div { class: "flex-1 flex flex-col items-center justify-center gap-[3px] min-w-0",
                                            span { class: "{masehi_class}", "{date.day()}" }
                                            if let Some(nama) = &acara {
                                                span { class: "{acara_class}", title: "{nama}", "{nama}" }
                                            }
                                        }
                                        div { class: "h-[13px] flex items-center justify-between",
                                            span { class: "{weton_class}", "{weton_tgl}" }
                                            span { class: "{weton_class}", "{pasaran_nama}" }
                                            span { class: "{weton_class}", "{neptu}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Keterangan warna.
            div { class: "flex items-center gap-[18px] px-1.5 py-0.5",
                div { class: "flex items-center gap-[7px]",
                    span { class: "w-[7px] h-[7px] rounded-full bg-amber" }
                    span { class: "font-mono text-[9.5px] font-semibold uppercase text-ink-2",
                        "{tr(lang, \"today\")}"
                    }
                }
                div { class: "flex items-center gap-[7px]",
                    span { class: "w-[7px] h-[7px] rounded-full bg-ahad" }
                    span { class: "font-mono text-[9.5px] font-semibold uppercase text-ink-2",
                        "{tr(lang, \"day_ahad\")}"
                    }
                }
                div { class: "flex items-center gap-[7px]",
                    span { class: "w-[7px] h-[7px] rounded-full bg-jumat" }
                    span { class: "font-mono text-[9.5px] font-semibold uppercase text-ink-2",
                        "{day_name(lang, 4)}"
                    }
                }
                div { class: "flex items-center gap-[7px]",
                    span { class: "font-mono text-[9.5px] font-bold uppercase text-amber",
                        "1 Ram"
                    }
                    span { class: "font-mono text-[9.5px] font-semibold uppercase text-ink-2",
                        "{tr(lang, \"sec_dates\")}"
                    }
                }
            }
        }
    }
}

/// Angka Latin → angka Arab-Indic (٠–٩) untuk tanggal hijriah.
fn arabic_numerals(n: u32) -> String {
    n.to_string()
        .chars()
        .map(|c| char::from_u32(0x0660 + c.to_digit(10).expect("digit")).expect("arabic digit"))
        .collect()
}

/// Indeks pasaran (0 = Legi … 4 = Kliwon) untuk tanggal.
/// Anchor: 26 September 2026 = Legi (indeks 0), dikalibrasi dari design
/// system (27 Sep 2026 = Pahing).
fn pasaran_index(date: NaiveDate) -> usize {
    let anchor = NaiveDate::from_ymd_opt(2026, 9, 26).expect("tanggal valid");
    let diff = (date - anchor).num_days();
    (((diff % 5) + 5) % 5) as usize
}

/// Jumlah hari dalam satu bulan.
fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_y, next_m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let start = NaiveDate::from_ymd_opt(year, month, 1).expect("bulan valid");
    let next = NaiveDate::from_ymd_opt(next_y, next_m, 1).expect("bulan berikutnya valid");
    (next - start).num_days() as u32
}
