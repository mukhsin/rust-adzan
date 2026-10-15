//! Embed resource Windows (ikon exe + metadata file) ke binary.
//! Ikon: assets/icon.ico (generate via `just icon`).

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Saat build dikendalikan `dx` (serve/bundle), dioxus CLI memasang
    // resource-nya sendiri (ikon + versioninfo dari Dioxus.toml). Memasang
    // dua VERSION resource bikin linker gagal: CVT1100 duplicate resource.
    // Deteksinya via OUT_DIR milik cargo: di bawah dx selalu mengandung
    // komponen `desktop-release`/`desktop-dev` (build standalone tidak).
    let under_dx = std::env::var("OUT_DIR")
        .is_ok_and(|d| d.contains("desktop-release") || d.contains("desktop-dev"));
    if under_dx {
        return;
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("FileDescription", "Jadwal Sholat");
    res.set("ProductName", "Jadwal Sholat");
    res.set("OriginalFilename", "rust-adzan.exe");
    res.compile()
        .expect("gagal embed resource windows (icon.ico ada?)");
}
