//! Root dom window utama: bar atas + layar sesuai tab aktif, sinkron tema
//! lintas dom, dan penerima perintah tampil dari widget.

use crate::state;
use crate::ui::components::title_bar::{Tab, TitleBar};
use crate::ui::screens::{about::AboutView, daily::DailyView, settings::SettingsView};
use crate::ui::tray::TraySetup;
use crate::ui::window;
use dioxus::prelude::*;

pub fn app() -> Element {
    let mut theme = use_signal(|| state::settings().theme);
    let lang = state::settings().language;
    let tr = crate::i18n::tr;
    // Tab aktif bar atas. Layar Bulanan & Kalender menyusul — untuk kini
    // keduanya menampilkan placeholder.
    let mut tab = use_signal(|| Tab::Pengaturan);

    // Sinkronkan tema dari dom lain (mis. toggle di widget window) tiap detik.
    let mut sync_tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            sync_tick += 1;
        }
    });
    let _ = sync_tick();
    if *theme.read() != state::settings().theme {
        theme.set(state::settings().theme);
    }

    // Perintah lintas window: tombol pengaturan di widget menampilkan window
    // utama. `dioxus::desktop::window()` di dom widget menunjuk widget, jadi
    // jalannya lewat channel ini (handler di dom utama).
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            while let Some(cmd) = state::pop_main_command() {
                match cmd {
                    state::MainCommand::ShowMain => {
                        window::show_main();
                    }
                }
            }
        }
    });

    // Saat startup hanya widget yang tampil — window pengaturan tetap
    // tersembunyi sampai dibuka lewat tray / tombol pengaturan di widget.
    use_future(move || async move {
        // Jeda sekejap agar inisialisasi menu tray (TraySetup) selesai dulu,
        // supaya centang "Widget Desktop" ikut tersinkron.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        window::show_widget();
    });

    // Window utama 1200×800: posisikan di tengah monitor sekali saat startup
    // (window masih tersembunyi), jadi saat dibuka ia sudah center. Pakai
    // use_hook (sekali) — use_effect jalan tiap render dan akan menyentak
    // window kembali ke tengah saat user sedang menggesernya.
    use_hook(|| {
        let win = dioxus::desktop::window();
        let Some(monitor) = win.current_monitor() else {
            return;
        };
        let mon_pos = monitor.position();
        let mon_size = monitor.size();
        let inner = win.inner_size();
        let x = mon_pos.x + (mon_size.width as i32 - inner.width as i32) / 2;
        let y = mon_pos.y + (mon_size.height as i32 - inner.height as i32) / 2;
        win.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(x, y));
    });

    let mut root_class = String::from(
        "h-screen w-screen flex flex-col overflow-hidden font-sans \
         bg-sheet text-ink transition-colors duration-300",
    );
    if theme.read().is_dark() {
        root_class.push_str(" dark");
    }

    rsx! {
        style { {crate::ui::TAILWIND_CSS} }
        TraySetup {}
        div { class: "{root_class}",
            TitleBar { tab: *tab.read(), on_select_tab: move |t: Tab| tab.set(t) }
            match tab() {
                Tab::Pengaturan => rsx! { SettingsView {} },
                Tab::Tentang => rsx! { AboutView {} },
                Tab::Harian => rsx! { DailyView {} },
                Tab::Bulanan | Tab::Kalender => rsx! {
                    div { class: "flex-1 flex items-center justify-center",
                        span { class: "font-mono text-xs font-semibold tracking-[0.14em] uppercase text-ink-3",
                            "{tr(lang, \"coming_soon\")}"
                        }
                    }
                },
            }
        }
    }
}
