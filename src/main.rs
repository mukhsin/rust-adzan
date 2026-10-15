// Aplikasi GUI: tanpa jendela console di build release.
// (Debug tetap pakai console agar println!/eprintln! terlihat saat dev.)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod autostart;
mod cities;
mod logger;
mod notify;
mod prayer;
mod scheduler;
mod settings;
mod state;
mod ui;

use dioxus::desktop::{Config, WindowBuilder, WindowCloseBehaviour, tao::dpi::LogicalSize};

fn main() {
    logger::init();

    // Muat settings ke global + nyalakan engine adzan (thread terpisah).
    let _ = state::settings();
    scheduler::start();
    logger::info("aplikasi dimulai");

    // Sinkronkan auto-start dengan settings (memperbarui path exe yang bisa usang).
    if state::settings().autostart {
        let _ = autostart::set_enabled(true);
    }

    let window_icon =
        dioxus::desktop::tao::window::Icon::from_rgba(ui::WINDOW_ICON_RGBA.to_vec(), 256, 256)
            .expect("ikon window valid");

    let config = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title(crate::settings::APP_NAME)
                .with_decorations(false) // frameless: title bar custom di ui::TitleBar
                .with_inner_size(LogicalSize::new(400.0, 700.0))
                .with_min_inner_size(LogicalSize::new(380.0, 620.0)),
        )
        .with_icon(window_icon)
        // Tombol close hanya menyembunyikan window; app tetap jalan di tray.
        .with_close_behaviour(WindowCloseBehaviour::WindowHides)
        .with_exits_when_last_window_closes(false)
        // Handler bawaan dioxus menampilkan SEMUA window (termasuk widget yang
        // sengaja disembunyikan) saat tray icon diklik. Matikan; kita tangani
        // sendiri agar hanya window utama yang muncul.
        .with_tray_icon_show_window_on_click(false);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui::app);
}
