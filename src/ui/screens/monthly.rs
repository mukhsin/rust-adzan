//! Layar Bulanan — jadwal sholat satu bulan dalam dua papan berdampingan
//! (1–15 kiri, 16–akhir bulan kanan) ala frame "Bulanan" design system pen.
//!
//! CATATAN DEVIASI dari pen: tinggi BARIS dibuat seragam (33px) di kedua
//! papan — papan kanan memang satu baris lebih banyak (16 vs 15), jadi
//! papan kanan lebih tinggi total (items-start), bukan dipaksa sama tinggi
//! seperti mockup yang membuat baris kanan lebih pendek.

use crate::i18n::{month_name, tr};
use crate::prayer;
use crate::state;
use crate::ui::components::ui::icons;
use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;

/// Tinggi seragam satu baris tanggal. Konstanta class terpisah (bukan
/// interpolasi angka) supaya scanner Tailwind menemukan kelasnya.
const ROW_H_CLASS: &str = "h-[33px]";

/// Waktu sholat utama satu hari, terformat siap tampil
/// (subuh, dzuhur, ashar, maghrib, isya).
#[derive(Clone, PartialEq)]
struct HariRow {
    day: u32,
    times: Option<[String; 5]>,
}

#[component]
pub fn MonthlyView() -> Element {
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

    // Jadwal seluruh hari di bulan tampil — dihitung tiap render; tanpa tick
    // 1 detik, jadi hanya berjalan saat ganti bulan / settings.
    let last_day = days_in_month(year, month);
    let mut rows: Vec<HariRow> = Vec::with_capacity(last_day as usize);
    for day in 1..=last_day {
        let times = NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| prayer::day_times(&settings, date))
            .map(|t| {
                [
                    prayer::format_time(t.subuh, settings.time_format_24),
                    prayer::format_time(t.dzuhur, settings.time_format_24),
                    prayer::format_time(t.ashar, settings.time_format_24),
                    prayer::format_time(t.maghrib, settings.time_format_24),
                    prayer::format_time(t.isya, settings.time_format_24),
                ]
            });
        rows.push(HariRow { day, times });
    }

    let split = last_day.min(15) as usize;
    let (kiri, kanan) = rows.split_at(split);
    let bulan_tampil = (year, month) == (today.year(), today.month());
    let today_day = bulan_tampil.then_some(today.day());

    let label_kiri = format!("1 — {split} {}", month_name(lang, (month - 1) as usize));
    let label_kanan = format!(
        "{} — {} {}",
        split + 1,
        last_day,
        month_name(lang, (month - 1) as usize)
    );

    rsx! {
        div { class: "flex-1 p-4 flex flex-col gap-[14px] overflow-y-auto",

            // Kepala bulan: navigasi + judul + penanda hari ini.
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
                if bulan_tampil {
                    div { class: "flex items-center gap-2 bg-amber rounded-full px-3 py-1.5",
                        span { class: "w-[7px] h-[7px] rounded-full bg-amber-ink" }
                        span { class: "font-mono text-[10.5px] font-semibold tracking-[0.08em] uppercase text-amber-ink",
                            "{today.day()} · {tr(lang, \"today\")}"
                        }
                    }
                }
            }

            // Sepasang papan: 1–15 kiri, 16–akhir kanan. Tinggi baris seragam
            // di kedua papan; items-start = papan kanan lebih tinggi karena
            // satu baris ekstra (deviasi sengaja dari mockup pen).
            div { class: "flex items-start gap-4",
                MonthBoard { label: label_kiri, rows: kiri.to_vec(), today_day: today_day }
                MonthBoard { label: label_kanan, rows: kanan.to_vec(), today_day: today_day }
            }
        }
    }
}

/// Satu papan tanggal: label rentang, kepala kolom, lalu baris per tanggal.
#[component]
fn MonthBoard(label: String, rows: Vec<HariRow>, today_day: Option<u32>) -> Element {
    let lang = state::settings().language;
    let kolom = ["subuh", "dzuhur", "ashar", "maghrib", "isya"];

    rsx! {
        div { class: "flex-1 bg-board border border-pass-line rounded-xl p-2 px-3.5 flex flex-col gap-0.5",
            div { class: "px-2.5 pt-2",
                span { class: "font-mono text-[10.5px] tracking-[0.14em] uppercase text-ink-3",
                    "{label}"
                }
            }
            div { class: "flex items-center gap-3.5 px-2.5 h-[26px]",
                span { class: "font-mono text-[9.5px] uppercase text-ink-3 w-[34px]",
                    "{tr(lang, \"col_tgl\")}"
                }
                for k in kolom {
                    span { class: "flex-1 font-mono text-[9.5px] uppercase text-ink-3",
                        "{tr(lang, k)}"
                    }
                }
            }
            for (i, row) in rows.iter().enumerate() {
                {
                    let is_today = today_day == Some(row.day);
                    let is_last = i == rows.len() - 1;
                    let row_class = if is_today {
                        "flex items-center gap-3.5 px-2.5 bg-amber"
                    } else if is_last {
                        "flex items-center gap-3.5 px-2.5"
                    } else {
                        "flex items-center gap-3.5 px-2.5 border-b border-hairline"
                    };
                    let tgl_class = if is_today {
                        "w-[34px] font-mono text-xs font-bold tabular-nums text-amber-ink"
                    } else {
                        "w-[34px] font-mono text-xs font-bold tabular-nums text-board-text"
                    };
                    let time_base = if is_today {
                        "flex-1 font-mono text-[11.5px] font-semibold tabular-nums text-amber-ink"
                    } else {
                        "flex-1 font-mono text-[11.5px] tabular-nums text-board-text"
                    };
                    let tgl = format!("{:02}", row.day);
                    rsx! {
                        div { class: "{row_class} {ROW_H_CLASS}",
                            span { class: "{tgl_class}", "{tgl}" }
                            match &row.times {
                                Some(times) => rsx! {
                                    for nilai in times {
                                        span { class: "{time_base}", "{nilai}" }
                                    }
                                },
                                None => rsx! {
                                    for _ in 0..5 {
                                        span { class: "{time_base} opacity-40", "—" }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
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
