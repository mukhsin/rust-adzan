use crate::settings::Settings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Settings global yang dibagikan antara UI utama, widget, dan scheduler thread.
static SETTINGS: OnceLock<RwLock<Arc<Settings>>> = OnceLock::new();

/// Permintaan "lewati adzan berikutnya" (menu tray): scheduler membuang
/// audio + toast masuk waktu pada pemicu adzan berikutnya sekali. Statusnya
/// terlihat di dua tempat: centang item tray dan lonceng-off papan jadwal
/// untuk sholat berikutnya.
static SKIP_NEXT_ADZAN: AtomicBool = AtomicBool::new(false);

pub fn request_skip_next_adzan() {
    SKIP_NEXT_ADZAN.store(true, Ordering::Relaxed);
}

/// Batalkan permintaan skip sebelum sempat terpakai.
pub fn cancel_skip_next_adzan() {
    SKIP_NEXT_ADZAN.store(false, Ordering::Relaxed);
}

/// Sedang armed (dipakai papan jadwal & tray tiap render).
pub fn skip_next_armed() -> bool {
    SKIP_NEXT_ADZAN.load(Ordering::Relaxed)
}

/// Ambil sekaligus kosongkan permintaan skip (sekali pakai).
pub fn take_skip_next_adzan() -> bool {
    SKIP_NEXT_ADZAN.swap(false, Ordering::Relaxed)
}

/// Set centang item tray "Lewati adzan berikutnya" (main thread saja —
/// item muda tidak Send). Pelepasan saat skip tidak aktif datang lewat
/// [`MainCommand::SkipInactive`].
pub fn set_tray_skip_checked(checked: bool) {
    TRAY_SKIP_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_checked(checked);
        }
    });
}

/// Perintah UI lintas window (dari dom widget / thread scheduler → dom utama).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainCommand {
    /// Sembunyikan widget dan tampilkan window utama.
    ShowMain,
    /// Lepas centang "Lewati adzan berikutnya" di tray — skip sudah tidak
    /// aktif (terkonsumsi scheduler atau dibatalkan dari papan jadwal).
    SkipInactive,
}

static MAIN_CMD_TX: OnceLock<tokio::sync::mpsc::UnboundedSender<MainCommand>> = OnceLock::new();
static MAIN_CMD_RX: OnceLock<Mutex<tokio::sync::mpsc::UnboundedReceiver<MainCommand>>> =
    OnceLock::new();

/// Kirim perintah ke window utama (aman dipanggil dari dom mana pun).
pub fn send_main_command(cmd: MainCommand) {
    let tx = MAIN_CMD_TX.get_or_init(|| {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = MAIN_CMD_RX.set(Mutex::new(rx));
        tx
    });
    let _ = tx.send(cmd);
}

/// Ambil satu perintah yang tertunda (dipoll window utama tiap 100 ms).
pub fn pop_main_command() -> Option<MainCommand> {
    MAIN_CMD_RX.get()?.lock().unwrap().try_recv().ok()
}

/// Handle tao Window milik widget (dibuat tersembunyi saat startup, di-show/hide sesuai setting).
pub static WIDGET_WINDOW: Mutex<Option<Arc<dioxus::desktop::tao::window::Window>>> =
    Mutex::new(None);

thread_local! {
    /// Handle CheckMenuItem "Widget" di tray. muda item bersifat !Send (Rc internal),
    /// tetapi handler tray dan setup selalu berjalan di main thread, jadi thread_local aman.
    pub static WIDGET_MENU_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::CheckMenuItem>>
        = const { std::cell::RefCell::new(None) };

    /// Handle item "Tampilkan Jendela" di tray — teksnya diganti saat bahasa UI berubah.
    pub static TRAY_SHOW_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::MenuItem>>
        = const { std::cell::RefCell::new(None) };

    /// Handle CheckMenuItem "Lewati adzan berikutnya" di tray — teksnya
    /// diganti saat bahasa UI berubah; centangnya mengikuti status armed.
    pub static TRAY_SKIP_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::CheckMenuItem>>
        = const { std::cell::RefCell::new(None) };

    /// Handle item "Keluar" di tray — teksnya diganti saat bahasa UI berubah.
    pub static TRAY_QUIT_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::MenuItem>>
        = const { std::cell::RefCell::new(None) };
}

pub fn settings() -> Arc<Settings> {
    let lock = SETTINGS.get_or_init(|| RwLock::new(Arc::new(crate::settings::load())));
    lock.read().expect("SETTINGS lock").clone()
}

/// Simpan settings baru: tulis global + simpan ke disk.
pub fn update_settings(new: Settings) {
    crate::settings::save(&new);
    let lock = SETTINGS.get_or_init(|| RwLock::new(Arc::new(crate::settings::load())));
    *lock.write().expect("SETTINGS lock") = Arc::new(new);
}
