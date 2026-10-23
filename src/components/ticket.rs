//! Kartu tiket utuh — komposisi pane countdown (kiri), garis sobekan
//! perforasi (tengah), dan papan jadwal hari ini (kanan). Dipakai widget
//! window dan layar Harian window utama.

use super::countdown::CountdownPane;
use super::prayer_table::PrayerTable;
use crate::prayer::{self, DayTimes};
use crate::settings::Language;
use chrono::{DateTime, NaiveTime};
use chrono_tz::Tz;
use dioxus::prelude::*;

/// Label (hari, tanggal masehi, tanggal hijri) untuk kartu tiket — dipakai
/// widget dan layar Harian. Hari & tanggal masehi mengikuti jam sekarang;
/// tanggal hijri memakai tanggal efektif hasil rollover Maghrib
/// (`prayer::tanggal_efektif_hijriah`) — setelah Maghrib, hari hijriah
/// sudah milik besok.
pub fn tanggal_labels(
    lang: Language,
    now: &DateTime<Tz>,
    tanggal_hijri: chrono::NaiveDate,
) -> (String, String, String) {
    use crate::i18n::{day_name, month_name};
    use chrono::Datelike as _;
    (
        day_name(lang, now.weekday().num_days_from_monday() as usize).to_string(),
        format!(
            "{} {} {}",
            now.day(),
            month_name(lang, (now.month() - 1) as usize),
            now.year()
        ),
        crate::hijri::format(lang, tanggal_hijri),
    )
}

#[component]
pub fn Ticket(
    times: DayTimes,
    now: NaiveTime,
    jumat: bool,
    tz: String,
    hari: String,
    tanggal: String,
    tanggal_hijri: String,
    ramadan_day: Option<u32>,
    puasa: Option<String>,
) -> Element {
    // Sholat berikutnya menentukan sorotan amber di papan jadwal.
    let (next_kind, _target) = prayer::next_prayer(&times, now);

    rsx! {
        div { class: "w-full h-full flex bg-pass overflow-hidden widget-holes",
            // KARTU UTAMA — identitas, hitung mundur, status.
            CountdownPane {
                times: times,
                now: now,
                jumat: jumat,
                tz: tz,
                hari: hari,
                tanggal: tanggal,
                tanggal_hijri: tanggal_hijri,
                ramadan_day: ramadan_day,
                puasa: puasa,
            }

            div { class: "shrink-0 flex flex-col items-center justify-between py-16",
                for _ in 0..5 {
                    span { class: "w-1 h-1 rounded-full bg-pass-line shrink-0" }
                }
            }

            // PAPAN JADWAL HARI INI.
            PrayerTable { times: times, now: now, jumat: jumat, next_kind: next_kind }
        }
    }
}
