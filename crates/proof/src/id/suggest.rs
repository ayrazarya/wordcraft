//! Spelling suggestions for Indonesian words.
//!
//! Evaluates common non-standard Indonesian spellings (e.g. `praktek` -> `praktik`, `apotik` -> `apotek`,
//! `nasehat` -> `nasihat`, `jadual` -> `jadwal`, `analisa` -> `analisis`) and generates candidates
//! via edit distance.

/// Known non-standard spellings to standard Indonesian (KBBI/PUEBI) mapping.
const STANDARD_REPLACEMENTS: &[(&str, &str)] = &[
    ("praktek", "praktik"),
    ("apotik", "apotek"),
    ("nasehat", "nasihat"),
    ("jadual", "jadwal"),
    ("analisa", "analisis"),
    ("antri", "antre"),
    ("azas", "asas"),
    ("atlit", "atlet"),
    ("aktifitas", "aktivitas"),
    ("efektifitas", "efektivitas"),
    ("kreatifitas", "kreativitas"),
    ("produktifitas", "produktivitas"),
    ("cidera", "cedera"),
    ("diagnosa", "diagnosis"),
    ("gladi", "geladi"),
    ("hakekat", "hakikat"),
    ("ijin", "izin"),
    ("jaman", "zaman"),
    ("kaedah", "kaidah"),
    ("kanker", "kanker"),
    ("karir", "karier"),
    ("kongkrit", "konkret"),
    ("kuitansi", "kuitansi"),
    ("lobang", "lubang"),
    ("manajemen", "manajemen"),
    ("milyar", "miliar"),
    ("nomer", "nomor"),
    ("obyek", "objek"),
    ("subyek", "subjek"),
    ("proyek", "proyek"),
    ("pasca", "pasca"),
    ("resiko", "risiko"),
    ("sekedar", "sekadar"),
    ("silahkan", "silakan"),
    ("standarisasi", "standarisasi"),
    ("taulan", "tolan"),
    ("tehnik", "teknik"),
    ("trampil", "terampil"),
    ("ustad", "ustaz"),
    ("propinsi", "provinsi"),
    ("hirarki", "hierarki"),
    ("kolektifitas", "kolektivitas"),
    ("konsekwen", "konsekuen"),
    ("kualitas", "kualitas"),
    ("kwalitas", "kualitas"),
];

pub fn suggest(word: &str, max: usize) -> Vec<String> {
    if word.is_empty() || max == 0 {
        return Vec::new();
    }
    let lower = word.to_lowercase();
    let mut out = Vec::new();

    // 1. Check direct known non-standard pairs
    for (wrong, right) in STANDARD_REPLACEMENTS {
        if lower == *wrong {
            let res = match_casing(word, right);
            if !out.contains(&res) {
                out.push(res);
            }
            if out.len() >= max {
                return out;
            }
        }
    }

    // 2. Single-character transposition (swapped adjacent characters)
    if out.len() < max && lower.len() >= 3 {
        let chars: Vec<char> = lower.chars().collect();
        for i in 0..chars.len().saturating_sub(1) {
            let mut swapped = chars.clone();
            swapped.swap(i, i + 1);
            let cand: String = swapped.into_iter().collect();
            if cand != lower && super::is_correct(&cand) {
                let res = match_casing(word, &cand);
                if !out.contains(&res) {
                    out.push(res);
                }
                if out.len() >= max {
                    return out;
                }
            }
        }
    }

    // 3. Single-character deletion
    if out.len() < max && lower.len() > 3 {
        for (i, c) in lower.char_indices() {
            let mut cand = lower.clone();
            cand.drain(i..i + c.len_utf8());
            if cand != lower && super::is_correct(&cand) {
                let res = match_casing(word, &cand);
                if !out.contains(&res) {
                    out.push(res);
                }
                if out.len() >= max {
                    return out;
                }
            }
        }
    }

    out
}

fn match_casing(orig: &str, sug: &str) -> String {
    let mut chars = orig.chars();
    let first_upper = chars.next().is_some_and(|c| c.is_uppercase());
    let all_upper = first_upper && chars.all(|c| c.is_uppercase());

    if all_upper {
        sug.to_uppercase()
    } else if first_upper {
        let mut s = sug.to_string();
        if let Some(c) = s.chars().next() {
            s.replace_range(0..c.len_utf8(), &c.to_uppercase().to_string());
        }
        s
    } else {
        sug.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indonesian_suggestions() {
        let s = suggest("praktek", 5);
        assert!(s.contains(&"praktik".to_string()));

        let s = suggest("Apotik", 5);
        assert!(s.contains(&"Apotek".to_string()));

        let s = suggest("SILAHKAN", 5);
        assert!(s.contains(&"SILAKAN".to_string()));
    }
}
