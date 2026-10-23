//! Pemilih bulan & tahun — sepasang droplist (ala pengaturan) pengganti
//! navigasi chevron di kepala layar Kalender & Bulanan: kiri memilih
//! bulan (Januari–Desember), kanan memilih tahun (±10 tahun dari tahun
//! berjalan).

use crate::components::ui::drop_search::DropSearch;
use crate::i18n::month_name;
use crate::prayer;
use crate::state;
use chrono::Datelike;
use dioxus::prelude::*;

#[component]
pub fn PemilihBulan(year: i32, month: u32, on_change: EventHandler<(i32, u32)>) -> Element {
    let lang = state::settings().language;
    let tahun_kini = prayer::now_in_tz(&state::settings()).year();

    let opsi_bulan: Vec<(String, String)> = (1..=12)
        .map(|m| {
            (
                m.to_string(),
                month_name(lang, (m - 1) as usize).to_string(),
            )
        })
        .collect();
    let opsi_tahun: Vec<(String, String)> = ((tahun_kini - 10)..=(tahun_kini + 10))
        .map(|y| (y.to_string(), y.to_string()))
        .collect();

    rsx! {
        div { class: "flex items-center gap-2",
            DropSearch {
                // Trigger memakai label ramah yang sama dengan item daftar.
                value: month_name(lang, (month - 1) as usize).to_string(),
                active: month.to_string(),
                options: opsi_bulan,
                searchable: false,
                on_select: move |v: String| {
                    if let Ok(m) = v.parse::<u32>() {
                        on_change.call((year, m));
                    }
                },
            }
            DropSearch {
                value: year.to_string(),
                active: year.to_string(),
                options: opsi_tahun,
                searchable: false,
                on_select: move |v: String| {
                    if let Ok(y) = v.parse::<i32>() {
                        on_change.call((y, month));
                    }
                },
            }
        }
    }
}
