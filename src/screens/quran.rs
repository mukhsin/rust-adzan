//! Layar Al-Qur'an — daftar 114 surah + pembaca, teks Arab mushaf Kemenag
//! dan terjemahan resmi Kemenag RI (100% offline dari `crate::quran`),
//! ala frame "Quran" design system pen.
//!
//! - mode daftar: kotak cari + papan baris surah (nomor, nama latin, meta
//!   jumlah ayat, badge Makkiyah/Madaniyah, nama Arab rata kanan);
//! - mode baca (master–detail): sidebar surah di kiri + kartu baca di
//!   kanan — kepala surah, basmalah, lalu ayat dengan gaya baca dari
//!   settings: "per ayat" (blok terpisah, terjemahan di bawah tiap ayat)
//!   atau "mushaf" (aliran Arab menerus, selalu tanpa terjemahan, jarak
//!   baris longgar). Tanda waqf dipisah dari teks (lihat
//!   `quran::segmen_teks`) sehingga bisa diredek/disembunyikan.

use crate::components::ui::icons;
use crate::i18n::tr;
use crate::quran::{self, Surah};
use crate::state;
use dioxus::prelude::*;

#[component]
pub fn QuranView() -> Element {
    let settings = state::settings();
    let lang = settings.language;
    let mut cari = use_signal(String::new);
    // None = papan daftar surah; Some(nomor) = mode baca surah tersebut.
    let mut aktif = use_signal(|| None::<u8>);

    let kueri = cari.read().to_lowercase();
    let daftar: Vec<&'static Surah<'static>> = quran::surah_list()
        .iter()
        .filter(|s| {
            kueri.is_empty()
                || s.nama.to_lowercase().contains(&kueri)
                || s.nomor.to_string() == kueri
        })
        .collect();
    let _ = settings;

    let konteks = match aktif() {
        Some(n) => quran::baca(n)
            .map(|s| s.nama.to_uppercase())
            .unwrap_or_default(),
        None => tr(lang, "quran_konteks").replace("{n}", "114"),
    };

    rsx! {
        div { class: "flex-1 p-4 flex flex-col gap-[14px] overflow-hidden",

            // Baris cari: kotak cari + konteks kanan (ala frame Quran pen).
            div { class: "flex items-center gap-4 px-1 py-1",
                div { class: "relative w-[280px]",
                    icons::IconSearch { class: "w-4 h-4 text-ink-3 absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" }
                    input {
                        class: "w-full bg-board border border-pass-line rounded-lg pl-9 pr-3 py-2 text-[13px] text-ink placeholder:text-ink-3 outline-none focus:border-ink-3 transition-colors",
                        placeholder: tr(lang, "quran_cari"),
                        value: cari,
                        oninput: move |e| cari.set(e.value()),
                    }
                }
                div { class: "flex-1" }
                span { class: "font-mono text-[10.5px] tracking-[0.08em] uppercase text-ink-3",
                    "{konteks}"
                }
            }

            match aktif() {
                None => rsx! {
                    // Papan daftar surah.
                    div { class: "flex-1 bg-board border border-pass-line rounded-xl flex flex-col overflow-hidden",
                        div { class: "flex-1 overflow-y-auto",
                            for s in &daftar {
                                BarisSurah { nomor: s.nomor, aktif: false, on_pilih: move |n| aktif.set(Some(n)) }
                            }
                            if daftar.is_empty() {
                                div { class: "flex-1 flex items-center justify-center py-10",
                                    span { class: "text-[13px] text-ink-3", "{tr(lang, \"quran_tidak_ada\")}" }
                                }
                            }
                        }
                    }
                },
                Some(nomor) => rsx! {
                    // Mode baca: sidebar surah + kartu baca.
                    div { class: "flex-1 flex items-stretch gap-[14px] overflow-hidden",
                        div { class: "w-[280px] bg-board border border-pass-line rounded-xl flex flex-col overflow-hidden",
                            div { class: "flex-1 overflow-y-auto",
                                for s in &daftar {
                                    BarisSurah { nomor: s.nomor, aktif: s.nomor == nomor, on_pilih: move |n| aktif.set(Some(n)) }
                                }
                            }
                        }
                        KartuBaca { nomor: nomor }
                    }
                },
            }
        }
    }
}

/// Satu baris surah (papan daftar & sidebar mode baca): nomor dalam kotak,
/// nama latin + meta, badge jenis, nama Arab rata kanan.
#[component]
fn BarisSurah(nomor: u8, aktif: bool, on_pilih: EventHandler<u8>) -> Element {
    let lang = state::settings().language;
    let Some(s) = quran::baca(nomor) else {
        return rsx! {};
    };
    let jenis = if s.mekah {
        tr(lang, "quran_makkiyah")
    } else {
        tr(lang, "quran_madaniyah")
    };
    let kotak = if aktif {
        "w-8 h-8 shrink-0 rounded-lg bg-amber text-amber-ink flex items-center justify-center font-mono text-[12px] font-bold"
    } else {
        "w-8 h-8 shrink-0 rounded-lg border border-pass-line text-ink-3 flex items-center justify-center font-mono text-[12px] font-semibold"
    };
    let baris_class = if aktif {
        "flex items-center gap-3 px-4 py-2.5 cursor-pointer bg-amber/10"
    } else {
        "flex items-center gap-3 px-4 py-2.5 cursor-pointer hover:bg-ink/5 transition-colors"
    };

    rsx! {
        button { class: "{baris_class} w-full text-left border-b border-hairline",
            onclick: move |_| on_pilih.call(nomor),
            span { class: "{kotak}", "{s.nomor}" }
            div { class: "flex flex-col gap-1 min-w-0 flex-1",
                span { class: "text-[13.5px] font-bold text-ink truncate", "{s.nama}" }
                {
                    let meta = tr(lang, "quran_ayat_count")
                        .replace("{n}", &s.ayat.len().to_string());
                    rsx! {
                        span { class: "text-[11px] text-ink-3", "{jenis} · {meta}" }
                    }
                }
            }
            span { class: "font-arab text-[19px] text-ink shrink-0", dir: "rtl", "{s.arab}" }
        }
    }
}

/// Kartu baca: kepala surah dan blok ayat. Gaya baca dari settings:
/// - per ayat (default): tiap ayat satu blok — Arab RTL + ornamen nomor,
///   terjemahan di bawahnya;
/// - mushaf: seluruh ayat mengalir menerus (aliran RTL) dengan jarak
///   baris longgar, selalu tanpa terjemahan; ornamen nomor menyatu di
///   aliran.
/// Pindah surah lewat sidebar.
#[component]
fn KartuBaca(nomor: u8) -> Element {
    let settings = state::settings();
    let lang = settings.language;
    let mushaf = settings.quran_mushaf;
    // Mode mushaf selalu Arab murni — setting terjemahan hanya berlaku di
    // mode per ayat (switch di Pengaturan ikut dinonaktifkan saat mushaf).
    let tampil_terjemahan = settings.quran_terjemahan && !mushaf;
    // Tanda waqf dirender sebagai inline-block ber-font-size sendiri — ini
    // memutus shaping run, jadi glyph tidak lagi "menggantung" di atas
    // huruf terakhir (perilaku mark-attachment Amiri yang bikin posisinya
    // beda-beda: nempel huruf = tinggi sekali, ada spasi = netral).
    // Semua jenis tanda jadi seragam: 16px, sejajar tengah, turun sedikit.
    // Jarak horizontalnya dihitung di TeksAyat (tergantung ada tidaknya
    // spasi di teks). None = disembunyikan sesuai setting.
    let kelas_waqf = match settings.quran_waqf {
        crate::settings::QuranWaqf::Tampil => {
            Some("inline-block text-[16px] align-middle relative top-[2px]")
        }
        crate::settings::QuranWaqf::Redup => {
            Some("inline-block text-[16px] align-middle relative top-[2px] text-ink-3")
        }
        crate::settings::QuranWaqf::Sembunyi => None,
    };
    let Some(s) = quran::baca(nomor) else {
        return rsx! {};
    };
    let jenis = if s.mekah {
        tr(lang, "quran_makkiyah")
    } else {
        tr(lang, "quran_madaniyah")
    };
    let kepala = tr(lang, "quran_surah_label")
        .replace("{n}", &s.nomor.to_string())
        .replace("{ayat}", &s.ayat.len().to_string())
        .replace("{jenis}", &jenis.to_uppercase());
    // Basmalah tampil sebagai baris sendiri — kecuali Al-Fatihah (ayat 1-nya
    // memang basmalah) dan At-Taubah (tanpa basmalah).
    let tampil_basmalah = s.nomor != 1 && s.nomor != 9;

    rsx! {
        div { class: "flex-1 bg-board border border-pass-line rounded-xl flex flex-col overflow-hidden",
            // Kepala surah: identitas. Pindah surah lewat sidebar di kiri.
            div { class: "flex items-center gap-4 px-5 py-3.5 border-b border-hairline",
                div { class: "flex flex-col gap-0.5 min-w-0",
                    span { class: "font-mono text-[9.5px] font-semibold tracking-[0.14em] uppercase text-ink-3",
                        "{kepala}"
                    }
                    div { class: "flex items-baseline gap-2.5",
                        span { class: "text-[17px] font-bold text-ink", "{s.nama}" }
                        span { class: "font-arab text-[17px] text-ink-2", dir: "rtl", "{s.arab}" }
                    }
                }
                div { class: "flex-1" }
            }

            div { class: "flex-1 overflow-y-auto",
                if tampil_basmalah {
                    div { class: "py-5 flex items-center justify-center border-b border-hairline",
                        span { class: "font-arab text-[22px] leading-relaxed text-ink", dir: "rtl",
                            "بِسْمِ ٱللَّهِ ٱلرَّحْمَـٰنِ ٱلرَّحِيمِ"
                        }
                    }
                }
                if mushaf {
                    // Gaya mushaf: satu paragraf aliran Arab menerus, jarak
                    // baris longgar biar nyaman dibaca; tanpa terjemahan.
                    div { class: "px-6 py-6",
                        p { class: "font-arab text-[23px] leading-[2.75] text-ink text-right",
                            dir: "rtl",
                            for (i, (arab, _arti)) in s.ayat.iter().enumerate() {
                                {
                                    let nomor_arab =
                                        crate::screens::calendar::arabic_numerals((i + 1) as u32);
                                    rsx! {
                                        TeksAyat { teks: arab, kelas_waqf: kelas_waqf }
                                        // Ornamen nomor menyatu di aliran —
                                        // jarak dua sisi agar ayat berikutnya
                                        // tak menempel.
                                        span { class: "mx-2 inline-flex w-8 h-8 shrink-0 items-center justify-center rounded-full border border-amber/50 bg-amber/10 align-middle font-arab text-[13px] text-amber",
                                            "{nomor_arab}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for (i, (arab, terjemahan)) in s.ayat.iter().enumerate() {
                        {
                            let nomor_arab = crate::screens::calendar::arabic_numerals((i + 1) as u32);
                            rsx! {
                                div { class: "px-5 py-4 border-b border-hairline last:border-b-0",
                                    // Penanda nomor mengalir langsung setelah kata
                                    // terakhir (gaya mushaf) — lingkaran ornamen
                                    // dengan jarak kecil dari teks (ms = sisi
                                    // kanan di RTL).
                                    p { class: "font-arab text-[23px] leading-[2] text-ink text-right",
                                        dir: "rtl",
                                        TeksAyat { teks: arab, kelas_waqf: kelas_waqf }
                                        span { class: "ms-2 inline-flex w-8 h-8 shrink-0 items-center justify-center rounded-full border border-amber/50 bg-amber/10 align-middle font-arab text-[13px] text-amber",
                                            "{nomor_arab}"
                                        }
                                    }
                                    if tampil_terjemahan {
                                        p { class: "mt-2 text-[13px] leading-relaxed text-ink-2",
                                            "{terjemahan}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Teks Arab satu ayat dengan tanda waqf yang sudah dipisah (`quran::
/// segmen_teks`): tanda bisa distyle (diturunkan + jarak, atau redup)
/// atau disembunyikan sesuai settings. Jarak di depan tanda kondisional:
/// tintanya menjorok balik ke arah kata sebelumnya, sehingga tanda yang
/// menempel langsung di huruf (tanpa spasi di teks, mis. akhir ayat 2)
/// butuh margin lebih besar agar setara dengan tanda ber-spasi.
#[component]
fn TeksAyat(teks: &'static str, kelas_waqf: Option<&'static str>) -> Element {
    let segmen = quran::segmen_teks(teks);
    rsx! {
        for (i, (potongan, tanda)) in segmen.iter().enumerate() {
            {
                if *tanda {
                    match kelas_waqf {
                        Some(dasar) => {
                            let nempel = i > 0
                                && !segmen[i - 1].0.ends_with(char::is_whitespace);
                            let jarak = if nempel { "ms-[10px]" } else { "ms-[4px]" };
                            rsx! { span { class: "{dasar} {jarak}", "{potongan}" } }
                        }
                        None => rsx! {},
                    }
                } else {
                    rsx! { span { "{potongan}" } }
                }
            }
        }
    }
}
