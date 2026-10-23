// Bangun assets/quran.json (teks Arab mushaf Kemenag + terjemahan resmi
// Kemenag) dari sumber publik. Jalankan sekali; hasilnya di-commit:
//
//   bun tools/quran-build.mjs
//
// Sumber:
// - risan/quran-json data/kemenag/quran.json  → arab + terjemahan Kemenag
// - risan/quran-json data/tanzil/chapters.json → jumlah ayat + meccan/medinan
// - NAMA_LATIN di bawah → nama surah gaya Indonesia (Kemenag)

const BASE = "https://cdn.jsdelivr.net/gh/risan/quran-json@main/data";
const KEMENAG = `${BASE}/kemenag/quran.json`;
const CHAPTERS = `${BASE}/tanzil/chapters.json`;

// Nama Latin gaya Indonesia, indeks 0 = surah 1. Urutan wajib sama dengan
// penomoran mushaf standar.
const NAMA_LATIN = [
  "Al-Fatihah", "Al-Baqarah", "Ali 'Imran", "An-Nisa'", "Al-Ma'idah",
  "Al-An'am", "Al-A'raf", "Al-Anfal", "At-Taubah", "Yunus",
  "Hud", "Yusuf", "Ar-Ra'd", "Ibrahim", "Al-Hijr",
  "An-Nahl", "Al-Isra'", "Al-Kahf", "Maryam", "Taha",
  "Al-Anbiya'", "Al-Hajj", "Al-Mu'minun", "An-Nur", "Al-Furqan",
  "Asy-Syu'ara'", "An-Naml", "Al-Qasas", "Al-'Ankabut", "Ar-Rum",
  "Luqman", "As-Sajdah", "Al-Ahzab", "Saba'", "Fatir",
  "Yasin", "As-Saffat", "Sad", "Az-Zumar", "Ghafir",
  "Fussilat", "Asy-Syura", "Az-Zukhruf", "Ad-Dukhan", "Al-Jasiyah",
  "Al-Ahqaf", "Muhammad", "Al-Fath", "Al-Hujurat", "Qaf",
  "Az-Zariyat", "At-Tur", "An-Najm", "Al-Qamar", "Ar-Rahman",
  "Al-Waqi'ah", "Al-Hadid", "Al-Mujadalah", "Al-Hasyr", "Al-Mumtahanah",
  "As-Saff", "Al-Jumu'ah", "Al-Munafiqun", "At-Tagabun", "At-Talaq",
  "At-Tahrim", "Al-Mulk", "Al-Qalam", "Al-Haqqah", "Al-Ma'arij",
  "Nuh", "Al-Jinn", "Al-Muzzammil", "Al-Muddassir", "Al-Qiyamah",
  "Al-Insan", "Al-Mursalat", "An-Naba'", "An-Nazi'at", "'Abasa", "At-Takwir",
  "Al-Infithar", "Al-Muthaffifin", "Al-Insyiqaq", "Al-Buruj", "At-Tariq",
  "Al-A'la", "Al-Gasyiyah", "Al-Fajr", "Al-Balad", "Asy-Syams",
  "Al-Lail", "Ad-Duha", "Al-Insyirah", "At-Tin", "Al-'Alaq",
  "Al-Qadar", "Al-Bayyinah", "Az-Zalzalah", "Al-'Adiyat", "Al-Qari'ah",
  "At-Takatsur", "Al-'Asr", "Al-Humazah", "Al-Fil", "Quraisy",
  "Al-Ma'un", "Al-Kautsar", "Al-Kafirun", "An-Nasr", "Al-Masad",
  "Al-Ikhlas", "Al-Falaq", "An-Nas",
];

const lisensi =
  "Teks Arab & terjemahan: Mushaf dan Terjemahan Al-Qur'an Kementerian Agama RI " +
  "(via github.com/risan/quran-json). Metadata surah: Tanzil.net (CC BY 3.0).";

function galat(pesan) {
  console.error(`GAGAL: ${pesan}`);
  process.exit(1);
}

async function unduh(url, coba = 3) {
  for (let i = 1; i <= coba; i++) {
    try {
      const res = await fetch(url);
      if (!res.ok) galat(`${url} → HTTP ${res.status}`);
      return res.json();
    } catch (e) {
      if (i === coba) galat(`${url} → ${e}`);
      console.error(`  retry ${i}/${coba - 1}: ${e.cause?.code || e.message}`);
      await new Promise((r) => setTimeout(r, 2000));
    }
  }
}

// Bersihkan teks agar JSON-nya bebas escape (syarat zero-copy borrow di
// src/quran.rs): petik lurus → petik tipografis, baris baru → spasi.
function bersih(s) {
  return s.replace(/[\r\n\t]+/g, " ").replace(/"/g, "”").trim();
}

const [kemenag, chapters] = await Promise.all([unduh(KEMENAG), unduh(CHAPTERS)]);

if (NAMA_LATIN.length !== 114) galat(`nama latin ${NAMA_LATIN.length}, harus 114`);
if (chapters.chapters.length !== 114) galat(`chapters ${chapters.chapters.length}, harus 114`);

let total = 0;
const surah = [];
for (let n = 1; n <= 114; n++) {
  const meta = chapters.chapters[n - 1];
  const ayat_mentah = kemenag[String(n)];
  if (!Array.isArray(ayat_mentah)) galat(`surah ${n} hilang di data kemenag`);
  if (ayat_mentah.length !== meta.total_verses) {
    galat(`surah ${n}: ${ayat_mentah.length} ayat, metadata ${meta.total_verses}`);
  }

  const ayat = ayat_mentah.map((a, i) => {
    if (a.verse !== i + 1) galat(`surah ${n} ayat ${i + 1}: nomor kacau (${a.verse})`);
    if (!a.text?.trim()) galat(`surah ${n}:${a.verse} teks Arab kosong`);
    if (!a.translation?.trim()) galat(`surah ${n}:${a.verse} terjemahan kosong`);
    const arab = bersih(a.text);
    const terjemahan = bersih(a.translation);
    if (arab.includes("\\") || terjemahan.includes("\\")) {
      galat(`surah ${n}:${a.verse} masih mengandung backslash`);
    }
    return [arab, terjemahan];
  });

  total += ayat.length;
  surah.push({
    nomor: n,
    nama: NAMA_LATIN[n - 1],
    arab: meta.name,
    mekah: meta.type === "meccan",
    ayat,
  });
}

if (total !== 6236) galat(`total ayat ${total}, harus 6236`);
// At-Taubah satu-satunya surah tanpa basmalah sebagai ayat.
if (surah[8].ayat[0][0].startsWith("بِسْمِ")) galat("At-Taubah seharusnya tanpa basmalah");

const keluar = {
  lisensi,
  surah,
};
const teks = JSON.stringify(keluar);
const tujuan = new URL("../assets/quran.json", import.meta.url);
await Bun.write(tujuan, teks);
console.log(
  `OK: ${surah.length} surah, ${total} ayat, ` +
    `${(teks.length / 1024 / 1024).toFixed(2)} MB → assets/quran.json`,
);
