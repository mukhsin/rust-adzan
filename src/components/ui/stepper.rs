//! Stepper ala komponen "Stepper": − nilai + dalam kotak bergaris pass-line.

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
            button {
                class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                title: tr(lang, "dec"),
                onclick: move |_| on_decrease.call(()),
                "−"
            }
            span { class: "font-mono text-xs font-semibold text-ink tabular-nums text-center min-w-[64px]",
                "{value} {tr(lang, \"unit_minutes\")}"
            }
            button {
                class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                title: tr(lang, "inc"),
                onclick: move |_| on_increase.call(()),
                "+"
            }
        }
    }
}
