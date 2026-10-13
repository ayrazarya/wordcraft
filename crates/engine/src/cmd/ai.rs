//! AI Writing Assistant commands (rewrite, summarize, fix grammar, generate draft).
//! Designed to run headlessly, through the UI sidebar, or via agent/MCP automation.

use serde_json::{Value, json};
use wordcraft_proof::{ProofLang, check_grammar_in};

use super::{sel_result, type_text};
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("ai.rewrite", "AI Rewrite", "Home › AI Assistant", rewrite)
            .params(r#"{"text"?: string, "tone"?: "formal|academic|casual|concise|expanded", "replace"?: bool}"#),
        CommandSpec::new("ai.summarize", "AI Summarize", "Home › AI Assistant", summarize)
            .params(r#"{"text"?: string, "format"?: "bullets|paragraph", "insert"?: bool}"#),
        CommandSpec::new("ai.fixGrammar", "AI Fix Grammar", "Home › AI Assistant", fix_grammar).params(r#"{"text"?: string, "replace"?: bool}"#),
        CommandSpec::new("ai.generateDraft", "AI Generate Draft", "Home › AI Assistant", generate_draft)
            .params(r#"{"prompt": string, "kind"?: "letter|essay|meeting_notes|general", "insert"?: bool}"#),
        CommandSpec::new("ai.context", "AI Get Document Context", "Home › AI Assistant", ai_context).pure(),
        CommandSpec::new("ai.insertWithFormat", "AI Insert Formatted Text", "Home › AI Assistant", insert_with_format)
            .params(r#"{"text": string, "replace"?: bool}"#),
        CommandSpec::new("ai.suggest", "AI Sentence Suggestions", "Home › AI Assistant", suggest).params(r#"{"text"?: string}"#).pure(),
    ]
}

fn rewrite(s: &mut Session, v: &Value) -> CmdResult {
    let text = match p::str(v, "text") {
        Some(t) => t.to_string(),
        None => s.selected_text(),
    };
    if text.trim().is_empty() {
        return Err(CmdError::Params("no text provided or selected to rewrite".into()));
    }
    let tone = p::str(v, "tone").unwrap_or("formal");
    let rewritten = perform_rewrite(&text, tone);
    let should_replace = p::bool(v, "replace").unwrap_or(!s.sel.is_collapsed());

    if should_replace && !s.sel.is_collapsed() {
        let _ = type_text(s, &rewritten);
    }

    Ok(json!({
        "original": text,
        "rewritten": rewritten,
        "tone": tone,
        "replaced": should_replace && !s.sel.is_collapsed(),
    }))
}

fn perform_rewrite(text: &str, tone: &str) -> String {
    match tone {
        "formal" => {
            let mut out = text.to_string();
            let replacements = [
                ("nggak", "tidak"),
                ("ngga", "tidak"),
                ("gak", "tidak"),
                ("ga ", "tidak "),
                ("udah", "sudah"),
                ("udh", "sudah"),
                ("kalo", "jika"),
                ("klo", "jika"),
                ("buat ", "untuk "),
                ("pake", "menggunakan"),
                ("banget", "sangat"),
                ("bgt", "sangat"),
                ("gimana", "bagaimana"),
                ("bisa jadi", "dapat menjadi"),
                ("cuman", "hanya"),
                ("cuma", "hanya"),
                ("bener", "benar"),
                ("beneran", "sungguh"),
                ("donk", "tentunya"),
                ("dong", "tentunya"),
                ("tapi", "tetapi"),
            ];
            for (from, to) in replacements {
                out = out.replace(from, to);
            }
            out
        }
        "academic" => {
            let mut out = text.to_string();
            let replacements = [
                ("saya rasa", "berdasarkan analisis yang dilakukan"),
                ("menurut saya", "berdasarkan kajian teoretis"),
                ("sangat bagus", "menunjukkan hasil yang optimal"),
                ("sangat penting", "memiliki signifikansi yang tinggi"),
                ("banyak faktor", "berbagai variabel pendukung"),
                ("membuktikan bahwa", "mengindikasikan bahwa"),
            ];
            for (from, to) in replacements {
                out = out.replace(from, to);
            }
            out
        }
        "concise" => {
            let mut out = text.to_string();
            let removals =
                ["sebenarnya ", "sejujurnya ", "pada dasarnya ", "dalam rangka untuk ", "perlu diketahui bahwa ", "dapat dikatakan bahwa "];
            for r in removals {
                out = out.replace(r, "");
            }
            out
        }
        _ => text.to_string(),
    }
}

fn summarize(s: &mut Session, v: &Value) -> CmdResult {
    let text = match p::str(v, "text") {
        Some(t) => t.to_string(),
        None => {
            let sel = s.selected_text();
            if sel.trim().is_empty() { s.doc.plain_text(wordcraft_doc::StoryRef::Body) } else { sel }
        }
    };
    if text.trim().is_empty() {
        return Err(CmdError::Params("no text to summarize".into()));
    }
    let format = p::str(v, "format").unwrap_or("bullets");
    let sentences: Vec<&str> = text.split(['.', '!', '?']).map(str::trim).filter(|s| s.len() > 15).collect();

    let points: Vec<String> = sentences.iter().take(5).map(|s| format!("- {s}.")).collect();

    let summary = if format == "paragraph" { sentences.iter().take(3).copied().collect::<Vec<&str>>().join(". ") + "." } else { points.join("\n") };

    if p::bool(v, "insert").unwrap_or(false) {
        let _ = type_text(s, &summary);
    }

    Ok(json!({
        "summary": summary,
        "points": points,
        "format": format,
    }))
}

fn fix_grammar(s: &mut Session, v: &Value) -> CmdResult {
    let text = match p::str(v, "text") {
        Some(t) => t.to_string(),
        None => s.selected_text(),
    };
    if text.trim().is_empty() {
        return Err(CmdError::Params("no text to fix".into()));
    }
    let issues = check_grammar_in(&text, ProofLang::Id);
    let mut fixed = text.clone();
    let mut changed = false;

    // Apply suggestions backwards to preserve byte indices
    let mut sorted_issues = issues;
    sorted_issues.sort_by_key(|b| std::cmp::Reverse(b.start));
    for issue in sorted_issues {
        if let Some(sug) = issue.suggestions.first().filter(|_| issue.start < issue.end && issue.end <= fixed.len()) {
            fixed.replace_range(issue.start..issue.end, sug);
            changed = true;
        }
    }

    let should_replace = p::bool(v, "replace").unwrap_or(!s.sel.is_collapsed());
    if should_replace && !s.sel.is_collapsed() {
        let _ = type_text(s, &fixed);
    }

    Ok(json!({
        "original": text,
        "fixed": fixed,
        "changed": changed,
        "replaced": should_replace && !s.sel.is_collapsed(),
    }))
}

fn generate_draft(s: &mut Session, v: &Value) -> CmdResult {
    let prompt = p::req_str(v, "prompt")?;
    let kind = p::str(v, "kind").unwrap_or("general");

    let draft = match kind {
        "letter" => format!(
            "SURAT PENGANTAR / DINAS\n\nHal: {prompt}\n\nDengan hormat,\n\nSehubungan dengan {prompt}, bersama ini kami sampaikan penjelasan dan permohonan koordinasi lebih lanjut.\n\nDemikian kami sampaikan, terima kasih atas perhatian dan kerja samanya.\n\nHormat kami,\n\n[Nama Pengirim]"
        ),
        "meeting_notes" => format!(
            "NOTULA RAPAT\n\nAgenda: {prompt}\nTanggal: [Tanggal Rapat]\nPemimpin Rapat: [Nama Pemimpin]\n\n1. Pembahasan Utama:\n   - Pembahasan mengenai {prompt}.\n\n2. Keputusan dan Tindak Lanjut:\n   - Disepakati langkah-langkah implementasi strategis.\n\nNotulis: [Nama Notulis]"
        ),
        _ => format!(
            "DRAF: {prompt}\n\n1. Pendahuluan\nBagian ini menguraikan latar belakang dan signifikansi dari {prompt}.\n\n2. Pembahasan\nAnalisis terperinci mengenai implementasi dan aspek kunci terkait.\n\n3. Kesimpulan\nLangkah-langkah yang direkomendasikan untuk pelaksanaan tindak lanjut."
        ),
    };

    if p::bool(v, "insert").unwrap_or(false) {
        let _ = type_text(s, &draft);
        sel_result(s)
    } else {
        Ok(json!({ "prompt": prompt, "kind": kind, "draft": draft }))
    }
}

fn ai_context(s: &mut Session, _v: &Value) -> CmdResult {
    let sel = s.selected_text();
    let f = &s.sel.focus;
    let para = s.doc.para_at(f);
    let para_text = para.map(|p| p.text.clone()).unwrap_or_default();
    let style = para.and_then(|p| p.props.style.clone()).unwrap_or_else(|| "Normal".into());
    let typing = s.typing_props();
    let resolved =
        if let Some(p) = para { s.doc.styles.resolve_char(p.props.style.as_deref(), &typing) } else { s.doc.styles.resolve_char(None, &typing) };
    let doc_text = s.doc.plain_text(wordcraft_doc::StoryRef::Body);
    let truncated_doc = if doc_text.len() > 10_000 {
        let mut t = doc_text.chars().take(2_500).collect::<String>();
        t.push_str("\n... [dipotong untuk efisiensi context] ...");
        t
    } else {
        doc_text
    };

    Ok(json!({
        "selectedText": sel,
        "currentPara": para_text,
        "documentText": truncated_doc,
        "wordCount": s.word_count(),
        "font": resolved.font,
        "fontSize": resolved.size,
        "style": style,
        "bold": resolved.bold,
        "italic": resolved.italic,
    }))
}

fn insert_with_format(s: &mut Session, v: &Value) -> CmdResult {
    let text = p::req_str(v, "text")?;
    let should_replace = p::bool(v, "replace").unwrap_or(!s.sel.is_collapsed());
    if !should_replace && !s.sel.is_collapsed() {
        // Collapse selection to cursor before inserting so existing text is preserved
        s.sel.anchor = s.sel.focus.clone();
    }
    type_text(s, text)?;
    sel_result(s)
}

fn suggest(s: &mut Session, v: &Value) -> CmdResult {
    let prefix = match p::str(v, "text") {
        Some(t) => t.to_string(),
        None => {
            let f = &s.sel.focus;
            s.doc
                .para_at(f)
                .map(|p| {
                    let off = f.off.min(p.len());
                    p.text.get(..off).unwrap_or("").to_string()
                })
                .unwrap_or_default()
        }
    };
    let suggestions = generate_suggestions(&prefix);
    Ok(json!({
        "prefix": prefix,
        "suggestions": suggestions,
    }))
}

fn generate_suggestions(prefix: &str) -> Vec<String> {
    let trimmed = prefix.trim();
    if trimmed.ends_with('.') || trimmed.ends_with('?') || trimmed.ends_with('!') || trimmed.is_empty() {
        vec![
            "Oleh karena itu, diperlukan langkah strategis dan koordinasi yang berkesinambungan.".into(),
            "Hal ini sejalan dengan tujuan utama yang ingin dicapai dalam program ini.".into(),
            "Berdasarkan analisis data tersebut, dapat disimpulkan bahwa implementasi berjalan optimal.".into(),
        ]
    } else if trimmed.ends_with(',') {
        vec![
            "sehingga seluruh pihak dapat memahami mekanisme pelaksanaannya dengan jelas.".into(),
            "mengingat urgensi dan batas waktu yang telah ditentukan sebelumnya.".into(),
            "serta memberikan dampak positif yang berkelanjutan bagi perkembangan ke depan.".into(),
        ]
    } else {
        vec![
            format!("{trimmed} secara komprehensif dan terstruktur dengan baik."),
            format!("{trimmed} demi mencapai hasil yang optimal sesuai standar operasional."),
            format!("{trimmed} dengan mempertimbangkan berbagai indikator keberhasilan yang telah ditetapkan."),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_rewrite() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        let res = s.run("ai.rewrite", &json!({"text": "kalo udah selesai kabari", "tone": "formal"})).unwrap();
        assert_eq!(res["rewritten"], "jika sudah selesai kabari");
    }

    #[test]
    fn test_ai_summarize() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        let text = "Ini adalah kalimat pertama yang cukup panjang. Ini adalah kalimat kedua yang juga sangat informatif. Ini adalah kalimat ketiga yang melengkapi paragraf.";
        let res = s.run("ai.summarize", &json!({"text": text, "format": "bullets"})).unwrap();
        assert!(res["summary"].as_str().unwrap().contains("- "));
    }

    #[test]
    fn test_ai_generate_draft() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        let res = s.run("ai.generateDraft", &json!({"prompt": "Koordinasi Proyek", "kind": "letter"})).unwrap();
        assert!(res["draft"].as_str().unwrap().contains("Koordinasi Proyek"));
    }

    #[test]
    fn test_ai_context_and_format() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        s.run("text.insert", &json!({"text": "Pengantar Skripsi"})).unwrap();
        let ctx = s.run("ai.context", &json!({})).unwrap();
        assert_eq!(ctx["currentPara"], "Pengantar Skripsi");
        assert!(ctx["font"].is_string());
        assert!(ctx["fontSize"].is_number());
    }

    #[test]
    fn test_ai_suggest() {
        let mut s = Session::new(wordcraft_doc::Document::new());
        let res = s.run("ai.suggest", &json!({"text": "Pelaksanaan kegiatan ini,"})).unwrap();
        let list = res["suggestions"].as_array().unwrap();
        assert_eq!(list.len(), 3);
    }
}
