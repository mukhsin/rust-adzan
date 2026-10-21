//! Manajemen window: pembuatan widget window, snap/clamp posisi, dan
//! perintah tampil/sembunyi dari tray maupun dom lain.

use crate::state;
use crate::ui::components::widget::WidgetApp;
use dioxus::desktop::tao::dpi::{LogicalSize, PhysicalPosition};
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::tao::window::Window;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

/// Margin widget dari tepi monitor (px logical) — dipakai saat posisi default
/// dan clamp saat drag.
pub(crate) const WIDGET_MARGIN: f64 = 16.0;
/// Ukuran window widget (px logical) — window = kartu tiket, tanpa gap.
pub(crate) const WIDGET_SIZE: (f64, f64) = (600.0, 300.0);

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

        let ctx = main_win.new_window(VirtualDom::new(WidgetApp), cfg).await;
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
    state::WIDGET_MENU_ITEM.with(|cell| {
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
    state::WIDGET_MENU_ITEM.with(|cell| {
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
pub fn toggle_widget() {
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
