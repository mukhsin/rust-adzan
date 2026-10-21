//! Layar Harian — kartu tiket yang sama persis dengan widget (identitas +
//! hitung mundur + papan jadwal), ditampilkan di tengah lembar dengan sudut
//! membulat + shadow. Mockup pen punya panel kanan yang berbeda, tapi
//! isinya sudah tercakup di kartu ini sehingga kartu widget dipakai langsung.

use crate::components::ticket::{Ticket, tanggal_labels};
use crate::i18n::tr;
use crate::prayer;
use crate::state;
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
        div { class: "flex-1 flex items-center justify-center p-4",
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
