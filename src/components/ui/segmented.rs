//! Tersegmen ala komponen "Tersegmen": lajur board pill, segmen aktif ink
//! dengan teks on-ink. Untuk pilihan pendek (tema, format jam, bahasa).

use dioxus::prelude::*;

#[component]
pub fn Segmented(
    options: Vec<(String, String)>,
    active: String,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-0.5 bg-board rounded-full p-[3px]",
            for (value, label) in options {
                {
                    let is_active = value == active;
                    rsx! {
                        button {
                            class: if is_active {
                                "px-3.5 py-[5px] rounded-full bg-ink text-on-ink text-[12.5px] font-bold transition-colors cursor-pointer"
                            } else {
                                "px-3.5 py-[5px] rounded-full text-[12.5px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer"
                            },
                            onclick: move |_| on_select.call(value.clone()),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
