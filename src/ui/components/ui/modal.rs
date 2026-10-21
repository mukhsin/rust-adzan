//! Modal dialog bertema aplikasi (pengganti dialog OS yang jelek di tema
//! gelap). Klik backdrop atau tombol batal = tutup tanpa aksi.

use super::button::{BTN, BTN_CONFIRM, BTN_DANGER};
use crate::i18n::tr;
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn Modal(
    title: String,
    message: String,
    confirm_text: String,
    danger: bool,
    show_cancel: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let lang = state::settings().language;
    rsx! {
        div {
            // Scrim gelap di KEDUA tema — bg-ink/60 akan jadi putih di tema
            // gelap karena ink terbalik (#ece9e2).
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "w-80 bg-pass border border-pass-line rounded-[14px] p-5 shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] flex flex-col gap-2",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-sm font-semibold text-ink", "{title}" }
                p { class: "text-xs text-ink-2 leading-relaxed", "{message}" }
                div { class: "flex justify-end gap-2 mt-3",
                    if show_cancel {
                        button {
                            class: "{BTN}",
                            onclick: move |_| on_cancel.call(()),
                            "{tr(lang, \"cancel\")}"
                        }
                    }
                    button {
                        class: if danger { "{BTN_DANGER}" } else { "{BTN_CONFIRM}" },
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_text}"
                    }
                }
            }
        }
    }
}
