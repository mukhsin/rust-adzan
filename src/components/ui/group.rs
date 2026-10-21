//! Kartu grup + baris setting — ala komponen "Grup Pengaturan" dan
//! "Baris Pengaturan" design system pen.

use super::tooltip::{PosisiTooltip, Tooltip};
use dioxus::prelude::*;

/// Kartu grup setting: pass, radius 14, shadow lembut, label mono uppercase
/// ink-3, padding [14, 26, 10, 26].
#[component]
pub fn Group(label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "bg-pass rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] px-[26px] pt-3.5 pb-2.5 flex flex-col",
            div { class: "font-mono text-[11px] font-semibold tracking-[0.14em] uppercase text-ink-3 pb-1",
                "{label}"
            }
            {children}
        }
    }
}

/// Baris setting: kiri label + keterangan, kanan kontrol; pemisah hairline
/// bawah (baris terakhir tanpa garis). Tinggi baris DIBUAT SERAGAM (58px)
/// agar grup bersebelahan di kolom kiri/kanan sejajar. `tooltip` (opsi)
/// menampilkan bubble custom di atas label saat hover — penjelasan setting
/// ala design system pen.
#[component]
pub fn Row(
    label: &'static str,
    hint: Option<String>,
    tooltip: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3 h-[58px] shrink-0 border-b border-hairline last:border-b-0",
            div { class: "flex flex-col gap-0.5 min-w-0",
                if let Some(tip) = tooltip {
                    Tooltip { label: tip, posisi: PosisiTooltip::Atas,
                        span { class: "text-[13.5px] font-semibold text-ink cursor-help", "{label}" }
                    }
                } else {
                    span { class: "text-[13.5px] font-semibold text-ink", "{label}" }
                }
                if let Some(h) = hint {
                    span { class: "text-[11.5px] text-ink-3 truncate", "{h}" }
                }
            }
            div { class: "flex items-center gap-2 shrink-0",
                {children}
            }
        }
    }
}
