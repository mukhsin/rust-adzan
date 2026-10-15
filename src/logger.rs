//! Logging sederhana ke `%APPDATA%\jadwal-sholat\app.log`.
//!
//! - Selalu menulis ke file (release maupun debug).
//! - Di build debug, baris log juga dicetak ke console.
//! - File dirotasi ke `app.log.old` bila melebihi batas ukuran.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Batas ukuran log sebelum dirotasi (512 KB).
const MAX_LOG_BYTES: u64 = 512 * 1024;

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Inisialisasi lokasi log. Panggil sekali di awal `main`.
pub fn init() {
    let dir = crate::settings::config_dir();
    let path = dir.join("app.log");
    rotate_if_needed(&path);
    let _ = LOG_FILE.set(path);
}

pub fn info(msg: &str) {
    write_line("INFO", msg);
    #[cfg(debug_assertions)]
    println!("{msg}");
}

pub fn error(msg: &str) {
    write_line("ERROR", msg);
    #[cfg(debug_assertions)]
    eprintln!("{msg}");
}

fn write_line(level: &str, msg: &str) {
    let Some(path) = LOG_FILE.get() else {
        return; // logger belum di-init — abaikan.
    };
    let _guard = WRITE_LOCK.lock().unwrap();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{ts}] [{level}] {msg}");
    }
}

fn rotate_if_needed(path: &PathBuf) {
    let oversized = std::fs::metadata(path).is_ok_and(|m| m.len() > MAX_LOG_BYTES);
    if !oversized {
        return;
    }
    let old = path.with_extension("log.old");
    let _ = std::fs::remove_file(&old);
    let _ = std::fs::rename(path, &old);
}
