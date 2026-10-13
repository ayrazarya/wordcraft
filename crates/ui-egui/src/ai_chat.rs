//! AI Assistant chatbot sidebar panel (chat, API credentials, sentence suggestions,
//! document reading & editing, format & font preservation).

use crate::WordApp;
use crate::theme::{Tokens, regular, semibold};
use egui::{Color32, Stroke, Ui};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use serde_json::json;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Saved AI API configuration (persisted in ui.json).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "openai_compatible".into(),
            endpoint: "http://localhost:11434/v1".into(),
            api_key: String::new(),
            model: "llama3.2".into(),
            system_prompt: "Anda adalah asisten penulisan cerdas di WordCraft. Bantu pengguna menulis, menyunting, memperbaiki tata bahasa, dan memformat dokumen dengan kualitas tinggi dalam bahasa Indonesia atau bahasa yang diminta. Jaga kesesuaian gaya penulisan dokumen.".into(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// Message role in the AI conversation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

/// A chat message in the AI assistant history.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiMessage {
    pub id: u64,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
    pub context_badge: Option<String>,
    pub font_info: Option<String>,
}

/// Current active tab in the AI pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AiTab {
    #[default]
    Chat,
    Suggestions,
    QuickTools,
    Settings,
}

/// State for the AI Assistant panel.
pub struct AiChatState {
    pub messages: Vec<AiMessage>,
    pub input_text: String,
    pub is_busy: bool,
    pub active_tab: AiTab,
    pub include_selection: bool,
    pub include_full_doc: bool,
    pub show_api_key: bool,
    pub connection_status: Option<(bool, String)>,
    pub testing_connection: bool,
    pub suggestions: Vec<String>,
    pub suggestion_prefix: String,
    pub suggestion_busy: bool,
    next_msg_id: u64,
    tx_req: Option<Sender<AiTask>>,
    rx_res: Option<Receiver<AiResult>>,
}

#[allow(dead_code)]
enum AiTask {
    Chat {
        config: AiConfig,
        messages: Vec<(String, String)>, // (role, content)
    },
    Suggest {
        config: AiConfig,
        prefix: String,
        context: String,
    },
    TestConnection {
        config: AiConfig,
    },
}

#[allow(dead_code)]
enum AiResult {
    ChatSuccess { content: String, font_info: Option<String> },
    ChatError(String),
    SuggestSuccess(Vec<String>),
    SuggestError(String),
    TestConnectionSuccess(String),
    TestConnectionError(String),
}

impl Default for AiChatState {
    fn default() -> Self {
        let (tx, rx) = channel();
        let (tx_worker, rx_worker) = channel();

        // Spawn background worker on native platforms
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = std::thread::Builder::new().name("wordcraft-ai-worker".into()).spawn(move || {
                ai_worker_loop(rx_worker, tx);
            });
        }
        #[cfg(target_arch = "wasm32")]
        let (_tx, _rx_worker) = (tx, rx_worker);

        let initial_messages = vec![AiMessage {
            id: 1,
            role: MessageRole::Assistant,
            content: "Halo! Saya Asisten AI WordCraft. Saya dapat membaca dokumen Anda, membantu menulis, memperbaiki tata bahasa, atau memberikan saran kalimat dengan format dan font yang otomatis menyesuaikan dokumen Anda.\n\nSilakan konfigurasikan kredensial API Anda di tab Pengaturan jika belum terhubung.".into(),
            timestamp: "Sekarang".into(),
            context_badge: None,
            font_info: None,
        }];

        Self {
            messages: initial_messages,
            input_text: String::new(),
            is_busy: false,
            active_tab: AiTab::Chat,
            include_selection: true,
            include_full_doc: false,
            show_api_key: false,
            connection_status: None,
            testing_connection: false,
            suggestions: Vec::new(),
            suggestion_prefix: String::new(),
            suggestion_busy: false,
            next_msg_id: 2,
            tx_req: Some(tx_worker),
            rx_res: Some(rx),
        }
    }
}

impl AiChatState {
    /// Poll for responses from the background worker.
    pub fn poll(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.rx_res else { return };
        let mut repainted = false;

        while let Ok(res) = rx.try_recv() {
            repainted = true;
            match res {
                AiResult::ChatSuccess { content, font_info } => {
                    self.is_busy = false;
                    let id = self.next_msg_id;
                    self.next_msg_id += 1;
                    self.messages.push(AiMessage {
                        id,
                        role: MessageRole::Assistant,
                        content,
                        timestamp: "Baru saja".into(),
                        context_badge: None,
                        font_info,
                    });
                }
                AiResult::ChatError(err) => {
                    self.is_busy = false;
                    let id = self.next_msg_id;
                    self.next_msg_id += 1;
                    self.messages.push(AiMessage {
                        id,
                        role: MessageRole::Error,
                        content: format!(
                            "Gagal memproses permintaan AI:\n{err}\n\nPastikan Endpoint URL, Model, dan API Key di tab Pengaturan sudah benar."
                        ),
                        timestamp: "Error".into(),
                        context_badge: None,
                        font_info: None,
                    });
                }
                AiResult::SuggestSuccess(items) => {
                    self.suggestion_busy = false;
                    self.suggestions = items;
                }
                AiResult::SuggestError(err) => {
                    self.suggestion_busy = false;
                    self.suggestions = vec![format!("Gagal memuat saran: {err}")];
                }
                AiResult::TestConnectionSuccess(msg) => {
                    self.testing_connection = false;
                    self.connection_status = Some((true, msg));
                }
                AiResult::TestConnectionError(err) => {
                    self.testing_connection = false;
                    self.connection_status = Some((false, err));
                }
            }
        }

        if repainted {
            ctx.request_repaint();
        }
    }
}

/// Render the AI Assistant sidebar panel.
pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());

    // Navigation Tab Bar
    ui.horizontal(|ui| {
        let tabs = [(AiTab::Chat, "Chat"), (AiTab::Suggestions, "Saran"), (AiTab::QuickTools, "Alat"), (AiTab::Settings, "API")];
        for (tab, label) in tabs {
            let active = app.ai_chat.active_tab == tab;
            let resp = ui.selectable_label(active, label);
            if resp.clicked() {
                app.ai_chat.active_tab = tab;
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();

    // Context & Document Formatting Indicator
    let doc_format = detect_document_styling(&app.session);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Format Aktif:").small().color(t.text_dim));
        let mut badge_text = format!("{} {}pt ({})", doc_format.font, doc_format.size, doc_format.style);
        if doc_format.bold {
            badge_text.push_str(" [Tebal]");
        }
        if doc_format.italic {
            badge_text.push_str(" [Miring]");
        }
        ui.label(egui::RichText::new(badge_text).small().strong().color(t.accent));
    });

    let sel_len = app.session.selected_text().chars().count();
    if sel_len > 0 {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Terpilih:").small().color(t.text_dim));
            ui.label(egui::RichText::new(format!("{sel_len} karakter")).small().strong());
        });
    }

    ui.separator();
    ui.add_space(2.0);

    match app.ai_chat.active_tab {
        AiTab::Chat => render_chat_tab(app, ui, &doc_format),
        AiTab::Suggestions => render_suggestions_tab(app, ui, &doc_format),
        AiTab::QuickTools => render_quick_tools_tab(app, ui, &doc_format),
        AiTab::Settings => render_settings_tab(app, ui),
    }
}

struct DocFormatInfo {
    font: String,
    size: f32,
    style: String,
    bold: bool,
    italic: bool,
}

fn detect_document_styling(s: &wordcraft_engine::Session) -> DocFormatInfo {
    let f = &s.sel.focus;
    let para = s.doc.para_at(f);
    let style = para.and_then(|p| p.props.style.clone()).unwrap_or_else(|| "Normal".into());
    let typing = s.typing_props();
    let resolved =
        if let Some(p) = para { s.doc.styles.resolve_char(p.props.style.as_deref(), &typing) } else { s.doc.styles.resolve_char(None, &typing) };

    DocFormatInfo { font: resolved.font, size: resolved.size, style, bold: resolved.bold, italic: resolved.italic }
}

fn render_chat_tab(app: &mut WordApp, ui: &mut Ui, doc_format: &DocFormatInfo) {
    let t = Tokens::get(ui.ctx());

    // Chat Message History Area
    let mut insert_action: Option<(String, bool)> = None;
    let avail_height = ui.available_height() - 110.0;
    egui::ScrollArea::vertical().max_height(avail_height.max(120.0)).auto_shrink([false, false]).stick_to_bottom(true).show(ui, |ui| {
        for msg in &app.ai_chat.messages {
            render_message_bubble(ui, msg, &t, &mut insert_action);
        }
        if app.ai_chat.is_busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(egui::RichText::new("AI sedang merespons...").italics().color(t.text_dim));
            });
        }
    });

    if let Some((text, replace)) = insert_action {
        insert_ai_text_into_document(app, &text, replace);
    }

    ui.separator();

    // Context Toggles
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.ai_chat.include_selection, "Sertakan Teks Terpilih");
        ui.checkbox(&mut app.ai_chat.include_full_doc, "Seluruh Dokumen");
    });

    // Input Area
    ui.horizontal(|ui| {
        let input_edit = egui::TextEdit::multiline(&mut app.ai_chat.input_text)
            .desired_rows(2)
            .desired_width(ui.available_width() - 65.0)
            .hint_text("Tulis pesan ke AI... (Ctrl+Enter kirim)");

        let resp = ui.add(input_edit);
        let send_btn = ui.button("Kirim");

        let enter_pressed = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
        if (send_btn.clicked() || (resp.has_focus() && enter_pressed)) && !app.ai_chat.is_busy {
            send_user_message(app, doc_format);
        }
    });
}

fn render_message_bubble(ui: &mut Ui, msg: &AiMessage, t: &Tokens, insert_action: &mut Option<(String, bool)>) {
    let (bg_color, align_right) = match msg.role {
        MessageRole::User => (t.accent.linear_multiply(0.12), true),
        MessageRole::Assistant => (t.panel.linear_multiply(0.7), false),
        MessageRole::System => (Color32::from_gray(60), false),
        MessageRole::Error => (Color32::from_rgb(120, 30, 30), false),
    };

    let frame = egui::Frame::NONE.fill(bg_color).corner_radius(6.0).inner_margin(8).stroke(Stroke::new(1.0, t.border));

    ui.with_layout(if align_right { egui::Layout::right_to_left(egui::Align::Min) } else { egui::Layout::left_to_right(egui::Align::Min) }, |ui| {
        frame.show(ui, |ui| {
            ui.set_max_width(ui.available_width() * 0.95);
            ui.vertical(|ui| {
                let role_label = match msg.role {
                    MessageRole::User => "Anda",
                    MessageRole::Assistant => "AI Assistant",
                    MessageRole::System => "Sistem",
                    MessageRole::Error => "Kesalahan",
                };

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(role_label).small().font(semibold(12.0)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&msg.timestamp).small().weak());
                    });
                });

                ui.add_space(2.0);
                ui.label(egui::RichText::new(&msg.content).font(regular(13.0)));

                if msg.role == MessageRole::Assistant {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button("Sisipkan di Kursor")
                            .on_hover_text("Sisipkan teks dengan mempertahankan font & formatting dokumen saat ini")
                            .clicked()
                        {
                            *insert_action = Some((msg.content.clone(), false));
                        }
                        if ui.button("Gantikan Seleksi").on_hover_text("Gantikan teks yang dipilih dengan teks AI ini").clicked() {
                            *insert_action = Some((msg.content.clone(), true));
                        }
                        if ui.button("Salin").clicked() {
                            ui.ctx().copy_text(msg.content.clone());
                        }
                    });
                }
            });
        });
    });
    ui.add_space(4.0);
}

fn send_user_message(app: &mut WordApp, doc_format: &DocFormatInfo) {
    let text = app.ai_chat.input_text.trim().to_string();
    if text.is_empty() {
        return;
    }

    app.ai_chat.input_text.clear();
    let id = app.ai_chat.next_msg_id;
    app.ai_chat.next_msg_id += 1;

    app.ai_chat.messages.push(AiMessage {
        id,
        role: MessageRole::User,
        content: text.clone(),
        timestamp: "Baru saja".into(),
        context_badge: None,
        font_info: None,
    });

    app.ai_chat.is_busy = true;

    // Collect conversation history
    let mut conv = Vec::new();
    let sys_prompt = app.ui.ai_config.system_prompt.clone();
    let font_summary = format!("Font dokumen aktif: {}, ukuran: {}pt, gaya: {}", doc_format.font, doc_format.size, doc_format.style);

    let mut full_system = format!(
        "{sys_prompt}\n[INFORMASI FORMAT DOKUMEN: {font_summary}]\nSaat memberikan saran penulisan atau kalimat, hasilkan teks yang langsung siap disisipkan ke dalam dokumen."
    );

    if app.ai_chat.include_selection {
        let sel = app.session.selected_text();
        if !sel.trim().is_empty() {
            full_system.push_str(&format!("\n\n[TEKS TERPILIH SAAT INI]:\n{sel}"));
        }
    }

    if app.ai_chat.include_full_doc {
        let full = app.session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        let capped = if full.len() > 8000 { full.chars().take(2000).collect::<String>() } else { full };
        full_system.push_str(&format!("\n\n[DOKUMEN PENUH]:\n{capped}"));
    }

    conv.push(("system".into(), full_system));

    for m in &app.ai_chat.messages {
        // Skip the initial static greeting so the conversation always begins with the user's turn
        if m.id == 1 && m.role == MessageRole::Assistant {
            continue;
        }
        let r = match m.role {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            _ => continue,
        };
        conv.push((r.into(), m.content.clone()));
    }

    let config = app.ui.ai_config.clone();
    if let Some(tx) = &app.ai_chat.tx_req {
        let _ = tx.send(AiTask::Chat { config, messages: conv });
    }
}

fn insert_ai_text_into_document(app: &mut WordApp, text: &str, replace_selection: bool) {
    // Preserve formatting: Session::type_text / text.insert automatically adopts
    // the document's typing_props at the cursor / selection.
    let _ = app.session.run(
        "ai.insertWithFormat",
        &json!({
            "text": text,
            "replace": replace_selection,
        }),
    );
}

fn render_suggestions_tab(app: &mut WordApp, ui: &mut Ui, doc_format: &DocFormatInfo) {
    let t = Tokens::get(ui.ctx());

    ui.label(egui::RichText::new("Saran Kelanjutan Kalimat").font(semibold(14.0)));
    ui.label(
        egui::RichText::new("Menghasilkan ide kelanjutan kalimat berdasarkan konteks paragraf aktif, menggunakan model AI yang Anda pasang.")
            .small()
            .color(t.text_dim),
    );

    ui.add_space(6.0);

    let f = &app.session.sel.focus;
    let para_text = app.session.doc.para_at(f).map(|p| p.text.clone()).unwrap_or_default();
    let sample = if para_text.trim().is_empty() {
        "(Paragraf saat ini kosong)".into()
    } else {
        let end = f.off.min(para_text.len());
        para_text.get(..end).unwrap_or(&para_text).to_string()
    };

    ui.group(|ui| {
        ui.label(egui::RichText::new("Konteks Paragraf Saat Ini:").small().strong());
        ui.label(egui::RichText::new(&sample).small().italics());
    });

    ui.add_space(6.0);

    let btn_label = if app.ai_chat.suggestion_busy { "Mengambil Saran AI..." } else { "Minta Saran Kelanjutan Kalimat" };
    if ui.button(btn_label).clicked() && !app.ai_chat.suggestion_busy {
        app.ai_chat.suggestion_busy = true;
        app.ai_chat.suggestion_prefix = sample.clone();
        let config = app.ui.ai_config.clone();
        if let Some(tx) = &app.ai_chat.tx_req {
            let _ = tx.send(AiTask::Suggest { config, prefix: sample, context: format!("Font: {}, Size: {}pt", doc_format.font, doc_format.size) });
        }
    }

    ui.add_space(8.0);
    ui.separator();

    let mut to_insert: Option<String> = None;
    if app.ai_chat.suggestions.is_empty() {
        ui.label(egui::RichText::new("Belum ada saran. Klik tombol di atas untuk meminta AI menyarankan kalimat berikutnya.").small().weak());
    } else {
        ui.label(egui::RichText::new("Pilihan Saran Kalimat:").small().strong());
        for (i, sug) in app.ai_chat.suggestions.iter().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("{}.", i + 1)).strong());
                    ui.label(egui::RichText::new(sug).font(regular(13.0)));
                });
                ui.horizontal(|ui| {
                    if ui.button("Sisipkan ke Dokumen").clicked() {
                        let text_to_insert = if sug.starts_with(' ') { sug.clone() } else { format!(" {sug}") };
                        to_insert = Some(text_to_insert);
                    }
                    if ui.button("Salin").clicked() {
                        ui.ctx().copy_text(sug.clone());
                    }
                });
            });
            ui.add_space(3.0);
        }
    }

    if let Some(text) = to_insert {
        insert_ai_text_into_document(app, &text, false);
    }
}

fn render_quick_tools_tab(app: &mut WordApp, ui: &mut Ui, _doc_format: &DocFormatInfo) {
    let t = Tokens::get(ui.ctx());

    ui.label(egui::RichText::new("Alat Cepat AI").font(semibold(14.0)));
    ui.label(egui::RichText::new("Operasi penulisan cepat satu-klik untuk teks terpilih atau dokumen aktif.").small().color(t.text_dim));

    ui.add_space(8.0);

    let tools = [
        ("Perbaiki Tata Bahasa (PUEBI/KBBI)", "ai.fixGrammar", "Perbaiki ejaan, kata baku, dan tata bahasa teks terpilih"),
        ("Ubah Menjadi Lebih Formal", "ai.rewrite", "Ubah nada bahasa menjadi formal dan baku"),
        ("Ubah Menjadi Nada Akademik", "ai.rewrite", "Sesuaikan gaya bahasa untuk skripsi/karya ilmiah"),
        ("Buat Ringkasan Poin-Poin", "ai.summarize", "Ringkas teks menjadi poin-poin penting"),
        ("Buat Draf Surat Resmi", "ai.generateDraft", "Buat draf kerangka surat resmi standar"),
    ];

    for (name, cmd, desc) in tools {
        ui.group(|ui| {
            ui.label(egui::RichText::new(name).strong());
            ui.label(egui::RichText::new(desc).small().color(t.text_dim));
            if ui.button("Terapkan").clicked() {
                match cmd {
                    "ai.fixGrammar" => {
                        let _ = app.run("ai.fixGrammar", json!({"replace": true}));
                    }
                    "ai.rewrite" => {
                        let tone = if name.contains("Akademik") { "academic" } else { "formal" };
                        let _ = app.run("ai.rewrite", json!({"tone": tone, "replace": true}));
                    }
                    "ai.summarize" => {
                        let _ = app.run("ai.summarize", json!({"format": "bullets", "insert": true}));
                    }
                    "ai.generateDraft" => {
                        let _ = app.run("ai.generateDraft", json!({"prompt": "Koordinasi Resmi", "kind": "letter", "insert": true}));
                    }
                    _ => {}
                }
            }
        });
        ui.add_space(3.0);
    }
}

fn render_settings_tab(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());

    ui.label(egui::RichText::new("Pengaturan Kredensial AI API").font(semibold(14.0)));
    ui.label(egui::RichText::new("Konfigurasikan API key dan endpoint AI (OpenAI, Ollama, DeepSeek, OpenRouter, dll). Data disimpan secara lokal di konfigurasi aplikasi Anda.").small().color(t.text_dim));

    ui.add_space(6.0);

    // Preset buttons
    ui.label(egui::RichText::new("Preset Cepat:").small().strong());
    ui.horizontal_wrapped(|ui| {
        if ui.button("Google Gemma").on_hover_text("Preset untuk Google Gemma (mis. gemma-4-26b-a4b-it) via Google AI Studio API").clicked() {
            app.ui.ai_config.endpoint = "https://generativelanguage.googleapis.com/v1beta".into();
            app.ui.ai_config.model = "gemma-4-26b-a4b-it".into();
            app.ui.ai_config.provider = "google".into();
        }
        if ui.button("Google Gemini").on_hover_text("Preset untuk Google Gemini (mis. gemini-3.8-flash) via Google AI Studio API").clicked() {
            app.ui.ai_config.endpoint = "https://generativelanguage.googleapis.com/v1beta".into();
            app.ui.ai_config.model = "gemini-3.8-flash".into();
            app.ui.ai_config.provider = "google".into();
        }
        if ui.button("Ollama (Lokal)").clicked() {
            app.ui.ai_config.endpoint = "http://localhost:11434/v1".into();
            app.ui.ai_config.model = "llama3.2".into();
            app.ui.ai_config.api_key.clear();
            app.ui.ai_config.provider = "openai_compatible".into();
        }
        if ui.button("OpenAI").clicked() {
            app.ui.ai_config.endpoint = "https://api.openai.com/v1".into();
            app.ui.ai_config.model = "gpt-4o-mini".into();
            app.ui.ai_config.provider = "openai_compatible".into();
        }
        if ui.button("OpenRouter").clicked() {
            app.ui.ai_config.endpoint = "https://openrouter.ai/api/v1".into();
            app.ui.ai_config.model = "deepseek/deepseek-chat".into();
            app.ui.ai_config.provider = "openai_compatible".into();
        }
        if ui.button("Groq").clicked() {
            app.ui.ai_config.endpoint = "https://api.groq.com/openai/v1".into();
            app.ui.ai_config.model = "llama-3.3-70b-versatile".into();
            app.ui.ai_config.provider = "openai_compatible".into();
        }
    });

    ui.add_space(6.0);
    ui.separator();

    // Endpoint URL
    ui.label(egui::RichText::new("Endpoint URL:").small().strong());
    ui.add(
        egui::TextEdit::singleline(&mut app.ui.ai_config.endpoint)
            .desired_width(f32::INFINITY)
            .hint_text("https://generativelanguage.googleapis.com/v1beta atau https://api.openai.com/v1"),
    );

    ui.add_space(4.0);

    // API Key
    ui.label(egui::RichText::new("API Key / Credential:").small().strong());
    ui.horizontal(|ui| {
        let edit = egui::TextEdit::singleline(&mut app.ui.ai_config.api_key)
            .password(!app.ai_chat.show_api_key)
            .desired_width(ui.available_width() - 85.0)
            .hint_text("Masukkan API Key Anda (disimpan aman di konfigurasi lokal ui.json)");
        ui.add(edit);
        let pw_label = if app.ai_chat.show_api_key { "Sembunyikan" } else { "Tampilkan" };
        if ui.button(pw_label).on_hover_text("Tampilkan/Sembunyikan API Key").clicked() {
            app.ai_chat.show_api_key = !app.ai_chat.show_api_key;
        }
    });
    if app.ui.ai_config.api_key.trim().is_empty() {
        ui.label(
            egui::RichText::new("Kunci API tidak di-hardcode. Masukkan kunci Anda di atas atau sediakan env GEMINI_API_KEY.")
                .small()
                .color(t.text_dim),
        );
    }

    ui.add_space(4.0);

    // Model Name
    ui.label(egui::RichText::new("Model:").small().strong());
    ui.add(
        egui::TextEdit::singleline(&mut app.ui.ai_config.model)
            .desired_width(f32::INFINITY)
            .hint_text("e.g. gemma-4-26b-a4b-it, gemini-3.8-flash, gpt-4o-mini, llama3.2"),
    );

    ui.add_space(4.0);

    // System Prompt
    ui.label(egui::RichText::new("Instruksi Sistem (System Prompt):").small().strong());
    ui.add(egui::TextEdit::multiline(&mut app.ui.ai_config.system_prompt).desired_rows(3).desired_width(f32::INFINITY));

    ui.add_space(8.0);

    // Test Connection & Save
    ui.horizontal(|ui| {
        let test_label = if app.ai_chat.testing_connection { "Menghubungi..." } else { "Tes Koneksi" };
        if ui.button(test_label).clicked() && !app.ai_chat.testing_connection {
            app.ui.ai_config.model = app.ui.ai_config.model.trim().to_ascii_lowercase();
            app.ai_chat.testing_connection = true;
            app.ai_chat.connection_status = None;
            let config = app.ui.ai_config.clone();
            if let Some(tx) = &app.ai_chat.tx_req {
                let _ = tx.send(AiTask::TestConnection { config });
            }
        }

        if ui.button("Simpan Pengaturan").clicked() {
            app.ui.ai_config.model = app.ui.ai_config.model.trim().to_ascii_lowercase();
            save_ai_config_to_prefs(app);
            app.ai_chat.connection_status = Some((true, "Pengaturan berhasil disimpan ke konfigurasi lokal!".into()));
        }
    });

    // Connection Status Feedback
    if let Some((ok, msg)) = &app.ai_chat.connection_status {
        ui.add_space(4.0);
        let color = if *ok { Color32::from_rgb(46, 139, 87) } else { Color32::from_rgb(220, 50, 50) };
        let icon = if *ok { "[OK]" } else { "[X]" };
        ui.label(egui::RichText::new(format!("{icon} {msg}")).small().color(color));
    }
}

fn save_ai_config_to_prefs(app: &WordApp) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let settings_dir = if cfg!(target_os = "macos") {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join("Library/Application Support/WordCraft"))
        } else if cfg!(windows) {
            std::env::var_os("APPDATA").map(|a| std::path::PathBuf::from(a).join("WordCraft"))
        } else {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
                .map(|c| c.join("wordcraft"))
        };

        if let Some(dir) = settings_dir {
            let p = dir.join("ui.json");
            let _ = std::fs::create_dir_all(&dir);
            if let Ok(bytes) = serde_json::to_vec_pretty(&app.prefs()) {
                let _ = std::fs::write(&p, bytes);
            }
        }
    }
}

// Background worker thread logic (curl-based execution)
#[cfg(not(target_arch = "wasm32"))]
fn ai_worker_loop(rx: Receiver<AiTask>, tx: Sender<AiResult>) {
    while let Ok(task) = rx.recv() {
        match task {
            AiTask::Chat { config, messages } => {
                let res = perform_ai_chat_request(&config, &messages);
                let msg = match res {
                    Ok(text) => AiResult::ChatSuccess { content: text, font_info: None },
                    Err(e) => AiResult::ChatError(e),
                };
                let _ = tx.send(msg);
            }
            AiTask::Suggest { config, prefix, context } => {
                let prompt = format!(
                    "Lanjutkan kalimat berikut dalam dokumen ini ({context}). Berikan 3 variasi saran kelanjutan kalimat berikutnya yang koheren, alami, dan bermutu tinggi dalam bahasa Indonesia. Tulis HANYA 3 pilihan bernomor 1., 2., 3.:\n\n\"{prefix}\""
                );
                let msgs = vec![
                    ("system".into(), "Anda adalah asisten autocomplete kalimat cerdas. Berikan 3 variasi kelanjutan kalimat berikutnya.".into()),
                    ("user".into(), prompt),
                ];
                let res = perform_ai_chat_request(&config, &msgs);
                let msg = match res {
                    Ok(text) => {
                        let parsed = parse_suggestion_lines(&text);
                        AiResult::SuggestSuccess(parsed)
                    }
                    Err(e) => AiResult::SuggestError(e),
                };
                let _ = tx.send(msg);
            }
            AiTask::TestConnection { config } => {
                let msgs = vec![("user".into(), "Katakan 'Koneksi Berhasil' dalam 2 kata.".into())];
                let res = perform_ai_chat_request(&config, &msgs);
                let msg = match res {
                    Ok(text) => AiResult::TestConnectionSuccess(format!(
                        "Terhubung ke {} ({})! Jawaban model: {}",
                        config.endpoint,
                        config.model,
                        text.trim()
                    )),
                    Err(e) => AiResult::TestConnectionError(e),
                };
                let _ = tx.send(msg);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn perform_ai_chat_request(config: &AiConfig, messages: &[(String, String)]) -> Result<String, String> {
    let is_google = config.provider == "google" || config.endpoint.contains("generativelanguage.googleapis.com");

    let (url, body_str, headers) = if is_google {
        let base = config.endpoint.trim().trim_end_matches('/');
        let raw_model = config.model.trim();
        let model = if raw_model.is_empty() { "gemma-4-26b-a4b-it".to_string() } else { raw_model.to_ascii_lowercase() };
        let key = if !config.api_key.trim().is_empty() {
            config.api_key.trim().to_string()
        } else {
            // Jika user belum mengisi di UI, periksa environment variable sebagai fallback
            std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY")).unwrap_or_default()
        };

        if key.is_empty() {
            return Err("API Key belum diisi. Silakan masukkan API Key Anda di tab Pengaturan (API) atau set environment variable GEMINI_API_KEY."
                .to_string());
        }

        let api_method = "streamGenerateContent";

        let mut base_url = if base.contains(":generateContent") || base.contains(":streamGenerateContent") {
            base.to_string()
        } else if base.contains("/models/") {
            format!("{base}:{api_method}")
        } else {
            format!("{base}/models/{model}:{api_method}")
        };

        if !base_url.contains("key=") {
            let sep = if base_url.contains('?') { '&' } else { '?' };
            base_url = format!("{base_url}{sep}key={key}");
        }

        let mut raw_contents = Vec::new();
        let mut sys_prompt = String::new();

        for (role, content) in messages {
            if role == "system" {
                if !sys_prompt.is_empty() {
                    sys_prompt.push_str("\n\n");
                }
                sys_prompt.push_str(content);
            } else {
                let google_role = if role == "assistant" || role == "model" { "model" } else { "user" };
                raw_contents.push((google_role, content.clone()));
            }
        }

        // Google Generative Language API rules:
        // 1. Turn sequence MUST start with "user" role (never start with "model").
        // 2. Turns must alternate between "user" and "model".
        // 3. Consecutive turns with the same role must be merged.
        let mut start_idx = 0;
        while start_idx < raw_contents.len() && raw_contents[start_idx].0 == "model" {
            start_idx += 1;
        }

        let mut alternating: Vec<(&str, String)> = Vec::new();
        for (role, text) in &raw_contents[start_idx..] {
            if text.trim().is_empty() {
                continue;
            }
            if let Some(last) = alternating.last_mut()
                && last.0 == *role
            {
                last.1.push_str("\n\n");
                last.1.push_str(text);
                continue;
            }
            alternating.push((role, text.clone()));
        }

        let mut final_contents = Vec::new();
        for (i, (role, text)) in alternating.into_iter().enumerate() {
            let content_text = if i == 0 && !sys_prompt.is_empty() { format!("{sys_prompt}\n\n{text}") } else { text };
            final_contents.push(json!({
                "role": role,
                "parts": [{ "text": content_text }]
            }));
        }

        if final_contents.is_empty() {
            let prompt = if !sys_prompt.is_empty() { sys_prompt } else { "Halo".to_string() };
            final_contents.push(json!({
                "role": "user",
                "parts": [{ "text": prompt }]
            }));
        }

        let body = json!({
            "contents": final_contents,
            "generationConfig": {
                "temperature": config.temperature,
                "maxOutputTokens": config.max_tokens,
            }
        });

        let body_str = serde_json::to_string(&body).map_err(|e| format!("Serialization error: {e}"))?;
        let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        if !key.is_empty() {
            headers.push(("x-goog-api-key".to_string(), key.to_string()));
        }

        (base_url, body_str, headers)
    } else {
        let mut url = config.endpoint.trim().trim_end_matches('/').to_string();
        if !url.ends_with("/chat/completions") && !url.ends_with("/messages") {
            url = format!("{url}/chat/completions");
        }

        let msgs_json: Vec<Value> = messages
            .iter()
            .map(|(role, content)| {
                json!({
                    "role": role,
                    "content": content
                })
            })
            .collect();

        let body = json!({
            "model": config.model.trim(),
            "messages": msgs_json,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
        });

        let body_str = serde_json::to_string(&body).map_err(|e| format!("Serialization error: {e}"))?;
        let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let user_key = config.api_key.trim();
        let key = if !user_key.is_empty() { user_key.to_string() } else { std::env::var("OPENAI_API_KEY").unwrap_or_default() };
        if !key.is_empty() {
            headers.push(("Authorization".to_string(), format!("Bearer {key}")));
        }

        (url, body_str, headers)
    };

    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-s").arg("-X").arg("POST").arg("--data-binary").arg("@-").arg(&url).arg("--max-time").arg("60");
    for (k, v) in &headers {
        cmd.arg("-H").arg(format!("{k}: {v}"));
    }

    cmd.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Gagal menjalankan curl: {e}. Pastikan curl terpasang."))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(body_str.as_bytes());
        let _ = stdin.flush();
        drop(stdin);
    }

    let output = child.wait_with_output().map_err(|e| format!("Gagal membaca output curl: {e}"))?;

    if !output.status.success() {
        let err_text = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl error (status {}): {err_text}", output.status));
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    let json_val: Value = serde_json::from_str(&out_str).map_err(|_| {
        if out_str.trim().is_empty() {
            "Server tidak memberikan jawaban (respons kosong). Periksa apakah server AI sedang berjalan.".to_string()
        } else {
            format!("Jawaban server bukan JSON yang valid:\n{out_str}")
        }
    })?;

    // Handle errors in root object or inside streaming array
    let items: &[Value] = if let Some(arr) = json_val.as_array() { arr.as_slice() } else { std::slice::from_ref(&json_val) };
    for item in items {
        if let Some(err_obj) = item.get("error") {
            let msg = err_obj.get("message").and_then(Value::as_str).unwrap_or("Unknown API error");
            return Err(format!("API Error: {msg}"));
        }
    }

    // Try Google Gemini / Gemma format: candidates[0].content.parts[0].text
    let mut answer_text = String::new();
    let mut thought_text = String::new();
    for item in items {
        if let Some(candidates) = item.get("candidates").and_then(Value::as_array) {
            for cand in candidates {
                if let Some(parts) = cand.get("content").and_then(|c| c.get("parts")).and_then(Value::as_array) {
                    for part in parts {
                        let is_thought = part.get("thought").and_then(Value::as_bool).unwrap_or(false);
                        if let Some(t) = part.get("text").and_then(Value::as_str) {
                            if is_thought {
                                thought_text.push_str(t);
                            } else {
                                answer_text.push_str(t);
                            }
                        }
                    }
                }
            }
        }
    }
    let chosen_text = if !answer_text.trim().is_empty() { answer_text.trim().to_string() } else { thought_text.trim().to_string() };
    if !chosen_text.is_empty() {
        return Ok(chosen_text);
    }

    // Try OpenAI format: choices[0].message.content
    if let Some(choices) = json_val.get("choices").and_then(Value::as_array)
        && let Some(first) = choices.first()
        && let Some(msg) = first.get("message")
        && let Some(content) = msg.get("content").and_then(Value::as_str)
    {
        return Ok(content.to_string());
    }

    // Try Anthropic format: content[0].text
    if let Some(content_arr) = json_val.get("content").and_then(Value::as_array)
        && let Some(first) = content_arr.first()
        && let Some(text) = first.get("text").and_then(Value::as_str)
    {
        return Ok(text.to_string());
    }

    // Try Ollama generate format: response
    if let Some(resp) = json_val.get("response").and_then(Value::as_str) {
        return Ok(resp.to_string());
    }

    Err(format!("Format respons tidak dikenali:\n{out_str}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_suggestion_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Clean leading numbering like "1. ", "1) ", "- ", "* "
        let clean = trimmed.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')' || c == '-' || c == '*' || c == ' ').trim();
        let clean = clean.trim_matches('"');
        if !clean.is_empty() {
            out.push(clean.to_string());
        }
        if out.len() >= 3 {
            break;
        }
    }

    if out.is_empty() {
        out.push(text.trim().to_string());
    }
    out
}
