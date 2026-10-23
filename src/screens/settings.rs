//! Layar Pengaturan — enam grup komponen tersusun dua kolom ala frame
//! "Pengaturan" design system pen. Satu grup = satu komponen di file ini;
//! perubahan nilai lewat [`apply_settings`] (persist + sebar + refresh
//! draft).

use crate::autostart;
use crate::cities::{CITIES, find_city};
use crate::components::ui::button::{BTN, BTN_AMBER, BTN_ICON};
use crate::components::ui::drop_search::DropSearch;
use crate::components::ui::form::INPUT;
use crate::components::ui::group::{Group, Row};
use crate::components::ui::icons;
use crate::components::ui::modal::Modal;
use crate::components::ui::segmented::Segmented;
use crate::components::ui::stepper::{OffsetStepper, Stepper};
use crate::components::ui::switch::Switch;
use crate::components::ui::tooltip::{PosisiTooltip, Tooltip};
use crate::i18n::tr;
use crate::settings::{CalcMethod, Language, MadhabChoice, QuranWaqf, Settings, Theme};
use crate::state;
use dioxus::prelude::*;
use std::path::PathBuf;

const ZONA_WAKTU: [(&str, &str); 3] = [
    ("Asia/Jakarta", "WIB — Jakarta (UTC+7)"),
    ("Asia/Makassar", "WITA — Makassar (UTC+8)"),
    ("Asia/Jayapura", "WIT — Jayapura (UTC+9)"),
];

/// Jenis dialog modal yang sedang tampil.
#[derive(Debug, Clone, PartialEq)]
enum Dialog {
    None,
    ResetConfirm,
    AutostartError(String),
}

#[component]
pub fn SettingsView() -> Element {
    // Draft lokal; setiap perubahan langsung disimpan + disebarkan.
    let mut draft = use_signal(|| (*state::settings()).clone());
    let mut dialog = use_signal(|| Dialog::None);
    let lang = state::settings().language;
    // Status pemutaran per slot (di-poll berkala agar ikon play/stop akurat);
    // sinyalnya dipakai komponen AudioGroup.
    let mut playing_umum = use_signal(|| crate::audio::is_playing(crate::audio::Slot::Umum));
    let mut playing_subuh = use_signal(|| crate::audio::is_playing(crate::audio::Slot::Subuh));
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let umum = crate::audio::is_playing(crate::audio::Slot::Umum);
            if *playing_umum.read() != umum {
                playing_umum.set(umum);
            }
            let subuh = crate::audio::is_playing(crate::audio::Slot::Subuh);
            if *playing_subuh.read() != subuh {
                playing_subuh.set(subuh);
            }
        }
    });

    rsx! {
        div { class: "flex-1 px-4 pt-2 pb-6 flex flex-col gap-[18px] overflow-y-scroll overflow-x-hidden",

            // Grup pengaturan dua kolom ala frame "Pengaturan" di design
            // system: kiri LOKASI + AUDIO, kanan PERHITUNGAN + TAMPILAN +
            // NOTIFIKASI + PEMBARUAN + LAINNYA. items-start = kartu tak
            // dipaksa sama tinggi, persis perilaku Baris Grup di pen.
            div { class: "flex items-start gap-[18px]",
                div { class: "flex-1 flex flex-col gap-[18px]",
                    LocationGroup { draft }
                    CalculationGroup { draft }
                    QuranGroup { draft }
                    AppearanceGroup { draft }
                }
                div { class: "flex-1 flex flex-col gap-[18px]",
                    AudioGroup { draft, playing_umum, playing_subuh }
                    NotificationGroup { draft }
                    PuasaGroup { draft }
                    UpdateGroup { draft }
                    MiscGroup { draft,
                        on_autostart_error: move |err: String| dialog.set(Dialog::AutostartError(err)),
                        on_reset: move |_| dialog.set(Dialog::ResetConfirm),
                    }
                }
            }

            {match dialog.read().clone() {
                Dialog::None => rsx! {},
                Dialog::ResetConfirm => rsx! {
                    Modal {
                        title: tr(lang, "reset_btn"),
                        message: tr(lang, "modal_reset_msg"),
                        confirm_text: tr(lang, "confirm_reset"),
                        danger: true,
                        show_cancel: true,
                        on_confirm: move |_| {
                            let _ = autostart::set_enabled(false);
                            let s = Settings::default();
                            state::update_settings(s.clone());
                            draft.set(s);
                            dialog.set(Dialog::None);
                        },
                        on_cancel: move |_| dialog.set(Dialog::None),
                    }
                },
                Dialog::AutostartError(err) => rsx! {
                    Modal {
                        title: tr(lang, "autostart_title"),
                        message: err.clone(),
                        confirm_text: "OK",
                        danger: false,
                        show_cancel: false,
                        on_confirm: move |_| dialog.set(Dialog::None),
                        on_cancel: move |_| dialog.set(Dialog::None),
                    }
                },
            }}
        }
    }
}

/// Simpan perubahan pengaturan dari form: persist ke disk, sebar ke semua
/// dom (widget, tray), lalu refresh draft agar tampilan mengikuti.
fn apply_settings(mut draft: Signal<Settings>, s: Settings) {
    crate::settings::save(&s);
    state::update_settings(s.clone());
    draft.set(s);
}

/// Grup LOKASI: kota, koordinat manual, zona waktu.
#[component]
fn LocationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_location"),
            Row { label: tr(lang, "f_city"), tooltip: Some(tr(lang, "hint_kota_tz").to_string()),
                DropSearch {
                    value: draft
                        .read()
                        .city
                        .clone()
                        .unwrap_or_else(|| tr(lang, "city_manual").to_string()),
                    active: draft.read().city.clone().unwrap_or_default(),
                    options: CITIES
                        .iter()
                        .map(|c| (c.name.to_string(), c.name.to_string()))
                        .collect(),
                    searchable: true,
                    on_select: move |nama: String| {
                        if let Some(city) = find_city(&nama) {
                            let mut s = draft.read().clone();
                            s.city = Some(city.name.to_string());
                            s.lat = city.lat;
                            s.lng = city.lng;
                            s.timezone = city.timezone.name().to_string();
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_lat"),
                input {
                    class: "{INPUT}",
                    r#type: "text",
                    value: "{draft.read().lat:.4}",
                    onchange: move |e| {
                        if let Ok(lat) = e.value().trim().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.lat = lat;
                            s.city = None;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_lng"),
                input {
                    class: "{INPUT}",
                    r#type: "text",
                    value: "{draft.read().lng:.4}",
                    onchange: move |e| {
                        if let Ok(lng) = e.value().trim().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.lng = lng;
                            s.city = None;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_tz"),
                DropSearch {
                    // Trigger menampilkan label ramah yang sama dengan item
                    // daftarnya — bukan nama mentah "Asia/Jakarta".
                    value: label_zona_waktu(&draft.read().timezone),
                    active: draft.read().timezone.clone(),
                    options: ZONA_WAKTU
                        .iter()
                        .map(|(name, label)| (name.to_string(), label.to_string()))
                        .collect(),
                    searchable: false,
                    on_select: move |tz: String| {
                        let mut s = draft.read().clone();
                        s.set_timezone_name(&tz);
                        s.city = None;
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Satu baris koreksi menit: sholat wajib (map `prayer_offsets`) atau
/// Terbit (field `terbit_offset` sendiri, bukan sholat adzan).
#[derive(Debug, Clone, Copy)]
enum EntriKoreksi {
    Sholat(crate::prayer::PrayerKind),
    Terbit,
}

/// Grup PERHITUNGAN: metode perhitungan, mazhab ashar, dan koreksi waktu
/// per sholat (±30 menit — selaraskan jadwal dengan masjid setempat).
#[component]
fn CalculationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    // Jadwal TANPA koreksi — sumber hint "waktu bawaan" di baris yang
    // offset-nya ≠ 0. Dihitung sekali per render grup.
    let mut dasar = (*state::settings()).clone();
    dasar.prayer_offsets.clear();
    let dasar_times =
        crate::prayer::day_times(&dasar, crate::prayer::now_in_tz(&dasar).date_naive());

    // Baris koreksi, urut kronologis: Subuh → Terbit → Dzuhur → … → Isya.
    const BARIS_KOREKSI: [(&str, EntriKoreksi); 6] = [
        (
            "subuh",
            EntriKoreksi::Sholat(crate::prayer::PrayerKind::Subuh),
        ),
        ("terbit", EntriKoreksi::Terbit),
        (
            "dzuhur",
            EntriKoreksi::Sholat(crate::prayer::PrayerKind::Dzuhur),
        ),
        (
            "ashar",
            EntriKoreksi::Sholat(crate::prayer::PrayerKind::Ashar),
        ),
        (
            "maghrib",
            EntriKoreksi::Sholat(crate::prayer::PrayerKind::Maghrib),
        ),
        (
            "isya",
            EntriKoreksi::Sholat(crate::prayer::PrayerKind::Isya),
        ),
    ];

    rsx! {
        Group { label: tr(lang, "sec_calc"),
            Row { label: tr(lang, "f_method"),
                DropSearch {
                    value: tr(lang, method_label(draft.read().method)).to_string(),
                    active: method_name(draft.read().method).to_string(),
                    options: METODE
                        .iter()
                        .map(|(kunci, kunci_i18n)| {
                            ((*kunci).to_string(), tr(lang, kunci_i18n).to_string())
                        })
                        .collect(),
                    searchable: false,
                    on_select: move |kunci: String| {
                        if let Some(m) = method_from_name(&kunci) {
                            let mut s = draft.read().clone();
                            s.method = m;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_madhab"),
                DropSearch {
                    value: if draft.read().madhab == MadhabChoice::Hanafi {
                        tr(lang, "madhab_hanafi").to_string()
                    } else {
                        tr(lang, "madhab_shafi").to_string()
                    },
                    active: if draft.read().madhab == MadhabChoice::Hanafi {
                        "hanafi".to_string()
                    } else {
                        "shafi".to_string()
                    },
                    options: vec![
                        ("shafi".to_string(), tr(lang, "madhab_shafi").to_string()),
                        (
                            "hanafi".to_string(),
                            tr(lang, "madhab_hanafi").to_string(),
                        ),
                    ],
                    searchable: false,
                    on_select: move |kunci: String| {
                        let mut s = draft.read().clone();
                        s.madhab = if kunci == "hanafi" {
                            MadhabChoice::Hanafi
                        } else {
                            MadhabChoice::Shafi
                        };
                        apply_settings(draft, s);
                    },
                }
            }
            for (key, baris) in BARIS_KOREKSI {
                {
                    // Offset & titik nol tergantung jenis baris — Terbit di
                    // luar map offsets karena bukan sholat wajib (Dhuha
                    // mengikuti Terbit).
                    let (offset, waktu_dasar) = match baris {
                        EntriKoreksi::Sholat(kind) => {
                            (draft.read().offset_of(kind), dasar_times.map(|t| t.waktu(kind)))
                        }
                        EntriKoreksi::Terbit => {
                            (draft.read().offset_terbit(), dasar_times.map(|t| t.terbit))
                        }
                    };
                    // Hint hanya saat dikoreksi: tampilkan jadwal bawaan agar
                    // user tahu titik nolnya.
                    let hint = if offset != 0 {
                        waktu_dasar.map(|waktu| {
                            tr(lang, "offset_hint").replace(
                                "{waktu}",
                                &crate::prayer::format_time(waktu, draft.read().time_format_24),
                            )
                        })
                    } else {
                        None
                    };
                    rsx! {
                        Row { label: tr(lang, key), hint: hint,
                            OffsetStepper {
                                value: offset,
                                on_decrease: move |_| {
                                    let mut s = draft.read().clone();
                                    match baris {
                                        EntriKoreksi::Sholat(kind) => {
                                            let next = (s.offset_of(kind) - 1).max(-30);
                                            // 0 disimpan sebagai tanpa-entri agar config tetap ramping.
                                            if next == 0 { s.prayer_offsets.remove(&kind); }
                                            else { s.prayer_offsets.insert(kind, next); }
                                        }
                                        EntriKoreksi::Terbit => {
                                            s.terbit_offset = (s.offset_terbit() - 1).max(-30);
                                        }
                                    }
                                    apply_settings(draft, s);
                                },
                                on_increase: move |_| {
                                    let mut s = draft.read().clone();
                                    match baris {
                                        EntriKoreksi::Sholat(kind) => {
                                            let next = (s.offset_of(kind) + 1).min(30);
                                            if next == 0 { s.prayer_offsets.remove(&kind); }
                                            else { s.prayer_offsets.insert(kind, next); }
                                        }
                                        EntriKoreksi::Terbit => {
                                            s.terbit_offset = (s.offset_terbit() + 1).min(30);
                                        }
                                    }
                                    apply_settings(draft, s);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grup AUDIO ADZAN: sakelar aktif, volume, file adzan umum & khusus Subuh
/// (pilih file + tes putar/stop per slot).
#[component]
fn AudioGroup(
    draft: Signal<Settings>,
    playing_umum: Signal<bool>,
    playing_subuh: Signal<bool>,
) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_audio"),
            Row { label: tr(lang, "f_audio_enabled"),
                Switch {
                    checked: draft.read().audio_enabled,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.audio_enabled = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_volume"), hint: "{(draft.read().volume * 100.0) as i32}%",
                input {
                    class: "w-32 accent-ink cursor-pointer",
                    r#type: "range",
                    min: "0", max: "100",
                    value: "{(draft.read().volume * 100.0) as i32}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.volume = (v / 100.0) as f32;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_adzan_file"), hint: deskripsi_sumber(&draft.read(), false),
                div { class: "flex items-center gap-2",
                    Tooltip { label: tr(lang, "pick_audio_title").to_string(), posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio(lang) {
                                    let mut s = draft.read().clone();
                                    s.adzan_file = Some(p);
                                    apply_settings(draft, s);
                                }
                            },
                            "{tr(lang, \"pick\")}"
                        }
                    }
                    Tooltip { label: if *playing_umum.read() { tr(lang, "stop_play").to_string() } else { tr(lang, "test_play").to_string() }, posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN_ICON}",
                            onclick: move |_| {
                                if *playing_umum.read() {
                                    crate::audio::stop_slot(crate::audio::Slot::Umum);
                                } else {
                                    let s = state::settings();
                                    let _ = crate::audio::play_adzan(&s, false);
                                }
                            },
                            if *playing_umum.read() {
                                icons::IconSquare { class: "w-[13px] h-[13px]" }
                            } else {
                                icons::IconPlay { class: "w-[13px] h-[13px]" }
                            }
                        }
                    }
                }
            }
            Row { label: tr(lang, "f_subuh_file"), hint: deskripsi_sumber(&draft.read(), true),
                div { class: "flex items-center gap-2",
                    Tooltip { label: tr(lang, "pick_audio_title").to_string(), posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio(lang) {
                                    let mut s = draft.read().clone();
                                    s.subuh_file = Some(p);
                                    apply_settings(draft, s);
                                }
                            },
                            "{tr(lang, \"pick\")}"
                        }
                    }
                    Tooltip { label: if *playing_subuh.read() { tr(lang, "stop_play").to_string() } else { tr(lang, "test_play_subuh").to_string() }, posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN_ICON}",
                            onclick: move |_| {
                                if *playing_subuh.read() {
                                    crate::audio::stop_slot(crate::audio::Slot::Subuh);
                                } else {
                                    let s = state::settings();
                                    let _ = crate::audio::play_adzan(&s, true);
                                }
                            },
                            if *playing_subuh.read() {
                                icons::IconSquare { class: "w-[13px] h-[13px]" }
                            } else {
                                icons::IconPlay { class: "w-[13px] h-[13px]" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grup NOTIFIKASI: sakelar toast + pengingat X menit sebelum adzan.
#[component]
fn NotificationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_notif"),
            Row { label: tr(lang, "f_notif_enabled"), hint: tr(lang, "hint_notif_toast"),
                Switch {
                    checked: draft.read().notif_enabled,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.notif_enabled = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_pre_notif"), hint: tr(lang, "hint_pre_notif"),
                Stepper {
                    value: draft.read().pre_notif_minutes,
                    on_decrease: move |_| {
                        let mut s = draft.read().clone();
                        if s.pre_notif_minutes > 1 {
                            s.pre_notif_minutes -= 1;
                        }
                        apply_settings(draft, s);
                    },
                    on_increase: move |_| {
                        let mut s = draft.read().clone();
                        s.pre_notif_minutes = (s.pre_notif_minutes + 1).min(60);
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_imsak_notif"), hint: tr(lang, "hint_imsak_notif"),
                Switch {
                    checked: draft.read().imsak_notif,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.imsak_notif = on;
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup AL-QUR'AN: terjemahan on/off, gaya baca (per ayat / mushaf), dan
/// tampilan tanda waqf. Pindah ke mushaf mematikan switch terjemahan
/// sekalian (mushaf selalu Arab murni — tidak ada kondisi disabled tapi
/// masih on); kembali ke per ayat, user bisa menyalakannya lagi.
#[component]
fn QuranGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;
    let mushaf = draft.read().quran_mushaf;

    rsx! {
        Group { label: tr(lang, "sec_quran"),
            Row {
                label: tr(lang, "f_quran_terjemahan"),
                hint: if mushaf {
                    tr(lang, "hint_terjemahan_mushaf").to_string()
                } else {
                    tr(lang, "hint_quran_terjemahan").to_string()
                },
                div { class: if mushaf { "opacity-40 pointer-events-none" } else { "" },
                    Switch {
                        checked: draft.read().quran_terjemahan,
                        on_change: move |on| {
                            let mut s = draft.read().clone();
                            s.quran_terjemahan = on;
                            apply_settings(draft, s);
                        },
                    }
                }
            }
            Row { label: tr(lang, "f_quran_tampilan"),
                Segmented {
                    options: vec![
                        ("ayat".to_string(), tr(lang, "quran_mode_ayat").to_string()),
                        (
                            "mushaf".to_string(),
                            tr(lang, "quran_mode_mushaf").to_string(),
                        ),
                    ],
                    active: if draft.read().quran_mushaf {
                        "mushaf".to_string()
                    } else {
                        "ayat".to_string()
                    },
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        let mushaf = v == "mushaf";
                        s.quran_mushaf = mushaf;
                        // Mode mushaf selalu tanpa terjemahan — switch
                        // terjemahan langsung dimatikan sekalian supaya
                        // tidak ambigu (disabled tapi masih on).
                        if mushaf {
                            s.quran_terjemahan = false;
                        }
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_quran_waqf"), hint: tr(lang, "hint_quran_waqf"),
                Segmented {
                    options: vec![
                        (
                            "tampil".to_string(),
                            tr(lang, "quran_waqf_tampil").to_string(),
                        ),
                        ("redup".to_string(), tr(lang, "quran_waqf_redup").to_string()),
                        (
                            "sembunyi".to_string(),
                            tr(lang, "quran_waqf_sembunyi").to_string(),
                        ),
                    ],
                    active: match draft.read().quran_waqf {
                        QuranWaqf::Tampil => "tampil",
                        QuranWaqf::Redup => "redup",
                        QuranWaqf::Sembunyi => "sembunyi",
                    }
                    .to_string(),
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.quran_waqf = match v.as_str() {
                            "redup" => QuranWaqf::Redup,
                            "sembunyi" => QuranWaqf::Sembunyi,
                            _ => QuranWaqf::Tampil,
                        };
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup PUASA SUNNAH: toast pengingat saat imsak + jenis puasa yang
/// diikutkan (menentukan pil widget/Harian juga).
#[component]
fn PuasaGroup(draft: Signal<Settings>) -> Element {
    use crate::puasa::PuasaSunnahKind;
    let lang = state::settings().language;

    // (jenis, kunci label, kunci hint opsional) — urut kanonik.
    const JENIS: [(PuasaSunnahKind, &str, Option<&str>); 5] = [
        (PuasaSunnahKind::Senin, "puasa_senin", None),
        (PuasaSunnahKind::Kamis, "puasa_kamis", None),
        (
            PuasaSunnahKind::AyyamulBidh,
            "puasa_ayyamul_bidh",
            Some("hint_ayyamul_bidh"),
        ),
        (
            PuasaSunnahKind::Syawal,
            "puasa_syawal",
            Some("hint_puasa_syawal"),
        ),
        (
            PuasaSunnahKind::HariPenting,
            "puasa_penting",
            Some("hint_puasa_penting"),
        ),
    ];

    rsx! {
        Group { label: tr(lang, "sec_puasa"),
            Row { label: tr(lang, "f_puasa_notif"), hint: tr(lang, "hint_puasa_notif"),
                Switch {
                    checked: draft.read().puasa_sunnah_notif,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.puasa_sunnah_notif = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_puasa_notif_malam"), hint: tr(lang, "hint_puasa_notif_malam"),
                Switch {
                    checked: draft.read().puasa_sunnah_notif_malam,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.puasa_sunnah_notif_malam = on;
                        apply_settings(draft, s);
                    },
                }
            }
            for (jenis, label, hint) in JENIS {
                {
                    let aktif = draft.read().puasa_sunnah.contains(&jenis);
                    let hint = hint.map(|h| tr(lang, h).to_string());
                    rsx! {
                        Row { label: tr(lang, label), hint: hint,
                            Switch {
                                checked: aktif,
                                on_change: move |on| {
                                    let mut s = draft.read().clone();
                                    if on {
                                        if !s.puasa_sunnah.contains(&jenis) {
                                            s.puasa_sunnah.push(jenis);
                                        }
                                    } else {
                                        s.puasa_sunnah.retain(|k| *k != jenis);
                                    }
                                    apply_settings(draft, s);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grup TAMPILAN: tema (ikut sistem / gelap / terang), bahasa UI, dan
/// format jam 24/12.
#[component]
fn AppearanceGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_appearance"),
            Row { label: tr(lang, "f_theme"),
                Segmented {
                    options: vec![
                        ("system".to_string(), tr(lang, "theme_system").to_string()),
                        ("dark".to_string(), tr(lang, "theme_dark").to_string()),
                        ("light".to_string(), tr(lang, "theme_light").to_string()),
                    ],
                    active: match draft.read().theme {
                        Theme::System => "system",
                        Theme::Dark => "dark",
                        Theme::Light => "light",
                    }
                    .to_string(),
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.theme = match v.as_str() {
                            "dark" => Theme::Dark,
                            "light" => Theme::Light,
                            _ => Theme::System,
                        };
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_language"),
                Segmented {
                    options: vec![
                        ("id".to_string(), "Indonesia".to_string()),
                        ("en".to_string(), "English".to_string()),
                    ],
                    active: match draft.read().language {
                        Language::Id => "id",
                        Language::En => "en",
                    }
                    .to_string(),
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.language = if v == "en" { Language::En } else { Language::Id };
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_time_format"),
                Segmented {
                    options: vec![
                        ("24".to_string(), tr(lang, "fmt_24").to_string()),
                        ("12".to_string(), tr(lang, "fmt_12").to_string()),
                    ],
                    active: if draft.read().time_format_24 {
                        "24".to_string()
                    } else {
                        "12".to_string()
                    },
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.time_format_24 = v == "24";
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup PEMBARUAN: mode cek pembaruan (manual/otomatis) + tombol cek
/// manual, dan baris kedua kondisional saat ada versi baru — tombol
/// "Perbarui & Instal" (installer silent dari share kantor).
#[component]
fn UpdateGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;
    // Status cek manual & pemasangan — tombol menampilkan spinner selama
    // HTTP/UNC scan atau copy+spawn installer berjalan di thread latar
    // (dipoll dari update.rs tiap 150 ms).
    let mut cek_berjalan = use_signal(|| false);
    let mut memasang = use_signal(|| false);
    let mut tersedia = use_signal(|| None::<String>);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let berjalan = crate::update::cek_manual_berjalan();
            if *cek_berjalan.read() != berjalan {
                cek_berjalan.set(berjalan);
            }
            let pasang = crate::update::pasang_berjalan();
            if *memasang.read() != pasang {
                memasang.set(pasang);
            }
            let tag = crate::update::versi_tersedia().map(|(r, _)| r.tag);
            if *tersedia.read() != tag {
                tersedia.set(tag);
            }
        }
    });

    rsx! {
        Group { label: tr(lang, "sec_update"),
            Row { label: tr(lang, "f_update_check"),
                div { class: "flex items-center gap-2",
                    // Tombol cek hanya relevan di mode Manual — saat Otomatis
                    // pemeriksaan berjalan sendiri tiap 1 jam.
                    if !draft.read().update_check {
                        button {
                            class: if *cek_berjalan.read() {
                                "{BTN} min-w-[118px] justify-center opacity-50 cursor-wait"
                            } else {
                                "{BTN} min-w-[118px] justify-center"
                            },
                            disabled: *cek_berjalan.read(),
                            onclick: move |_| {
                                // Re-entry di-guard di dalam check_now_manual.
                                crate::update::check_now_manual();
                            },
                            if *cek_berjalan.read() {
                                span { class: "w-[13px] h-[13px] rounded-full border-2 border-ink-3 border-t-transparent animate-spin inline-block" }
                                span { class: "text-ink-2", "{tr(lang, \"update_checking\")}" }
                            } else {
                                "{tr(lang, \"update_check_btn\")}"
                            }
                        }
                    }
                    Segmented {
                        options: vec![
                            (
                                "manual".to_string(),
                                tr(lang, "update_mode_manual").to_string(),
                            ),
                            (
                                "otomatis".to_string(),
                                tr(lang, "update_mode_otomatis").to_string(),
                            ),
                        ],
                        active: if draft.read().update_check {
                            "otomatis".to_string()
                        } else {
                            "manual".to_string()
                        },
                        on_select: move |v: String| {
                            let otomatis = v == "otomatis";
                            if draft.read().update_check == otomatis {
                                return;
                            }
                            let mut s = draft.read().clone();
                            s.update_check = otomatis;
                            apply_settings(draft, s);
                            if otomatis {
                                // Baru dinyalakan: cek langsung sekali, tidak
                                // menunggu siklus 1 jam.
                                crate::update::picu_cek_otomatis();
                            }
                        },
                    }
                }
            }
            // Baris kedua — hanya muncul saat ada versi baru dari cek
            // terakhir (manual maupun otomatis). Label dinamis (mengandung
            // nomor versi) sehingga tidak lewat komponen Row.
            if let Some(tag) = tersedia() {
                {
                    let label = tr(lang, "update_available_title").replace("{versi}", &tag);
                    let hint = if *memasang.read() {
                        tr(lang, "update_downloading_sub").to_string()
                    } else {
                        tr(lang, "update_available_sub").to_string()
                    };
                    rsx! {
                        div { class: "flex items-center justify-between gap-3 h-[58px] shrink-0 border-b border-hairline last:border-b-0",
                            div { class: "flex flex-col gap-0.5 min-w-0",
                                span { class: "text-[13.5px] font-semibold text-ink", "{label}" }
                                span { class: "text-[11.5px] text-ink-3 truncate", "{hint}" }
                            }
                            button {
                                class: "{BTN_AMBER} min-w-[160px] justify-center",
                                disabled: *memasang.read(),
                                onclick: move |_| {
                                    // Re-entry di-guard di dalam pasang_sekarang.
                                    crate::update::pasang_sekarang();
                                },
                                if *memasang.read() {
                                    span { class: "w-[13px] h-[13px] rounded-full border-2 border-amber-ink/40 border-t-transparent animate-spin inline-block" }
                                    "{tr(lang, \"update_installing\")}"
                                } else {
                                    "{tr(lang, \"update_install_btn\")}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Grup LAINNYA: auto-start saat Windows menyala + reset pengaturan.
#[component]
fn MiscGroup(
    draft: Signal<Settings>,
    on_autostart_error: EventHandler<String>,
    on_reset: EventHandler<()>,
) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_misc"),
            Row { label: tr(lang, "f_autostart"),
                Switch {
                    checked: draft.read().autostart,
                    on_change: move |on| match autostart::set_enabled(on) {
                        Ok(()) => {
                            let mut s = draft.read().clone();
                            s.autostart = on;
                            apply_settings(draft, s);
                        }
                        Err(err) => on_autostart_error.call(err),
                    },
                }
            }
            Row { label: tr(lang, "reset_btn"),
                button {
                    class: "px-3 py-1.5 rounded-lg text-[12.5px] font-semibold text-red-500 border border-red-400/60 hover:border-red-400 hover:bg-red-500/10 transition-colors cursor-pointer",
                    onclick: move |_| on_reset.call(()),
                    "{tr(lang, \"reset_action\")}"
                }
            }
        }
    }
}

fn method_name(m: CalcMethod) -> &'static str {
    match m {
        CalcMethod::Kemenag => "kemenag",
        CalcMethod::MuslimWorldLeague => "mwl",
        CalcMethod::Egyptian => "egyptian",
        CalcMethod::Karachi => "karachi",
        CalcMethod::UmmAlQura => "umm_al_qura",
        CalcMethod::NorthAmerica => "north_america",
    }
}

/// (kunci nilai, kunci terjemahan) metode perhitungan — sumber dropdown
/// custom di grup Perhitungan.
const METODE: [(&str, &str); 6] = [
    ("kemenag", "m_kemenag"),
    ("mwl", "m_mwl"),
    ("egyptian", "m_egyptian"),
    ("karachi", "m_karachi"),
    ("umm_al_qura", "m_umm_al_qura"),
    ("north_america", "m_isna"),
];

/// Kunci terjemahan label untuk satu metode.
fn method_label(m: CalcMethod) -> &'static str {
    let nama = method_name(m);
    METODE
        .iter()
        .find(|(kunci, _)| *kunci == nama)
        .map(|(_, kunci_i18n)| *kunci_i18n)
        .expect("metode selalu ada di tabel METODE")
}

fn method_from_name(s: &str) -> Option<CalcMethod> {
    match s {
        "kemenag" => Some(CalcMethod::Kemenag),
        "mwl" => Some(CalcMethod::MuslimWorldLeague),
        "egyptian" => Some(CalcMethod::Egyptian),
        "karachi" => Some(CalcMethod::Karachi),
        "umm_al_qura" => Some(CalcMethod::UmmAlQura),
        "north_america" => Some(CalcMethod::NorthAmerica),
        _ => None,
    }
}

/// Label ramah untuk dropdown Zona Waktu — sama dengan label item
/// daftarnya. Zona di luar daftar (mis. config diedit manual) jatuh ke
/// nama mentahnya.
fn label_zona_waktu(nama: &str) -> String {
    ZONA_WAKTU
        .iter()
        .find(|(zona, _)| *zona == nama)
        .map(|(_, label)| label.to_string())
        .unwrap_or_else(|| nama.to_string())
}

fn pick_audio(lang: Language) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a", "aac"])
        .set_title(tr(lang, "pick_audio_title"))
        .pick_file()
}

/// Deskripsi sumber audio aktif saat ini (kustom / file / bawaan ter-embed)
/// untuk ditampilkan di settings. `subuh` memilih baris mana yang dideskripsi.
fn deskripsi_sumber(s: &Settings, subuh: bool) -> String {
    let lang = s.language;
    let resolved = if subuh {
        crate::audio::resolve_subuh(s)
    } else {
        crate::audio::resolve_adzan(s)
    };
    match resolved {
        Some(p) => {
            // Cukup nama file — full path terlalu panjang untuk card yang sempit.
            let nama = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.display().to_string());
            let kustom = if subuh {
                s.subuh_file.as_ref() == Some(&p)
            } else {
                s.adzan_file.as_ref() == Some(&p)
            };
            if kustom {
                tr(lang, "src_custom").replace("{nama}", &nama)
            } else {
                tr(lang, "src_file").replace("{nama}", &nama)
            }
        }
        None => tr(lang, "src_embedded").to_string(),
    }
}
