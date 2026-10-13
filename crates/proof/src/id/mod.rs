//! Indonesian proofing (spelling and grammar).
//!
//! - **Spelling** against installed Hunspell `id_ID` dictionary (or built-in vocabulary
//!   and morphological affix analysis for Indonesian prefixes me-, ber-, di-, ter-, ke-, pe-, se-
//!   and suffixes -kan, -an, -i, -ku, -mu, -nya).
//! - **Suggestions** ([`suggest()`]): standard KBBI corrections (e.g. `praktek` -> `praktik`,
//!   `apotik` -> `apotek`, `nasehat` -> `nasihat`) and dictionary candidates.
//! - **Grammar** ([`grammar`]): PUEBI punctuation, double spaces, repeated words, and preposition spacing.

pub mod grammar;
mod suggest;

use crate::affix::AffixDict;
use std::sync::{Arc, RwLock};

pub use suggest::suggest;

/// The Indonesian dictionary: `None` until first use.
static DICT: RwLock<Option<Arc<AffixDict>>> = RwLock::new(None);

/// File names of an Indonesian Hunspell dictionary.
const NAMES: &[&str] = &["id_ID", "id-ID", "id", "ind"];

/// Built-in common Indonesian root and frequent words.
const COMMON_ROOTS: &[&str] = &[
    "ada",
    "adalah",
    "air",
    "ajar",
    "akan",
    "akhir",
    "aku",
    "anak",
    "antara",
    "apa",
    "arti",
    "atas",
    "atau",
    "awal",
    "baca",
    "bagi",
    "bagus",
    "bahwa",
    "baik",
    "banyak",
    "baru",
    "bawah",
    "beberapa",
    "belajar",
    "belum",
    "benar",
    "bentuk",
    "besar",
    "biasa",
    "bisa",
    "buka",
    "buku",
    "bulan",
    "bukan",
    "butuh",
    "cara",
    "cepat",
    "cinta",
    "cipta",
    "coba",
    "contoh",
    "cukup",
    "dalam",
    "dapat",
    "dari",
    "dasar",
    "data",
    "daya",
    "dekat",
    "dengan",
    "depan",
    "desa",
    "dengar",
    "dia",
    "didik",
    "diri",
    "dukung",
    "dunia",
    "dua",
    "empat",
    "guna",
    "guru",
    "hanya",
    "hari",
    "harus",
    "hasil",
    "hati",
    "hidup",
    "hingga",
    "hubungan",
    "hukum",
    "hutan",
    "ia",
    "ibu",
    "ikut",
    "ingat",
    "ingin",
    "ini",
    "itu",
    "jadi",
    "jalan",
    "jangan",
    "janji",
    "jauh",
    "jelas",
    "jika",
    "jual",
    "juga",
    "jumlah",
    "juta",
    "kabar",
    "kami",
    "kamu",
    "karena",
    "kata",
    "keluar",
    "kembali",
    "kemudian",
    "kepala",
    "kerja",
    "ketika",
    "khusus",
    "kita",
    "kondisi",
    "kota",
    "kurang",
    "lain",
    "laku",
    "lama",
    "langsung",
    "lanjut",
    "lapor",
    "layak",
    "lebih",
    "lima",
    "lihat",
    "luas",
    "luar",
    "macam",
    "main",
    "makan",
    "mampu",
    "mana",
    "manfaat",
    "masalah",
    "masuk",
    "mata",
    "mau",
    "mawar",
    "melalui",
    "melihat",
    "memang",
    "membuat",
    "memiliki",
    "menjadi",
    "menurut",
    "mereka",
    "merupakan",
    "meski",
    "minta",
    "minum",
    "model",
    "mudah",
    "mungkin",
    "mulai",
    "naik",
    "nama",
    "negara",
    "nilai",
    "nomor",
    "nyata",
    "oleh",
    "orang",
    "pada",
    "pagi",
    "paham",
    "paling",
    "panjang",
    "pasti",
    "penting",
    "perlu",
    "pernah",
    "pertama",
    "pikir",
    "pilih",
    "posisi",
    "program",
    "proses",
    "puas",
    "pulang",
    "punya",
    "pusat",
    "putus",
    "rakyat",
    "rasa",
    "ribu",
    "ruang",
    "rumah",
    "saat",
    "sah",
    "saja",
    "salah",
    "sama",
    "sampai",
    "satu",
    "saya",
    "sebab",
    "sebagai",
    "sebelum",
    "sebuah",
    "secara",
    "sedang",
    "sehingga",
    "sejak",
    "sekarang",
    "sekitar",
    "selalu",
    "selama",
    "seluruh",
    "semua",
    "sendiri",
    "seperti",
    "sering",
    "serta",
    "sesuai",
    "setelah",
    "setiap",
    "siapa",
    "sudah",
    "sumber",
    "surat",
    "tahu",
    "tahun",
    "tak",
    "tambah",
    "tampak",
    "tangan",
    "tanya",
    "tata",
    "tepat",
    "terhadap",
    "terima",
    "terus",
    "tetap",
    "tetapi",
    "tiga",
    "tinggi",
    "titik",
    "tolong",
    "tulis",
    "tumbuh",
    "tujuan",
    "turun",
    "tutup",
    "uang",
    "ubah",
    "umum",
    "undang",
    "untuk",
    "upaya",
    "usaha",
    "utama",
    "wajib",
    "waktu",
    "walau",
    "wilayah",
    "wujud",
    "yaitu",
    "yang",
    "zaman",
];

pub fn dictionary() -> Arc<AffixDict> {
    if let Some(d) = DICT.read().unwrap_or_else(|e| e.into_inner()).as_ref() {
        return d.clone();
    }
    let loaded = Arc::new(load_installed());
    let mut slot = DICT.write().unwrap_or_else(|e| e.into_inner());
    slot.get_or_insert(loaded).clone()
}

fn load_installed() -> AffixDict {
    let Some((aff, dic)) = crate::hunspell::find(NAMES) else {
        return AffixDict::default();
    };
    crate::hunspell::load(&aff, &dic).unwrap_or_else(|_| AffixDict::default())
}

/// Is `word` a correctly spelled Indonesian word?
pub fn is_correct(word: &str) -> bool {
    let lower = word.to_lowercase();
    if lower.is_empty() {
        return true;
    }

    // 1. Check user dictionary
    if crate::in_user_dictionary(&lower) {
        return true;
    }

    // 2. Check installed Hunspell dictionary if loaded
    let dict = dictionary();
    if !dict.is_empty() && dict.check(&lower) {
        return true;
    }

    // 3. Reduplication check (e.g. buku-buku, anak-anak)
    if lower.split_once('-').is_some_and(|(l, r)| l == r && is_root_or_affixed(l)) {
        return true;
    }

    // 4. Built-in morphological check
    is_root_or_affixed(&lower)
}

fn is_root_or_affixed(w: &str) -> bool {
    if COMMON_ROOTS.contains(&w) {
        return true;
    }

    let mut candidates = vec![w];
    for suffix in &["-lah", "-kah", "-pun", "lah", "kah", "pun", "ku", "mu", "nya", "kan", "an", "i"] {
        if let Some(stem) = w.strip_suffix(suffix).filter(|s| s.len() > 2) {
            candidates.push(stem);
        }
    }

    for stem in &candidates {
        if COMMON_ROOTS.contains(stem) {
            return true;
        }

        for prefix in
            &["meng", "meny", "men", "mem", "me", "ber", "bel", "be", "di", "ter", "te", "ke", "peng", "peny", "pen", "pem", "pe", "per", "pel", "se"]
        {
            if stem.len() > prefix.len() + 2 && stem.starts_with(prefix) {
                let remainder = stem.get(prefix.len()..).unwrap_or("");
                if COMMON_ROOTS.contains(&remainder) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indonesian_spelling() {
        assert!(is_correct("buku"));
        assert!(is_correct("buku-buku"));
        assert!(is_correct("membaca"));
        assert!(is_correct("ditulis"));
        assert!(is_correct("belajar"));
        assert!(is_correct("rumah"));
        assert!(is_correct("rumahnya"));
        assert!(is_correct("pendidikan"));
    }
}
