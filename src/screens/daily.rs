//! Layar Harian — kartu tiket (identik dengan widget) berdampingan dengan
//! panel info: kartu Tanggal Penting (status Ramadan + acara terdekat) dan
//! kartu Arah Kiblat (bearing + jarum kompas). Kartu memakai bahasa visual
//! kartu grup pengaturan (bg-pass, radius 14, shadow lembut).

use crate::components::ticket::{Ticket, tanggal_labels};
use crate::dates::{self, RamadanStatus};
use crate::i18n::tr;
use crate::prayer;
use crate::qibla;
use crate::state;
use chrono::NaiveDate;
use dioxus::prelude::*;

#[component]
pub fn DailyView() -> Element {
    // Hitung mundur harus berdetak — tick 1 detik seperti di widget.
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });
    let _ = tick();

    let settings = state::settings();
    let lang = settings.language;
    let now = prayer::now_in_tz(&settings);
    let tz = now.format("%Z").to_string();
    let (hari, tanggal, tanggal_hijri) = tanggal_labels(lang, &now);

    rsx! {
        div { class: "flex-1 flex items-center justify-center gap-[18px] p-4",
            match prayer::day_times(&settings, now.date_naive()) {
                Some(times) => rsx! {
                    // Ukuran alami kartu widget; dibulatkan + diberi shadow
                    // agar duduk sebagai kartu di atas lembar.
                    div { class: "w-[600px] h-[300px] rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] overflow-hidden",
                        Ticket {
                            times: times,
                            now: now.time(),
                            tz: tz,
                            hari: hari,
                            tanggal: tanggal,
                            tanggal_hijri: tanggal_hijri,
                        }
                    }
                    div { class: "w-[320px] h-[300px] flex flex-col gap-[18px]",
                        DatesCard { today: now.date_naive() }
                        QiblaCard { lat: settings.lat, lng: settings.lng }
                    }
                },
                None => rsx! {
                    div { class: "w-[600px] h-[300px] rounded-[14px] flex items-center justify-center bg-pass shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080]",
                        span { class: "text-sm text-ink-3", "{tr(lang, \"calc_failed\")}" }
                    }
                },
            }
        }
    }
}

/// Kartu Tanggal Penting: status Ramadan (menunggu / hari ke-N) + tiga
/// peristiwa berikutnya dengan sisa hari.
#[component]
fn DatesCard(today: NaiveDate) -> Element {
    let lang = state::settings().language;
    let ramadan = dates::ramadan_status(today);
    let acara = dates::upcoming(today, 3);

    let ramadan_line = match ramadan {
        RamadanStatus::Berlangsung { day } => tr(lang, "ramadan_day")
            .replace("{n}", &day.to_string())
            .replace("{year}", &today_hijri_year(today).to_string()),
        RamadanStatus::Menunggu { days, year } => {
            tr(lang, "days_until_ramadan").replace("{n}", &days.to_string())
                + &format!(" · {year} H")
        }
    };

    rsx! {
        div { class: "flex-1 bg-pass rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] px-[22px] pt-3.5 pb-2 flex flex-col justify-between",
            div { class: "font-mono text-[11px] font-semibold tracking-[0.14em] uppercase text-ink-3",
                "{tr(lang, \"sec_dates\")}"
            }

            // Status Ramadan — sorotan utama kartu.
            div { class: "flex items-center gap-2 mt-1.5",
                span { class: "w-1.5 h-1.5 rounded-full bg-amber shrink-0" }
                span { class: "text-[13.5px] font-bold text-ink", "{ramadan_line}" }
            }

            // Tiga peristiwa berikutnya.
            div { class: "flex flex-col",
                for acara in acara {
                    {
                        let sisa = (acara.date - today).num_days();
                        let tanggal = acara.date.format("%-d %b").to_string();
                        let sisa_label = if sisa == 0 {
                            tr(lang, "today").to_string()
                        } else {
                            tr(lang, "in_days").replace("{n}", &sisa.to_string())
                        };
                        let nama = tr(lang, acara.kind.key());
                        rsx! {
                            div { class: "flex items-center justify-between gap-2 py-[5px] border-t border-hairline first:border-t-0",
                                div { class: "flex items-center gap-2 min-w-0",
                                    span { class: "text-[12.5px] font-semibold text-ink-2 truncate",
                                        title: "{nama}",
                                        "{nama}"
                                    }
                                    span { class: "font-mono text-[10px] text-ink-3", "{tanggal}" }
                                }
                                span { class: "font-mono text-[10.5px] font-semibold text-amber shrink-0",
                                    "{sisa_label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Kartu Arah Kiblat: bearing derajat + arah mata angin + jarum kompas
/// (SVG dirotasi sesuai bearing; atas = utara).
#[component]
fn QiblaCard(lat: f64, lng: f64) -> Element {
    let lang = state::settings().language;
    let deg = qibla::bearing(lat, lng);
    let arah = tr(lang, qibla::arah_key(deg));

    rsx! {
        div { class: "flex-1 bg-pass rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] px-[22px] pt-3.5 pb-2.5 flex flex-col",
            div { class: "font-mono text-[11px] font-semibold tracking-[0.14em] uppercase text-ink-3",
                "{tr(lang, \"sec_qibla\")}"
            }
            div { class: "flex-1 flex items-center justify-between gap-3",
                div { class: "flex flex-col gap-0.5",
                    span { class: "font-mono text-[26px] font-bold tabular-nums text-ink",
                        "{deg:.1}°"
                    }
                    span { class: "text-[12.5px] font-semibold text-ink-2", "{arah}" }
                    span { class: "text-[10.5px] text-ink-3", "{tr(lang, \"qibla_from_north\")}" }
                }
                // Kompas: lingkaran + jarum menuju bearing (0° = atas).
                div { class: "w-[64px] h-[64px] rounded-full border border-pass-line flex items-center justify-center shrink-0",
                    div { class: "w-full h-full flex items-center justify-center",
                        style: "transform: rotate({deg:.1}deg)",
                        svg {
                            view_box: "0 0 24 24",
                            class: "w-8 h-8 text-amber",
                            fill: "currentColor",
                            path { d: "M12 3 L16 17 L12 14.4 L8 17 Z" }
                        }
                    }
                }
            }
        }
    }
}

/// Tahun hijriah hari ini — untuk label "Hari ke-N Ramadhan".
fn today_hijri_year(today: NaiveDate) -> i32 {
    crate::hijri::to_hijri(today).0
}
