//! Sakelar ala komponen "Sakelar" 36×20: isian ink saat nyala, kenop warna
//! on-ink; mati = isian redup dengan kenop pass.

use dioxus::prelude::*;

#[component]
pub fn Switch(checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        button {
            class: if checked {
                "w-9 h-5 rounded-full bg-ink transition-colors relative shrink-0 cursor-pointer"
            } else {
                "w-9 h-5 rounded-full bg-ink-3/40 transition-colors relative shrink-0 cursor-pointer"
            },
            onclick: move |_| on_change.call(!checked),
            span { class: if checked {
                "absolute top-[2px] left-[18px] w-4 h-4 rounded-full bg-on-ink shadow transition-all"
            } else {
                "absolute top-[2px] left-[2px] w-4 h-4 rounded-full bg-pass shadow transition-all"
            } }
        }
    }
}
