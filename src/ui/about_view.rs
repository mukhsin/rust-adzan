//! Layar Tentang — restyle dari frame "Tentang" di design system pen:
//! logo mosque asli, nama + tagline, pil versi, kartu INFORMASI
//! (kunci–nilai), dan baris footer mono. Semua konten center.

use super::icons;
use crate::i18n::tr;
use crate::settings::CalcMethod;
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn AboutView() -> Element {
    let lang = state::settings().language;
    let settings = state::settings();
    let versi = tr(lang, "version_label").replace("{versi}", env!("CARGO_PKG_VERSION"));

    // Baris kartu INFORMASI (kunci, nilai) — nilai perhitungan dinamis
    // mengikuti metode yang aktif di settings.
    let info = [
        (
            tr(lang, "sec_calc"),
            format!(
                "{} · {}",
                tr(lang, "about_astronomis"),
                method_short(settings.method)
            ),
        ),
        (tr(lang, "about_hijri"), "Umm al-Qura".to_string()),
        (tr(lang, "f_language"), "Indonesia · English".to_string()),
        (
            tr(lang, "about_adzan"),
            tr(lang, "about_adzan_value").to_string(),
        ),
        (
            tr(lang, "about_source"),
            "github.com/mukhsin/rust-adzan".to_string(),
        ),
    ];

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-[26px] overflow-y-auto px-4",

            // Logo mosque asli — SVG inline warna asli tiap segmen (bukan
            // glyph material ala mockup pen dan bukan versi currentColor
            // di bar atas).
            icons::MosqueLogo { class: "w-[76px] h-[76px] shrink-0" }

            span { class: "text-2xl font-bold tracking-wide uppercase text-ink",
                "{tr(lang, \"app_name\")}"
            }
            span { class: "text-[13.5px] text-ink-2 text-center",
                "{tr(lang, \"about_tagline\")}"
            }

            // Pil versi: titik amber + label mono.
            div { class: "flex items-center gap-2 bg-pass border border-pass-line rounded-full px-3.5 py-[7px]",
                span { class: "w-1.5 h-1.5 rounded-full bg-amber" }
                span { class: "font-mono text-[10.5px] font-semibold tracking-[0.14em] uppercase text-ink-2",
                    "{versi}"
                }
            }

            // Kartu INFORMASI — sama dengan kartu grup pengaturan, lebar
            // tetap 470px ala mockup.
            div { class: "w-[470px] bg-pass rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] px-[26px] pt-3.5 pb-2.5 flex flex-col",
                div { class: "font-mono text-[11px] font-semibold tracking-[0.14em] uppercase text-ink-3 pb-1",
                    "{tr(lang, \"about_info\")}"
                }
                for (kunci, nilai) in info {
                    div { class: "flex items-center justify-between gap-3 py-[9px] border-b border-hairline last:border-b-0",
                        span { class: "text-[13px] text-ink-2", "{kunci}" }
                        span { class: "font-mono text-xs font-semibold text-ink", "{nilai}" }
                    }
                }
            }

            span { class: "font-mono text-[9.5px] tracking-[0.14em] uppercase text-ink-3",
                "{tr(lang, \"about_built\")}"
            }
        }
    }
}

/// Nama pendek metode perhitungan untuk layar Tentang (versi panjang ada
/// di dropdown settings).
fn method_short(m: CalcMethod) -> &'static str {
    match m {
        CalcMethod::Kemenag => "Kemenag",
        CalcMethod::MuslimWorldLeague => "Muslim World League",
        CalcMethod::Egyptian => "Egyptian",
        CalcMethod::Karachi => "Karachi",
        CalcMethod::UmmAlQura => "Umm al-Qura",
        CalcMethod::NorthAmerica => "ISNA",
    }
}
