pub mod about_view;
pub mod icons;
pub mod settings_view;
pub mod widget;

use crate::state;
use dioxus::desktop::trayicon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{init_tray_icon, use_tray_icon};
use dioxus::desktop::use_muda_event_handler;
use dioxus::desktop::use_tray_icon_event_handler;
use dioxus::desktop::{
    Config, WindowBuilder,
    tao::dpi::{LogicalSize, PhysicalPosition},
    tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    tao::window::Window,
};
use dioxus::prelude::*;
use state::WIDGET_MENU_ITEM;

/// CSS hasil kompilasi Tailwind (assets/tailwind.css, dibangun via tools/build-css.sh).
pub static TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");
/// Data piksel RGBA (256×256) untuk ikon window/taskbar.
pub static WINDOW_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon.rgba");
/// Data piksel RGBA (32×32) untuk tray icon.
pub static TRAY_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon-tray.rgba");

/// Margin widget dari tepi monitor (px logical) — dipakai saat posisi default
/// dan clamp saat drag.
pub(crate) const WIDGET_MARGIN: f64 = 16.0;
/// Ukuran window widget (px logical) — window = kartu tiket, tanpa gap.
pub(crate) const WIDGET_SIZE: (f64, f64) = (600.0, 300.0);

pub fn app() -> Element {
    let mut theme = use_signal(|| state::settings().theme);
    let lang = state::settings().language;
    let tr = crate::i18n::tr;
    // Tab aktif bar atas. Layar Harian & Bulanan menyusul — untuk kini
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
                        let win = dioxus::desktop::window();
                        win.set_minimized(false);
                        win.set_visible(true);
                        win.set_focus();
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
        show_widget();
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
        win.set_outer_position(PhysicalPosition::new(x, y));
    });

    let mut root_class = String::from(
        "h-screen w-screen flex flex-col overflow-hidden font-sans \
         bg-sheet text-ink transition-colors duration-300",
    );
    if theme.read().is_dark() {
        root_class.push_str(" dark");
    }

    rsx! {
        style { {TAILWIND_CSS} }
        TraySetup {}
        div { class: "{root_class}",
            TitleBar { tab: *tab.read(), on_select_tab: move |t: Tab| tab.set(t) }
            match tab() {
                Tab::Pengaturan => rsx! { settings_view::SettingsView {} },
                Tab::Tentang => rsx! { about_view::AboutView {} },
                Tab::Harian | Tab::Bulanan | Tab::Kalender => rsx! {
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

/// Tab aplikasi pada bar atas (design system pen). Layar Harian, Bulanan,
/// dan Kalender menyusul — lihat placeholder di `app()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Harian,
    Bulanan,
    Kalender,
    Pengaturan,
    Tentang,
}

/// Bar atas window utama ala komponen "Bar Atas" di design system pen:
/// brand + tab kiri, kontrol jendela (minimize / maximize / close) kanan.
/// Seluruh bar bisa dipakai drag window; tab & tombol tidak ikut drag.
#[component]
pub fn TitleBar(tab: Tab, on_select_tab: EventHandler<Tab>) -> Element {
    let lang = state::settings().language;
    let tr = crate::i18n::tr;
    let tabs = [
        (Tab::Harian, tr(lang, "tab_daily")),
        (Tab::Bulanan, tr(lang, "tab_monthly")),
        (Tab::Kalender, tr(lang, "tab_kalender")),
        (Tab::Pengaturan, tr(lang, "settings_title")),
        (Tab::Tentang, tr(lang, "tab_tentang")),
    ];

    rsx! {
        div {
            class: "shrink-0 flex items-center gap-6 p-2 select-none cursor-default",
            onmousedown: move |_| { dioxus::desktop::window().drag(); },

            div { class: "flex items-center gap-2.5 px-4",
                icons::Mosque { class: "w-[22px] h-[22px] text-ink" }
                span { class: "text-base font-bold tracking-wide text-ink", "Jadwal Sholat" }
            }

            div { class: "flex items-center gap-4",
                for (t, label) in tabs {
                    {
                        let active = t == tab;
                        rsx! {
                            button {
                                class: if active {
                                    "flex flex-col gap-[5px] px-2 py-1.5 border-b-[2.5px] border-ink text-[13.5px] font-bold text-ink cursor-pointer"
                                } else {
                                    "flex flex-col gap-[5px] px-2 py-1.5 border-b-[2.5px] border-transparent text-[13.5px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer"
                                },
                                onmousedown: move |e| e.stop_propagation(),
                                onclick: move |_| on_select_tab.call(t),
                                "{label}"
                            }
                        }
                    }
                }
            }

            div { class: "flex-1" }

            div { class: "flex items-center gap-0.5",
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-ink/5 transition-colors cursor-pointer",
                    title: tr(lang, "win_minimize"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().set_minimized(true),
                    icons::IconMinus { class: "w-[15px] h-[15px]" }
                }
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-ink/5 transition-colors cursor-pointer",
                    title: tr(lang, "win_maximize"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| {
                        let win = dioxus::desktop::window();
                        win.set_maximized(!win.is_maximized());
                    },
                    icons::IconSquare { class: "w-3 h-3" }
                }
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-red-500 hover:text-white transition-colors cursor-pointer",
                    title: tr(lang, "close_hint"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().set_visible(false),
                    icons::IconClose { class: "w-[15px] h-[15px]" }
                }
            }
        }
    }
}

/// Inisialisasi system tray + handler menu tray (sekali saja).
#[component]
fn TraySetup() -> Element {
    // Inisialisasi tray (sekali) — jangan memanggil hook lain di dalam use_hook.
    use_hook(|| {
        let lang = state::settings().language;
        let menu = Menu::new();
        let show = MenuItem::with_id("show", crate::i18n::tr(lang, "menu_show"), true, None);
        let widget = CheckMenuItem::with_id(
            "widget",
            crate::i18n::tr(lang, "menu_widget"),
            true,
            false,
            None,
        );
        let quit = MenuItem::with_id("quit", crate::i18n::tr(lang, "menu_quit"), true, None);
        menu.append_items(&[
            &show,
            &PredefinedMenuItem::separator(),
            &widget,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("isi menu tray");

        let icon = dioxus::desktop::trayicon::Icon::from_rgba(TRAY_ICON_RGBA.to_vec(), 32, 32)
            .expect("tray icon");
        let _tray = init_tray_icon(menu, Some(icon));
        WIDGET_MENU_ITEM.with(|cell| *cell.borrow_mut() = Some(widget));
        state::TRAY_SHOW_ITEM.with(|cell| *cell.borrow_mut() = Some(show));
        state::TRAY_QUIT_ITEM.with(|cell| *cell.borrow_mut() = Some(quit));
        TRAY_LANG.with(|cell| cell.set(Some(lang)));
    });

    // Hook di level atas komponen, bukan di dalam hook lain.
    let tray = use_tray_icon();

    // Menu tray ikut ganti bahasa tanpa restart (render = main thread).
    sync_tray_language(state::settings().language);

    // Klik kiri ikon tray: toggle widget desktop.
    // Event `Click` di-emit dua kali (press & release) — reaksi hanya saat
    // release supaya toggle tidak langsung dibatalkan oleh event kedua.
    use_tray_icon_event_handler(|event| {
        if let dioxus::desktop::trayicon::TrayIconEvent::Click {
            button: dioxus::desktop::trayicon::MouseButton::Left,
            button_state: dioxus::desktop::trayicon::MouseButtonState::Up,
            ..
        } = event
        {
            toggle_widget();
        }
    });

    // PENTING: event menu tray di dioxus-desktop 0.7 datang sebagai `MudaMenuEvent`,
    // bukan `TrayMenuEvent` — handler `muda::MenuEvent` (menubar) dipasang lebih
    // dulu dan muda menyimpannya di OnceCell, sehingga pemasangan handler kedua
    // dioxus untuk `tray_icon::menu::MenuEvent` gagal diam-diam. Karena itu
    // `use_tray_menu_event_handler` tidak pernah terpanggil.
    use_muda_event_handler(move |event| match event.id().0.as_str() {
        "show" => show_main(),
        "widget" => toggle_widget(),
        "quit" => {
            if let Some(tray) = &tray {
                let _ = tray.set_visible(false);
            }
            std::process::exit(0);
        }
        _ => {}
    });

    rsx! {}
}

thread_local! {
    /// Bahasa yang sedang tampil di menu tray — untuk deteksi perubahan.
    static TRAY_LANG: std::cell::Cell<Option<crate::settings::Language>>
        = const { std::cell::Cell::new(None) };
}

/// Ganti teks item menu tray bila bahasa settings berubah.
/// Dipanggil dari render TraySetup (main thread — item muda tidak Send).
fn sync_tray_language(lang: crate::settings::Language) {
    let changed = TRAY_LANG.with(|cell| {
        if cell.get() != Some(lang) {
            cell.set(Some(lang));
            true
        } else {
            false
        }
    });
    if !changed {
        return;
    }
    use crate::i18n::tr;
    WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_widget"));
        }
    });
    state::TRAY_SHOW_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_show"));
        }
    });
    state::TRAY_QUIT_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_quit"));
        }
    });
}

/// Buat widget window (lazy — dipanggil saat pertama kali widget dibutuhkan).
/// Setelah webview selesai load, dioxus otomatis memvisibilitaskan window ini
/// (flag app-level `is_visible_before_start`), pas dengan saat ia diminta tampil.
fn spawn_widget_creation() {
    spawn(async move {
        let main_win = dioxus::desktop::window();
        let (w, h) = WIDGET_SIZE;
        let mut builder = WindowBuilder::new()
            .with_decorations(false)
            .with_resizable(false)
            .with_skip_taskbar(true)
            .with_visible(false)
            // Transparan: lubang perforasi (mask CSS) benar-benar "bolong"
            // ke desktop. Tanpa redirection bitmap, permukaan putih window
            // tidak menutupi piksel transparan webview. Kartu mengisi window
            // penuh — tidak ada margin.
            .with_transparent(true)
            .with_no_redirection_bitmap(true)
            .with_inner_size(LogicalSize::new(w, h));

        // Posisi default: pojok kanan-atas monitor, margin WIDGET_MARGIN.
        // (Tanpa always-on-top supaya tidak menghalangi window aktif.)
        if let Some(monitor) = main_win.current_monitor() {
            let scale = monitor.scale_factor();
            let mon_pos = monitor.position();
            let mon_size = monitor.size();
            let m = (WIDGET_MARGIN * scale).round() as i32;
            let x = mon_pos.x + mon_size.width as i32 - (w * scale).round() as i32 - m;
            let y = mon_pos.y + m;
            builder = builder.with_position(PhysicalPosition::new(x, y));
        }

        let cfg = Config::new()
            .with_window(builder)
            .with_exits_when_last_window_closes(false);

        let ctx = main_win
            .new_window(VirtualDom::new(widget::WidgetApp), cfg)
            .await;
        *state::WIDGET_WINDOW.lock().unwrap() = Some(ctx.window.clone());
        // Rapikan posisi awal (kompensasi border DWM di clamp) sebelum tampil.
        clamp_widget_position(&ctx.window);
        // Window TIDAK dimunculkan di sini: saat webview selesai load, dioxus
        // memanggil set_visible(is_visible_before_start) yang nilainya false
        // (window utama kini mulai tersembunyi) sehingga akan menutup widget
        // lagi. Widget memunculkan dirinya sendiri via use_effect di WidgetApp
        // setelah event initialize — sekaligus menghindari kedipan putih.
    });
}

/// Snap posisi widget ke dalam monitor dengan margin `WIDGET_MARGIN` di semua
/// sisi (batas terdekat ke tepi monitor 16px). Dipanggil saat drag selesai dan
/// setelah ganti ukuran — selama drag, window bebas mengikuti kursor.
/// Monitor yang lebih kecil dari window + margin di-fallback agar `clamp`
/// tidak panik.
pub(crate) fn clamp_widget_position(window: &Window) {
    let Some(monitor) = window.current_monitor() else {
        return;
    };
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let scale = monitor.scale_factor();
    let m = (WIDGET_MARGIN * scale).round() as i32;
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let inner = window.inner_size();
    let outer = window.outer_size();
    // Rect window di Windows menyertakan border invisible DWM kiri/kanan
    // (~9px fisik per sisi) sehingga konten terlihat mulai `border_left` dari
    // sisi kiri rect. Kompensasi supaya margin visual kiri/kanan sama dengan
    // atas/bawah. (Vertikal tidak berborder, jadi tidak perlu kompensasi.)
    let border_left = (outer.width as i32 - inner.width as i32) / 2;
    let min_x = mon_pos.x + m - border_left;
    let min_y = mon_pos.y + m;
    let max_x =
        (mon_pos.x + mon_size.width as i32 - border_left - inner.width as i32 - m).max(min_x);
    let max_y = (mon_pos.y + mon_size.height as i32 - inner.height as i32 - m).max(min_y);
    let clamped = PhysicalPosition::new(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y));
    if clamped != pos {
        window.set_outer_position(clamped);
    }
}

/// Rapikan frame window widget via DWM. SENGAJA TIDAK DIPAKAI: percobaan
/// DWMWCP_ROUND + DWMWA_BORDER_COLOR ternyata menyisakan garis gelap 1px di
/// tepi window yang terlihat menembus lubang perforasi. Dengan window
/// transparan + redirection bitmap mati, sudut membulat cukup dari CSS
/// (`rounded-[14px]` di kartu) dan DWM tidak menyentuh window sama sekali.
#[allow(dead_code)]
#[cfg(windows)]
fn round_widget_window(window: &Window) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };
    let hwnd = HWND(window.hwnd() as *mut _);
    let pref = DWMWCP_ROUND.0 as u32;
    // Warna border alpha-0 (0x00000000) = tanpa border.
    let no_border: u32 = 0;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const u32 as *const _,
            std::mem::size_of::<u32>() as _,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &no_border as *const u32 as *const _,
            std::mem::size_of::<u32>() as _,
        );
    }
}

#[cfg(not(windows))]
fn round_widget_window(_window: &Window) {}

/// Pantau tombol kiri mouse selama drag native widget berjalan; begitu
/// dilepas, snap posisi widget ke margin 16px semua sisi. Diperlukan karena
/// `Window::drag()` di Windows async (PostMessage WM_NCLBUTTONDOWN) — fungsi
/// itu kembali seketika dan tidak memberi tahu kapan drag selesai.
pub(crate) fn watch_widget_drag_release() {
    let Some(window) = state::WIDGET_WINDOW.lock().unwrap().clone() else {
        return;
    };
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        // Batas aman 60 detik: drag normal jauh lebih cepat dari itu.
        while start.elapsed() < std::time::Duration::from_secs(60) {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if !left_button_held() {
                break;
            }
        }
        clamp_widget_position(&window);
    });
}

/// True selagi tombol kiri mouse sedang ditahan.
#[cfg(windows)]
fn left_button_held() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16) & 0x8000 != 0 }
}

#[cfg(not(windows))]
fn left_button_held() -> bool {
    false
}

/// Tampilkan widget (window-nya dibuat lazy saat pertama kali).
pub fn show_widget() {
    WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_checked(true);
        }
    });
    if let Some(w) = state::WIDGET_WINDOW.lock().unwrap().as_ref() {
        w.set_visible(true);
        w.set_focus();
    } else {
        spawn_widget_creation();
    }
}

/// Sembunyikan widget (jika ada) + sinkron centang tray.
pub fn hide_widget() {
    WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_checked(false);
        }
    });
    if let Some(w) = state::WIDGET_WINDOW.lock().unwrap().as_ref() {
        w.set_visible(false);
    }
}

/// Tampilkan window utama (Pengaturan) tanpa mengubah widget — main window
/// dan widget boleh tampil bersamaan (main window kini khusus pengaturan).
pub fn show_main() {
    let win = dioxus::desktop::window();
    win.set_minimized(false);
    win.set_visible(true);
    win.set_focus();
}

/// Toggle widget dari tray/klik kiri ikon (tidak menyentuh window utama).
fn toggle_widget() {
    let visible = state::WIDGET_WINDOW
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|w| w.is_visible());
    if visible {
        hide_widget();
    } else {
        show_widget();
    }
}
