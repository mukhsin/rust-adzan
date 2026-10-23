//! Pane kiri kartu tiket: identitas + kota, hitung mundur ke sholat
//! berikutnya, dan tiga label tanggal (hari / masehi / hijri).
//! Saat Ramadhan (sebelum Maghrib): badge hari ke-N dan hitung mundur
//! berlabel "Menuju Berbuka".

use crate::i18n::tr;
use crate::prayer::{self, DayTimes, PrayerKind};
use crate::state;
use chrono::NaiveTime;
use dioxus::prelude::*;

#[component]
pub fn CountdownPane(
    times: DayTimes,
    now: NaiveTime,
    jumat: bool,
    tz: String,
    hari: String,
    tanggal: String,
    tanggal_hijri: String,
    ramadan_day: Option<u32>,
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
    // Saat puasa dan menunggu Maghrib, berbuka lebih bermakna daripada
    // nama sholatnya; Jumat: Dzuhur berlabel "Sholat Jumat"; selebihnya
    // label standar.
    let menuju = if ramadan_day.is_some() && next_kind == PrayerKind::Maghrib {
        tr(lang, "countdown_berbuka").to_string()
    } else if jumat && next_kind == PrayerKind::Dzuhur {
        format!(
            "{} {}",
            tr(lang, "countdown_prefix"),
            tr(lang, "sholat_jumat")
        )
    } else {
        format!("{} {}", tr(lang, "countdown_prefix"), next_kind.nama(lang))
    };
    let masuk = tr(lang, "masuk_pukul").replace("{waktu}", &jam_target);
    let badge_ramadan =
        ramadan_day.map(|day| tr(lang, "ramadan_badge").replace("{n}", &day.to_string()));

    rsx! {
        div { class: "flex-1",
            div { class: "flex flex-col justify-between p-6 h-full",
                div { class: "flex flex-col gap-1.5 min-w-0",
                    span {
                        class: "font-mono text-[10.5px] font-semibold tracking-[0.08em] uppercase text-ink-3 truncate",
                        "{tr(lang, \"app_name\")} · {kota}"
                    }
                    if let Some(badge) = badge_ramadan {
                        span { class: "font-mono text-[10.5px] font-semibold tracking-[0.08em] uppercase text-amber truncate",
                            "{badge}"
                        }
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
    }
}
