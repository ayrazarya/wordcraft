//! Indonesian grammar and punctuation checks: rule-based, conservative, and helpful.
//! Messages are in Indonesian.
//!
//! - **Spasi sebelum tanda baca** (`kata ,` -> `kata,`).
//! - **Spasi ganda** (`kata  kata` -> `kata kata`).
//! - **Huruf kapital di awal kalimat**, mengetahui singkatan umum Indonesia (`dll.`, `dsb.`, `dkk.`, `prof.`, `dr.`).
//! - **Kata berulang** (`buku buku` -> `buku-buku`), kecuali kata seru (`ha ha`, `wah wah`).
//! - **Preposisi "di / ke" tempat** (`dimana` -> `di mana`, `kemana` -> `ke mana`, `disana` -> `di sana`, `disini` -> `di sini`, `diatas` -> `di atas`, `dibawah` -> `di bawah`).
//! - **Tanda koma sebelum konjungsi pertentangan** (`tetapi`, `melainkan`, `sedangkan`, `padahal`).

use crate::{Issue, IssueKind, ProofLang, words};

/// Kata-kata yang sengaja diulang sebagai seruan / onomatope.
const ALLOWED_REPEATS: &[&str] = &["ha", "he", "hi", "ho", "ya", "wah", "aduh", "ayo", "dor", "tok", "ting", "tut", "huss", "ciye"];

/// Singkatan umum Bahasa Indonesia yang diikuti titik.
const ABBREVIATIONS: &[&str] = &[
    "dll", "dsb", "dst", "dkk", "tsb", "hal", "hlm", "prof", "dr", "drs", "dra", "ir", "sh", "se", "sked", "spd", "pt", "cv", "no", "jl", "jln",
    "rt", "rw", "kel", "kec", "kab", "prov", "yth", "sdr", "sdri", "alm", "bpk", "ibu", "kpd", "an", "ub", "up", "ttd", "a", "b", "c", "d", "e",
];

/// Konjungsi pertentangan yang menurut PUEBI / EYD baku didahului tanda koma.
const COMMA_CONJUNCTIONS: &[&str] = &["tetapi", "melainkan", "sedangkan", "padahal"];

pub fn check(text: &str) -> Vec<Issue> {
    let mut out = Vec::new();
    check_repeated_words(text, &mut out);
    check_spaces(text, &mut out);
    check_preposition_spacing(text, &mut out);
    check_comma_conjunctions(text, &mut out);
    check_capital_after_period(text, &mut out);
    out.sort_by_key(|i| i.start);
    out
}

fn check_repeated_words(text: &str, out: &mut Vec<Issue>) {
    let w = words(text);
    for window in w.windows(2) {
        let (a, b) = (window[0], window[1]);
        let sa = text.get(a.0..a.1).unwrap_or("");
        let sb = text.get(b.0..b.1).unwrap_or("");
        let gap = text.get(a.1..b.0).unwrap_or("");
        if gap.trim().is_empty() && sa.eq_ignore_ascii_case(sb) {
            let lower = sa.to_lowercase();
            if ALLOWED_REPEATS.contains(&lower.as_str()) {
                continue;
            }
            out.push(Issue {
                start: a.0,
                end: b.1,
                kind: IssueKind::Grammar,
                message: format!("Kata berulang; jika kata jamak gunakan tanda hubung ({}-{})", sa, sb),
                suggestions: vec![format!("{}-{}", sa, sb), sa.to_string()],
                lang: ProofLang::Id,
            });
        }
    }
}

fn check_spaces(text: &str, out: &mut Vec<Issue>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' {
            let start = i;
            while i < bytes.len() && bytes[i] == b' ' {
                i += 1;
            }
            let count = i - start;
            if count > 1 {
                out.push(Issue {
                    start,
                    end: i,
                    kind: IssueKind::Grammar,
                    message: "Spasi ganda".into(),
                    suggestions: vec![" ".into()],
                    lang: ProofLang::Id,
                });
            }
            if i < bytes.len() && matches!(bytes[i], b',' | b'.' | b'!' | b'?' | b':' | b';') {
                out.push(Issue {
                    start,
                    end: start + count,
                    kind: IssueKind::Grammar,
                    message: "Hapus spasi sebelum tanda baca".into(),
                    suggestions: vec![String::new()],
                    lang: ProofLang::Id,
                });
            }
        } else {
            i += 1;
        }
    }
}

fn check_preposition_spacing(text: &str, out: &mut Vec<Issue>) {
    // Kata depan yang sering salah digabung: dimana, kemana, disana, disini, diatas, dibawah
    let cases = [
        ("dimana", "di mana"),
        ("kemana", "ke mana"),
        ("disana", "di sana"),
        ("disini", "di sini"),
        ("diatas", "di atas"),
        ("dibawah", "di bawah"),
        ("kedepan", "ke depan"),
    ];

    let w = words(text);
    for (start, end) in w {
        let word = text.get(start..end).unwrap_or("");
        for (wrong, right) in &cases {
            if word.eq_ignore_ascii_case(wrong) {
                let is_capital = word.chars().next().is_some_and(|c| c.is_uppercase());
                let sug = if is_capital {
                    let mut s = right.to_string();
                    if let Some(c) = s.chars().next() {
                        s.replace_range(0..c.len_utf8(), &c.to_uppercase().to_string());
                    }
                    s
                } else {
                    right.to_string()
                };
                out.push(Issue {
                    start,
                    end,
                    kind: IssueKind::Grammar,
                    message: format!("Kata depan dipisah: '{}'", right),
                    suggestions: vec![sug],
                    lang: ProofLang::Id,
                });
            }
        }
    }
}

fn check_comma_conjunctions(text: &str, out: &mut Vec<Issue>) {
    let w = words(text);
    for window in w.windows(2) {
        let (prev, curr) = (window[0], window[1]);
        let curr_word = text.get(curr.0..curr.1).unwrap_or("").to_lowercase();
        if COMMA_CONJUNCTIONS.contains(&curr_word.as_str()) {
            let gap = text.get(prev.1..curr.0).unwrap_or("");
            if !gap.contains(',') && !gap.contains('.') && !gap.contains(';') && !gap.contains(':') && prev.0 > 0 {
                out.push(Issue {
                    start: prev.1,
                    end: curr.0,
                    kind: IssueKind::Grammar,
                    message: format!("Gunakan tanda koma sebelum konjungsi pertentangan '{}'", curr_word),
                    suggestions: vec![format!(", {}", gap.trim_start())],
                    lang: ProofLang::Id,
                });
            }
        }
    }
}

fn check_capital_after_period(text: &str, out: &mut Vec<Issue>) {
    let w = words(text);
    for window in w.windows(2) {
        let (a, b) = (window[0], window[1]);
        let prev = text.get(a.0..a.1).unwrap_or("");
        let gap = text.get(a.1..b.0).unwrap_or("");
        let next = text.get(b.0..b.1).unwrap_or("");

        if gap.contains('.') && !gap.contains("..") {
            let lower_prev = prev.to_lowercase();
            if ABBREVIATIONS.contains(&lower_prev.as_str()) {
                continue;
            }
            if let Some(first_char) = next.chars().next().filter(|c| c.is_lowercase()) {
                let mut capitalized = next.to_string();
                capitalized.replace_range(0..first_char.len_utf8(), &first_char.to_uppercase().to_string());
                out.push(Issue {
                    start: b.0,
                    end: b.1,
                    kind: IssueKind::Grammar,
                    message: "Awal kalimat sebaiknya diawali huruf kapital".into(),
                    suggestions: vec![capitalized],
                    lang: ProofLang::Id,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indonesian_grammar_rules() {
        let issues = check("Saya pergi kemana saja");
        assert!(issues.iter().any(|i| i.suggestions.contains(&"ke mana".to_string())));

        let issues = check("buku buku itu rapi");
        assert!(issues.iter().any(|i| i.suggestions.contains(&"buku-buku".to_string())));

        let issues = check("Saya mau  makan");
        assert!(issues.iter().any(|i| i.message == "Spasi ganda"));

        let issues = check("Saya lapar tetapi tidak ada makanan");
        assert!(issues.iter().any(|i| i.message.contains("tetapi")));

        let issues = check("Kemarin hujan. hari ini cerah.");
        assert!(issues.iter().any(|i| i.suggestions.contains(&"Hari".to_string())));
    }
}
