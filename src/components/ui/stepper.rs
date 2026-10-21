//! Stepper ala komponen "Stepper": − nilai + dalam kotak bergaris pass-line.
//! Tombol −/+ memakai tooltip custom design system pen (bukan `title`
//! native). [`OffsetStepper`] adalah varian bernilai bertanda untuk
//! koreksi waktu (−30..=30 menit).

use super::tooltip::{PosisiTooltip, Tooltip};
use crate::i18n::tr;
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn Stepper(
    value: u32,
    on_decrease: EventHandler<()>,
    on_increase: EventHandler<()>,
) -> Element {
    let lang = state::settings().language;
    rsx! {
        div { class: "flex items-center gap-2.5 border border-pass-line rounded-lg px-2.5 py-[5px]",
            Tooltip { label: tr(lang, "dec").to_string(), posisi: PosisiTooltip::Atas,
                button {
                    class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                    onclick: move |_| on_decrease.call(()),
                    "−"
                }
            }
            span { class: "font-mono text-xs font-semibold text-ink tabular-nums text-center min-w-[64px]",
                "{value} {tr(lang, \"unit_minutes\")}"
            }
            Tooltip { label: tr(lang, "inc").to_string(), posisi: PosisiTooltip::Atas,
                button {
                    class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                    onclick: move |_| on_increase.call(()),
                    "+"
                }
            }
        }
    }
}

/// Stepper bernilai bertanda untuk koreksi waktu (−30..=30 menit).
/// Nilai tampil eksplisit bertanda ("+2 menit" / "−3 menit") agar arah
/// koreksi terbaca; batas atas/bawah ditentukan pemanggil.
#[component]
pub fn OffsetStepper(
    value: i32,
    on_decrease: EventHandler<()>,
    on_increase: EventHandler<()>,
) -> Element {
    let lang = state::settings().language;
    let berlabel = if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    };
    rsx! {
        div { class: "flex items-center gap-2.5 border border-pass-line rounded-lg px-2.5 py-[5px]",
            Tooltip { label: tr(lang, "dec").to_string(), posisi: PosisiTooltip::Atas,
                button {
                    class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                    onclick: move |_| on_decrease.call(()),
                    "−"
                }
            }
            span { class: "font-mono text-xs font-semibold text-ink tabular-nums text-center min-w-[64px]",
                "{berlabel} {tr(lang, \"unit_minutes\")}"
            }
            Tooltip { label: tr(lang, "inc").to_string(), posisi: PosisiTooltip::Atas,
                button {
                    class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                    onclick: move |_| on_increase.call(()),
                    "+"
                }
            }
        }
    }
}
