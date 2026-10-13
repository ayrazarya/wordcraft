//! Side panes: Navigation (headings, pages, search results), Clipboard, Styles, Style Inspector,
//! Accessibility and Alt Text (see `a11y_pane`), Restrict Editing ([`crate::dialogs_review`]), Comments,
//! and the SmartArt Text Pane.

use egui::{Stroke, Ui, vec2};
use serde_json::{Value, json};
use wordcraft_doc::{Pos, StoryRef};

use crate::WordApp;
use crate::theme::{Tokens, regular, semibold};

pub fn show(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    // SmartArt Design › Text Pane, while a SmartArt graphic WordCraft can edit is selected.
    if app.session.view.smart_art_pane && wordcraft_engine::cmd::smart_art::selected_smart_art(&app.session).is_some() {
        egui::Panel::left("smart_art_pane")
            .default_size(250.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| {
                if header(ui, "Text Pane") {
                    let _ = app.run("smartArt.textPane", json!({"value": false}));
                    return;
                }
                crate::dialogs_smart_art::text_pane(app, ui);
            });
    }
    if app.session.view.nav_pane {
        egui::Panel::left("nav_pane")
            .default_size(260.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| nav(app, ui));
    }
    if app.session.view.clipboard_pane {
        egui::Panel::left("clipboard_pane")
            .default_size(250.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| clipboard(app, ui));
    }
    if app.session.view.styles_pane {
        egui::Panel::right("styles_pane")
            .default_size(250.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| styles(app, ui));
    }
    if app.session.view.style_inspector {
        egui::Panel::right("style_inspector")
            .default_size(240.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| inspector(app, ui));
    }
    if app.session.view.accessibility_pane {
        egui::Panel::right("accessibility_pane")
            .default_size(280.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| crate::a11y_pane::accessibility(app, ui));
    }
    if app.session.view.alt_text_pane {
        egui::Panel::right("alt_text_pane")
            .default_size(260.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| crate::a11y_pane::alt_text(app, ui));
    }
    if app.restrict.open {
        egui::Panel::right("restrict_pane")
            .default_size(270.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| {
                if header(ui, "Restrict Editing") {
                    app.restrict.open = false;
                    return;
                }
                crate::dialogs_review::restrict_pane(app, ui);
            });
    }
    if app.session.view.comments_pane {
        egui::Panel::right("comments_pane")
            .default_size(290.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| comments(app, ui));
    }
    if app.session.view.ai_pane {
        egui::Panel::right("ai_pane")
            .default_size(340.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| {
                if header(ui, "AI Assistant") {
                    let _ = app.run("view.aiPane", json!({"value": false}));
                    return;
                }
                crate::ai_chat::show(app, ui);
            });
    }
    if app.session.view.version_pane {
        egui::Panel::right("version_pane")
            .default_size(340.0)
            .resizable(true)
            .frame(egui::Frame::NONE.fill(t.panel).inner_margin(10).stroke(Stroke::new(1.0, t.border)))
            .show(ui, |ui| {
                if header(ui, "Version History") {
                    let _ = app.run("view.versionHistory", json!({"value": false}));
                    return;
                }
                version_history(app, ui);
            });
    }
}

pub(crate) fn header(ui: &mut Ui, title: &str) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!(title)).font(semibold(15.0)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Drawn, not the "✕" character: the interface fonts have no glyph for it.
            if crate::widgets::icon_button(ui, "close", tl!("Close")).clicked() {
                close = true;
            }
        });
    });
    ui.add_space(4.0);
    close
}

fn nav(app: &mut WordApp, ui: &mut Ui) {
    if header(ui, "Navigation") {
        let _ = app.run("view.navigationPane", json!({"value": false}));
        return;
    }
    let qid = egui::Id::new("nav_query");
    let mut q = ui.data(|d| d.get_temp::<String>(qid)).unwrap_or_default();
    // Ctrl+F with text selected searches for it, as Word does (a selection within one line).
    let focus = std::mem::take(&mut app.nav_search_focus);
    let picked = focus.then(|| app.session.selected_text()).filter(|t| !t.is_empty() && !t.contains(['\n', '\r']) && t.chars().count() <= 255);
    if let Some(t) = &picked {
        q = t.clone();
    }
    let mut out = egui::TextEdit::singleline(&mut q).hint_text(tl!("Search document")).desired_width(f32::INFINITY).show(ui);
    let r = out.response.clone();
    if focus {
        // Focused with its text selected, so typing replaces it.
        r.request_focus();
        let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(q.chars().count()));
        out.state.cursor.set_char_range(Some(all));
        out.state.store(ui.ctx(), r.id);
    }
    if r.changed() || picked.is_some() {
        ui.data_mut(|d| d.insert_temp(qid, q.clone()));
        if !q.is_empty() {
            let _ = app.session.run("edit.find", &json!({"text": q}));
            app.ui.nav_tab = "results".into();
        } else {
            app.session.find.results.clear();
        }
    }
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for (k, l) in [("headings", "Headings"), ("pages", "Pages"), ("results", "Results")] {
            if ui.selectable_label(app.ui.nav_tab == k, tl!(l)).clicked() {
                app.ui.nav_tab = k.into();
            }
        }
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| match app.ui.nav_tab.as_str() {
        "pages" => {
            let l = app.session.layout();
            for (i, p) in l.pages.iter().enumerate() {
                let label = crate::i18n::fmt(tl!("Page {page}"), &[("page", &p.number.to_string())]);
                if ui.selectable_label(app.session.page_hint == i, label).clicked() {
                    let _ = app.run("edit.goto", json!({"page": i + 1}));
                }
            }
        }
        "results" => {
            let results = app.session.find.results.clone();
            let count_text = if results.len() == 1 {
                "1 result found".to_string()
            } else {
                format!("{} results found", results.len())
            };
            ui.label(egui::RichText::new(count_text).small().weak());
            for (a, b) in results.iter().take(500) {
                let Some(p) = app.session.doc.para_at(a) else { continue };
                let from = p.clamp(a.off.saturating_sub(30));
                let to = p.clamp((b.off + 40).min(p.len()));
                let pre = p.text.get(from..a.off).unwrap_or("");
                let hit = p.text.get(a.off..b.off).unwrap_or("");
                let post = p.text.get(b.off..to).unwrap_or("");
                let mut job = egui::text::LayoutJob::default();
                let fmt = |c| egui::TextFormat { font_id: regular(12.0), color: c, ..Default::default() };
                job.append(pre, 0.0, fmt(ui.visuals().text_color()));
                // The match is marked the way Find marks it on the page, per theme (#656).
                let mark = Tokens::get(ui.ctx()).find_highlight;
                job.append(hit, 0.0, egui::TextFormat { font_id: semibold(12.0), background: mark, color: ui.visuals().text_color(), ..Default::default() });
                job.append(post, 0.0, fmt(ui.visuals().text_color()));
                job.wrap.max_width = ui.available_width();
                if ui.add(egui::Button::new(job).frame(false)).clicked() {
                    let _ = app.run("select.range", json!({"anchor": a, "focus": b}));
                }
                ui.separator();
            }
        }
        _ => {
            let caret = app.session.sel.focus.clone();
            let mut current: Option<usize> = None;
            let mut items = Vec::new();
            for path in app.session.doc.para_paths(StoryRef::Body) {
                let Some(p) = app.session.doc.para(StoryRef::Body, &path) else { continue };
                let rp = app.session.doc.styles.resolve_para(&p.props);
                if let Some(lv) = rp.outline_level {
                    let txt = p.plain_text().trim().to_string();
                    if txt.is_empty() || rp.style.starts_with("TOC") {
                        continue;
                    }
                    if path <= caret.path {
                        current = Some(items.len());
                    }
                    items.push((path, lv, txt));
                }
            }
            if items.is_empty() {
                ui.label(egui::RichText::new(tl!("Create an interactive outline of your document.\n\nIt's a great way to keep track of where you are or quickly move your content around.\n\nTo get started, go to the Home tab and apply Heading styles to the headings in your document.")).weak());
            }
            for (i, (path, lv, txt)) in items.into_iter().enumerate() {
                if nav_heading(ui, lv, &txt, current == Some(i)).clicked() {
                    let _ = app.run("caret.set", json!({"pos": Pos { story: StoryRef::Body, path, off: 0 }}));
                    // Align the chosen heading's page immediately against this pane, rather
                    // than the indeterminate minimum movement from scroll_to_rect.
                    app.canvas.nav_to_caret = true;
                    app.canvas.scroll_to_caret = false;
                    app.canvas.want_focus = true;
                }
            }
        }
    });
}

/// Constrain each heading to the available pane width so long document text cannot
/// enlarge the panel. The full heading stays accessible on hover.
fn nav_heading(ui: &mut Ui, level: u8, text: &str, selected: bool) -> egui::Response {
    ui.horizontal(|ui| {
        ui.add_space(level as f32 * 14.0);
        ui.add_sized(
            vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Button::selectable(selected, egui::RichText::new(text).font(if level == 0 { semibold(12.5) } else { regular(12.5) })).truncate(),
        )
        .on_hover_text(text)
    })
    .inner
}

/// Items collected by Copy and Cut, newest first; clicking one pastes it at the caret.
fn clipboard(app: &mut WordApp, ui: &mut Ui) {
    if header(ui, "Clipboard") {
        let _ = app.run("edit.clipboardPane", json!({"value": false}));
        return;
    }
    let items: Vec<String> = app.session.clip_history.items().iter().map(|i| i.preview()).collect();
    ui.horizontal(|ui| {
        if ui.add_enabled(!items.is_empty(), egui::Button::new(tl!("Paste All"))).clicked() {
            let _ = app.run("edit.pasteAllClipboard", json!({}));
            app.canvas.want_focus = true;
        }
        if ui.add_enabled(!items.is_empty(), egui::Button::new(tl!("Clear All"))).clicked() {
            let _ = app.run("edit.clearClipboard", json!({}));
        }
    });
    ui.add_space(4.0);
    let hint = if items.is_empty() { "Nothing collected yet. Items you copy or cut appear here." } else { "Click an item to paste it." };
    ui.label(egui::RichText::new(tl!(hint)).small().weak());
    ui.separator();
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, preview) in items.iter().enumerate() {
            egui::Frame::NONE.fill(t.input).stroke(Stroke::new(1.0, t.border)).corner_radius(6).inner_margin(8).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let text = if preview.is_empty() { tl!("(picture or object)").to_string() } else { preview.clone() };
                let mut job =
                    egui::text::LayoutJob::single_section(text, egui::TextFormat { font_id: regular(12.5), color: t.text, ..Default::default() });
                job.wrap.max_width = ui.available_width();
                job.wrap.max_rows = 3;
                if ui.add(egui::Button::new(job).frame(false)).on_hover_text(tl!("Paste")).clicked() {
                    let _ = app.run("edit.pasteClipboardItem", json!({"index": i}));
                    app.canvas.want_focus = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.small_button(tl!("Delete")).clicked() {
                        let _ = app.run("edit.deleteClipboardItem", json!({"index": i}));
                    }
                });
            });
            ui.add_space(6.0);
        }
    });
}

fn styles(app: &mut WordApp, ui: &mut Ui) {
    if header(ui, "Styles") {
        let _ = app.run("view.stylesPane", json!({"value": false}));
        return;
    }
    let st = app.session.run("format.state", &json!({})).unwrap_or_default();
    let cur = st.get("style").and_then(Value::as_str).unwrap_or("Normal").to_string();
    let current_name = app
        .session
        .doc
        .styles
        .get(&cur)
        .map(crate::i18n::style_name)
        .unwrap_or_else(|| st.get("styleName").and_then(Value::as_str).unwrap_or("").to_string());
    ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Current style: {style}"), &[("style", &current_name)])).small());
    ui.horizontal(|ui| {
        if ui.button(tl!("New Style…")).clicked() {
            app.dialog = crate::dialogs::Dialog::open("newStyle", app);
        }
        if ui.button(tl!("Update to Match")).clicked() {
            let _ = app.run("styles.updateToMatch", json!({}));
        }
        if ui.button(tl!("Clear")).clicked() {
            let _ = app.run("format.clear", json!({}));
        }
    });
    if ui.button(tl!("Manage Styles…")).clicked() {
        let _ = app.run("styles.manage", json!({}));
    }
    if ui.selectable_label(app.session.view.style_inspector, tl!("Style Inspector")).clicked() {
        let _ = app.run("styles.inspector", json!({}));
    }
    ui.separator();
    let mut list: Vec<(String, String, bool)> = app
        .session
        .doc
        .styles
        .styles
        .iter()
        .filter(|s| !s.hidden && s.kind != wordcraft_doc::StyleKind::Table)
        .map(|s| (s.id.clone(), crate::i18n::style_name(s), s.kind == wordcraft_doc::StyleKind::Character))
        .collect();
    list.sort_by_key(|s| s.1.to_lowercase());
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (id, name, is_char) in list {
            ui.horizontal(|ui| {
                let r = ui.selectable_label(id == cur, &name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(if is_char { "a" } else { "¶" }).weak());
                });
                if r.clicked() {
                    let _ = app.run("para.style", json!({"style": id}));
                    app.canvas.want_focus = true;
                }
                r.context_menu(|ui| {
                    if ui.button(tl!("Modify…")).clicked() {
                        app.dialog = crate::dialogs::Dialog::modify_style(app, &id);
                        ui.close();
                    }
                    if ui.button(tl!("Update to Match Selection")).clicked() {
                        let _ = app.run("styles.updateToMatch", json!({"style": id}));
                        ui.close();
                    }
                    if ui.button(tl!("Delete")).clicked() {
                        let _ = app.run("styles.delete", json!({"style": id}));
                        ui.close();
                    }
                });
            });
        }
    });
}

fn comments(app: &mut WordApp, ui: &mut Ui) {
    if header(ui, "Comments") {
        let _ = app.run("view.commentsPane", json!({"value": false}));
        return;
    }
    if crate::widgets::icon_text_button(ui, "newComment", tl!("New Comment"), regular(12.5)).clicked() {
        let _ = app.run("review.newComment", json!({"text": ""}));
    }
    ui.separator();
    let list = app.session.run("review.comments", &json!({})).unwrap_or_default();
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().show(ui, |ui| {
        for c in list.as_array().cloned().unwrap_or_default() {
            let id = c.get("id").and_then(Value::as_u64).unwrap_or(0);
            let reply = c.get("parent").is_some_and(|p| !p.is_null());
            egui::Frame::NONE.fill(t.input).stroke(Stroke::new(1.0, t.border)).corner_radius(6).inner_margin(8).show(ui, |ui| {
                ui.set_width(ui.available_width());
                if reply {
                    ui.add_space(0.0);
                }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(c.get("author").and_then(Value::as_str).unwrap_or("")).font(semibold(12.0)));
                    ui.label(egui::RichText::new(comment_stamp(c.get("date").and_then(Value::as_str).unwrap_or(""))).small().weak());
                    if c.get("resolved").and_then(Value::as_bool).unwrap_or(false) {
                        ui.label(egui::RichText::new(tl!("Resolved")).small().color(t.green));
                    }
                });
                // Editable comment text (one undo step per edit, like the balloons).
                comment_editor(app, ui, id as u32, None, false);
                let open_key = egui::Id::new(("comment_reply_open", id));
                let mut replying = ui.data(|d| d.get_temp::<bool>(open_key)).unwrap_or(false);
                ui.horizontal(|ui| {
                    if ui.small_button(tl!("Go to")).clicked()
                        && let Some(a) = c.get("anchor").filter(|a| !a.is_null())
                    {
                        let _ = app.run("caret.set", json!({"pos": a}));
                    }
                    if ui.small_button(tl!("Reply")).clicked() {
                        replying = true;
                        ui.data_mut(|d| d.insert_temp(open_key, true));
                        ui.data_mut(|d| d.insert_temp(egui::Id::new(("comment_reply_focus", id)), true));
                    }
                    let resolved = c.get("resolved").and_then(Value::as_bool).unwrap_or(false);
                    if ui.small_button(if resolved { tl!("Reopen") } else { tl!("Resolve") }).clicked() {
                        let _ = app.run("review.resolveComment", json!({"id": id}));
                    }
                    if ui.small_button(tl!("Delete")).clicked() {
                        let _ = app.run("review.deleteComment", json!({"id": id}));
                    }
                });
                if replying {
                    let focus = ui.data_mut(|d| d.remove_temp::<bool>(egui::Id::new(("comment_reply_focus", id)))).unwrap_or(false);
                    if reply_editor(app, ui, id as u32, None, focus) {
                        ui.data_mut(|d| d.remove::<bool>(open_key));
                    }
                }
            });
            ui.add_space(6.0);
        }
    });
}

fn edit_key(id: u32) -> egui::Id {
    egui::Id::new(("comment_edit", id))
}

fn reply_key(id: u32) -> egui::Id {
    egui::Id::new(("comment_reply", id))
}

/// A comment's text, editable in place (the Comments pane and the balloons). Typing stays in a
/// draft until the field loses focus; then [`commit_comment`] writes it back with one
/// `review.editComment`, so each editing session is one undo step.
pub(crate) fn comment_editor(app: &mut WordApp, ui: &mut Ui, id: u32, font: Option<egui::FontId>, focus: bool) {
    let Some(part) = app.session.doc.comments.get(&id).map(|c| c.part) else { return };
    let key = edit_key(id);
    let mut text = ui.data(|d| d.get_temp::<String>(key)).unwrap_or_else(|| app.session.doc.plain_text(StoryRef::Part(part)));
    let mut edit = egui::TextEdit::multiline(&mut text).desired_rows(1).desired_width(f32::INFINITY).hint_text(tl!("Add a comment"));
    if let Some(f) = font {
        edit = edit.font(f);
    }
    let r = ui.add(edit);
    if focus {
        r.request_focus();
    }
    if r.changed() {
        ui.data_mut(|d| d.insert_temp(key, text));
    }
    if r.lost_focus() {
        commit_comment(app, ui.ctx(), id);
    }
}

/// Write a comment's draft text back, when it has one that differs from the comment.
pub(crate) fn commit_comment(app: &mut WordApp, ctx: &egui::Context, id: u32) {
    let Some(text) = ctx.data_mut(|d| d.remove_temp::<String>(edit_key(id))) else { return };
    let Some(part) = app.session.doc.comments.get(&id).map(|c| c.part) else { return };
    if text != app.session.doc.plain_text(StoryRef::Part(part)) {
        let _ = app.run("review.editComment", json!({"id": id, "text": text}));
    }
}

/// The comment a reply to `id` answers: replies to a reply join its thread.
fn thread_root(app: &WordApp, id: u32) -> u32 {
    match app.session.doc.comments.get(&id).and_then(|c| c.parent) {
        Some(p) if app.session.doc.comments.contains_key(&p) => p,
        _ => id,
    }
}

/// A field for replying to a comment. When it loses focus a non-empty reply is posted with
/// `review.reply`; returns true when the field is done (posted or left empty).
pub(crate) fn reply_editor(app: &mut WordApp, ui: &mut Ui, id: u32, font: Option<egui::FontId>, focus: bool) -> bool {
    let key = reply_key(id);
    let mut text = ui.data(|d| d.get_temp::<String>(key)).unwrap_or_default();
    let mut edit = egui::TextEdit::multiline(&mut text).desired_rows(1).desired_width(f32::INFINITY).hint_text(tl!("Write a reply"));
    if let Some(f) = font {
        edit = edit.font(f);
    }
    let r = ui.add(edit);
    if focus {
        r.request_focus();
    }
    if r.changed() {
        ui.data_mut(|d| d.insert_temp(key, text));
    }
    if r.lost_focus() {
        commit_reply(app, ui.ctx(), id);
        return true;
    }
    false
}

/// Post a pending reply draft to comment `id` (nothing when there is none or it is blank).
pub(crate) fn commit_reply(app: &mut WordApp, ctx: &egui::Context, id: u32) {
    let Some(text) = ctx.data_mut(|d| d.remove_temp::<String>(reply_key(id))) else { return };
    if !text.trim().is_empty() {
        let root = thread_root(app, id);
        let _ = app.run("review.reply", json!({"id": root, "text": text.trim_end()}));
    }
}

/// Style Inspector: the paragraph and character levels at the caret, each with its style and the
/// direct formatting on top, and a button to clear each level.
fn inspector(app: &mut WordApp, ui: &mut Ui) {
    if header(ui, "Style Inspector") {
        let _ = app.run("styles.inspector", json!({"value": false}));
        return;
    }
    // Read-only, so call it directly: running the command every frame would pay for the command
    // machinery and consume per-command state (a pending `join_next_undo`) just to draw the pane.
    let r = wordcraft_engine::cmd::inspector::inspect(&app.session);
    let t = Tokens::get(ui.ctx());
    let none = tl!("None").to_string();
    let para = &r["paragraph"];
    let chr = &r["character"];
    let pstyle = para["styleName"].as_str().unwrap_or("Normal").to_string();
    let cstyle = chr["styleName"].as_str().map(str::to_string);
    // Built-in style names show in the interface language; the document keeps the English ones.
    let shown = |id: &Value, name: &str| {
        id.as_str().and_then(|id| app.session.doc.styles.get(id)).map(crate::i18n::style_name).unwrap_or_else(|| name.to_string())
    };
    let pstyle_shown = shown(&para["style"], &pstyle);
    let cstyle_shown = cstyle.as_deref().map(|n| shown(&chr["style"], n));
    let pdirect = direct_list(&para["direct"]);
    let cdirect = direct_list(&chr["direct"]);
    let mut clear: Option<&str> = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        section(ui, &t, "Paragraph", |ui| {
            if level_row(ui, &t, "Style", &pstyle_shown, false, "Reset to Normal", pstyle == "Normal" && para["style"] == "Normal") {
                clear = Some("paragraphStyle");
            }
            let (txt, empty) = if pdirect.is_empty() { (none.clone(), true) } else { (pdirect.join(", "), false) };
            if level_row(ui, &t, "Direct formatting", &txt, empty, "Clear Paragraph Formatting", empty) {
                clear = Some("paragraphFormatting");
            }
        });
        ui.add_space(8.0);
        section(ui, &t, "Characters", |ui| {
            let (txt, empty) = match &cstyle_shown {
                Some(n) => (n.clone(), false),
                None => (none.clone(), true),
            };
            if level_row(ui, &t, "Style", &txt, empty, "Clear Character Style", empty) {
                clear = Some("characterStyle");
            }
            let (txt, empty) = if cdirect.is_empty() { (none.clone(), true) } else { (cdirect.join(", "), false) };
            if level_row(ui, &t, "Direct formatting", &txt, empty, "Clear Character Formatting", empty) {
                clear = Some("characterFormatting");
            }
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(tl!("Clear All")).on_hover_text(tl!("Clear All Formatting")).clicked() {
                let _ = app.run("format.clear", json!({}));
                app.canvas.want_focus = true;
            }
            if ui.button(tl!("Styles Pane")).clicked() {
                let _ = app.run("view.stylesPane", json!({"value": true}));
            }
        });
    });
    if let Some(level) = clear {
        let _ = app.run("styles.inspectorClear", json!({"level": level}));
        app.canvas.want_focus = true;
    }
}

/// A titled, bordered group in the inspector.
fn section(ui: &mut Ui, t: &Tokens, title: &str, body: impl FnOnce(&mut Ui)) {
    ui.label(egui::RichText::new(tl!(title)).font(semibold(12.5)).color(t.text));
    ui.add_space(2.0);
    egui::Frame::NONE.fill(t.input).stroke(Stroke::new(1.0, t.border)).corner_radius(4).inner_margin(8).show(ui, |ui| {
        ui.set_width(ui.available_width());
        body(ui);
    });
}

/// One level: a caption, its value, and a clear button (disabled when there's nothing to clear).
/// Returns true when the button was clicked.
fn level_row(ui: &mut Ui, t: &Tokens, caption: &str, value: &str, dim: bool, tip: &str, disabled: bool) -> bool {
    ui.label(egui::RichText::new(tl!(caption)).font(regular(11.0)).color(t.text_dim));
    let mut clicked = false;
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 26.0).max(40.0);
        ui.allocate_ui(vec2(w, 18.0), |ui| {
            ui.set_width(w);
            let text = egui::RichText::new(value).font(regular(12.5)).color(if dim { t.text_disabled } else { t.text });
            ui.add(egui::Label::new(text).wrap());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            let (r, resp) = ui.allocate_exact_size(vec2(22.0, 20.0), egui::Sense::click());
            let enabled = !disabled;
            if enabled && resp.hovered() {
                ui.painter().rect_filled(r, 3.0, t.hover);
            }
            let c = if enabled { t.icon } else { t.text_disabled };
            crate::icons::paint(
                ui.painter(),
                egui::Rect::from_center_size(r.center(), vec2(16.0, 16.0)),
                "clear",
                c,
                if enabled { t.accent } else { c },
            );
            let resp = resp.on_hover_text(tl!(tip));
            clicked = enabled && resp.clicked();
        });
    });
    ui.add_space(4.0);
    clicked
}

/// Readable descriptions of the inspector's `direct` entries (`[{"prop", "value"}]`).
fn direct_list(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().take(64).map(|d| describe(d["prop"].as_str().unwrap_or(""), &d["value"])).collect()).unwrap_or_default()
}

/// Each `{"prop", "value"}` of a list (the Style Inspector's, or a formatting change's) as text.
pub(crate) fn describe_props(list: &[Value]) -> Vec<String> {
    list.iter().take(64).map(|d| describe(d["prop"].as_str().unwrap_or(""), &d["value"])).collect()
}

/// "Bold", "Not Italic", "Font: Arial", "Size: 14 pt"…
fn describe(prop: &str, v: &Value) -> String {
    let label = match prop {
        "font" => "Font",
        "size" => "Size",
        "bold" => "Bold",
        "italic" => "Italic",
        "underline" => "Underline",
        "underlineColor" => "Underline color",
        "strike" => "Strikethrough",
        "doubleStrike" => "Double strikethrough",
        "color" => "Font color",
        "highlight" => "Highlight",
        "shading" => "Shading",
        "vertAlign" => "Position",
        "caps" => "All caps",
        "smallCaps" => "Small caps",
        "hidden" => "Hidden",
        "spacing" => "Character spacing",
        "scale" => "Scale",
        "position" => "Raised/lowered",
        "outline" => "Outline",
        "shadow" => "Shadow",
        "emboss" => "Emboss",
        "engrave" => "Engrave",
        "align" => "Alignment",
        "indentLeft" => "Left indent",
        "indentRight" => "Right indent",
        "indentFirst" => "First line",
        "spaceBefore" => "Space before",
        "spaceAfter" => "Space after",
        "lineSpacing" => "Line spacing",
        "contextualSpacing" => "Don't add space between paragraphs of the same style",
        "mirrorIndents" => "Mirror indents",
        "keepNext" => "Keep with next",
        "keepLines" => "Keep lines together",
        "pageBreakBefore" => "Page break before",
        "widowControl" => "Widow/Orphan control",
        "outlineLevel" => "Outline level",
        "tabs" => "Tabs",
        "borders" => "Borders",
        "suppressHyphens" => "Don't hyphenate",
        "suppressLineNumbers" => "Suppress line numbers",
        "bidi" => "Right-to-left",
        "dropCap" => "Drop cap",
        "style" => "Style",
        "numbering" => "Numbering",
        "lang" => "Language",
        other => other,
    };
    let label = tl!(label);
    let pt = |x: f64| crate::i18n::fmt(tl!("{n} pt"), &[("n", &trim_num(x))]);
    let hex = |c: &Value| -> Option<String> {
        let a = c.as_array()?;
        let ch = |i: usize| a.get(i).and_then(Value::as_u64).unwrap_or(0).min(255);
        Some(format!("#{:02X}{:02X}{:02X}", ch(0), ch(1), ch(2)))
    };
    let value = match (prop, v) {
        (_, Value::Bool(true)) => return label.to_string(),
        (_, Value::Bool(false)) => return crate::i18n::fmt(tl!("Not {name}"), &[("name", label)]),
        ("scale", Value::Number(n)) => format!("{}%", trim_num(n.as_f64().unwrap_or(100.0))),
        ("outlineLevel", Value::Number(n)) => match n.as_u64().unwrap_or(9) {
            l @ 0..=8 => crate::i18n::fmt(tl!("Level {n}"), &[("n", &(l + 1).to_string())]),
            _ => tl!("Body Text").to_string(),
        },
        ("dropCap", Value::Number(n)) => crate::i18n::fmt(tl!("{n} lines"), &[("n", &n.to_string())]),
        (_, Value::Number(n)) => pt(n.as_f64().unwrap_or(0.0)),
        ("align", Value::String(s)) => tl!(match s.as_str() {
            "center" => "Centered",
            "right" => "Right",
            "justify" => "Justified",
            "distribute" => "Distributed",
            _ => "Left",
        })
        .to_string(),
        ("color", Value::String(_)) => tl!("Automatic").to_string(),
        ("color", Value::Object(o)) => o.get("Rgb").and_then(hex).unwrap_or_default(),
        (_, Value::Array(_)) if prop != "tabs" => hex(v).unwrap_or_default(),
        ("lineSpacing", Value::Object(o)) => {
            let x = o.get("value").and_then(Value::as_f64).unwrap_or(1.0);
            match o.get("rule").and_then(Value::as_str) {
                Some("atLeast") => crate::i18n::fmt(tl!("At least {n}"), &[("n", &pt(x))]),
                Some("exactly") => crate::i18n::fmt(tl!("Exactly {n}"), &[("n", &pt(x))]),
                _ => crate::i18n::fmt(tl!("{n} lines"), &[("n", &trim_num(x))]),
            }
        }
        (_, Value::String(s)) => tl!(&camel_words(s)).to_string(),
        _ => return label.to_string(),
    };
    format!("{label}: {value}")
}

/// `1.50` → `1.5`, `12.0` → `12`.
fn trim_num(x: f64) -> String {
    let s = format!("{:.2}", if x.is_finite() { x } else { 0.0 });
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// `2026-10-12T04:44:00Z` → `2026-10-12 04:44` (date and time, no seconds) for comment cards.
fn comment_stamp(iso: &str) -> String {
    iso.chars().take(16).map(|c| if c == 'T' { ' ' } else { c }).collect()
}

/// `dotDash` → `Dot dash` (enum values from the document model).
fn camel_words(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().take(64).enumerate() {
        if i == 0 {
            out.extend(c.to_uppercase());
        } else if c.is_uppercase() {
            out.push(' ');
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ─── Version History pane ──────────────────────────────────────────────────

/// The Version History side pane: a Git-like offline version control UI.
fn version_history(app: &mut WordApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());

    // ── Branch selector ──
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!("Branch:")).font(semibold(12.0)));
        let branch_id = egui::Id::new("version_branch_combo");
        let branches_result = app.session.run("file.versions", &json!({"listBranches": true}));
        let current = branches_result.as_ref().ok().and_then(|v| v.get("current")).and_then(Value::as_str).unwrap_or("main").to_string();
        let branches: Vec<String> = branches_result
            .ok()
            .and_then(|v| v.get("branches").and_then(Value::as_array).cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect();
        let mut selected = current.clone();
        egui::ComboBox::from_id_salt(branch_id).selected_text(&selected).width(ui.available_width() - 80.0).show_ui(ui, |ui| {
            for b in &branches {
                if ui.selectable_label(*b == selected, b).clicked() {
                    selected = b.clone();
                }
            }
        });
        if selected != current {
            let _ = app.run("file.versions", json!({"branch": selected}));
        }
    });

    // ── New branch button ──
    let new_branch_id = egui::Id::new("version_new_branch");
    let mut new_branch_open = ui.data(|d| d.get_temp::<bool>(new_branch_id)).unwrap_or(false);
    ui.horizontal(|ui| {
        if ui.small_button(tl!("+ New Branch")).clicked() {
            new_branch_open = !new_branch_open;
            ui.data_mut(|d| d.insert_temp(new_branch_id, new_branch_open));
        }
    });
    if new_branch_open {
        let input_id = egui::Id::new("version_new_branch_input");
        let mut name = ui.data(|d| d.get_temp::<String>(input_id)).unwrap_or_default();
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text(tl!("Branch name")).desired_width(ui.available_width() - 60.0));
            if r.changed() {
                ui.data_mut(|d| d.insert_temp(input_id, name.clone()));
            }
            if ui.small_button("✓").clicked() && !name.trim().is_empty() {
                let _ = app.run("file.versions", json!({"branch": name.trim()}));
                ui.data_mut(|d| d.remove::<String>(input_id));
                ui.data_mut(|d| d.insert_temp(new_branch_id, false));
            }
        });
    }

    ui.separator();

    // ── Commit form ──
    ui.label(egui::RichText::new(tl!("Create Version")).font(semibold(13.0)));
    let detected_info = app.session.run("file.versions", &json!({"detectUser": true})).unwrap_or_default();
    let display_author = detected_info.get("resolvedAuthor").and_then(Value::as_str).unwrap_or("WordCraft User");
    let is_os_detected = detected_info.get("systemUser").and_then(Value::as_str).is_some_and(|u| !u.is_empty() && display_author == u);
    ui.horizontal(|ui| {
        let tag = if is_os_detected { " [offline OS user]" } else { "" };
        ui.label(egui::RichText::new(format!("Author: {display_author}{tag}")).font(regular(11.0)).color(t.text_dim));
    });

    let label_id = egui::Id::new("version_commit_label");
    let msg_id = egui::Id::new("version_commit_msg");
    let mut label_text = ui.data(|d| d.get_temp::<String>(label_id)).unwrap_or_default();
    let mut msg_text = ui.data(|d| d.get_temp::<String>(msg_id)).unwrap_or_default();

    ui.add(egui::TextEdit::singleline(&mut label_text).hint_text(tl!("Version name (e.g. Draft 1)")).desired_width(f32::INFINITY));
    ui.data_mut(|d| d.insert_temp(label_id, label_text.clone()));
    ui.add(egui::TextEdit::multiline(&mut msg_text).hint_text(tl!("Description (optional)")).desired_rows(2).desired_width(f32::INFINITY));
    ui.data_mut(|d| d.insert_temp(msg_id, msg_text.clone()));

    ui.horizontal(|ui| {
        let can_commit = !label_text.trim().is_empty();
        if ui.add_enabled(can_commit, egui::Button::new(egui::RichText::new(tl!("Commit")).font(semibold(12.0)))).clicked() {
            let _ = app.run("file.versions", json!({"save": label_text.trim(), "message": msg_text.trim()}));
            ui.data_mut(|d| d.remove::<String>(label_id));
            ui.data_mut(|d| d.remove::<String>(msg_id));
        }
        if ui.small_button(tl!("Auto-save")).on_hover_text(tl!("Create an automatic checkpoint of the current state")).clicked() {
            let _ = app.run("file.versions", json!({"auto": true}));
        }
    });

    ui.separator();

    // ── Version timeline ──
    let versions = app.session.run("file.versions", &json!({})).unwrap_or_default();
    let list = versions.as_array().cloned().unwrap_or_default();

    let count_label = if list.len() == 1 { "1 version".to_string() } else { format!("{} versions", list.len()) };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tl!("Timeline")).font(semibold(14.0)));
        ui.label(egui::RichText::new(count_label).small().weak());
    });

    if list.is_empty() {
        ui.add_space(12.0);
        ui.label(egui::RichText::new(tl!("No versions yet.\n\nCreate your first version by entering a name above and clicking Commit, or enable auto-save checkpoints.\n\nVersions are stored locally on your machine — no internet needed.")).weak());
        return;
    }

    ui.add_space(4.0);

    // Show versions newest first.
    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in list.iter().rev() {
            let idx = entry.get("index").and_then(Value::as_u64).unwrap_or(0);
            let label = entry.get("label").and_then(Value::as_str).unwrap_or("");
            let message = entry.get("message").and_then(Value::as_str).unwrap_or("");
            let date = entry.get("date").and_then(Value::as_str).unwrap_or("");
            let author = entry.get("author").and_then(Value::as_str).unwrap_or("");
            let branch = entry.get("branch").and_then(Value::as_str).unwrap_or("");
            let kind = entry.get("kind").and_then(Value::as_str).unwrap_or("manual");
            let word_count = entry.get("wordCount").and_then(Value::as_u64).unwrap_or(0);
            let diff_summary = entry.get("diffSummary").and_then(Value::as_str).unwrap_or("");

            // Card colour based on kind.
            let card_color = match kind {
                "manual" => t.accent.gamma_multiply(0.15),
                "branch" => t.green.gamma_multiply(0.15),
                _ => t.input, // auto
            };
            let border_color = match kind {
                "manual" => t.accent.gamma_multiply(0.5),
                "branch" => t.green.gamma_multiply(0.5),
                _ => t.border,
            };

            egui::Frame::NONE.fill(card_color).stroke(Stroke::new(1.0, border_color)).corner_radius(6).inner_margin(10).show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Header: badge + label.
                ui.horizontal(|ui| {
                    let badge = match kind {
                        "manual" => "[Commit]",
                        "branch" => "[Branch]",
                        _ => "[Auto]",
                    };
                    ui.label(egui::RichText::new(badge).font(semibold(10.5)).color(t.accent));
                    ui.label(egui::RichText::new(label).font(semibold(13.0)).color(t.text));
                });

                // Date and author.
                ui.horizontal(|ui| {
                    let stamp = comment_stamp(date);
                    ui.label(egui::RichText::new(stamp).font(regular(10.5)).color(t.text_dim));
                    if !author.is_empty() {
                        ui.label(egui::RichText::new(format!("• {author}")).font(regular(10.5)).color(t.text_dim));
                    }
                });

                // Branch badge.
                if !branch.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("Branch: {branch}")).font(regular(10.0)).color(t.green));
                    });
                }

                // Message.
                if !message.is_empty() {
                    ui.label(egui::RichText::new(message).font(regular(11.5)).color(t.text));
                }

                // Diff summary and stats.
                ui.horizontal(|ui| {
                    if !diff_summary.is_empty() {
                        let diff_color = if diff_summary.starts_with('+') {
                            t.green
                        } else if diff_summary.starts_with('-') {
                            egui::Color32::from_rgb(220, 80, 60)
                        } else {
                            t.text_dim
                        };
                        ui.label(egui::RichText::new(diff_summary).font(regular(10.5)).color(diff_color));
                    }
                    ui.label(egui::RichText::new(format!("{word_count} words")).font(regular(10.0)).color(t.text_disabled));
                });

                // Actions.
                ui.horizontal(|ui| {
                    if ui.small_button(tl!("Restore")).on_hover_text(tl!("Restore document to this version")).clicked() {
                        let _ = app.run("file.versions", json!({"restore": idx}));
                    }
                    // Compare button with inline diff display.
                    let compare_id = egui::Id::new(("version_compare", idx));
                    let mut comparing = ui.data(|d| d.get_temp::<bool>(compare_id)).unwrap_or(false);
                    if ui.small_button(if comparing { tl!("Hide Diff") } else { tl!("Compare") }).clicked() {
                        comparing = !comparing;
                        ui.data_mut(|d| d.insert_temp(compare_id, comparing));
                    }
                    if ui.small_button(tl!("Delete")).clicked() {
                        let _ = app.run("file.versions", json!({"delete": idx}));
                    }
                });

                // Inline diff viewer.
                let compare_id = egui::Id::new(("version_compare", idx));
                if ui.data(|d| d.get_temp::<bool>(compare_id)).unwrap_or(false) {
                    ui.separator();
                    let cmp = app.session.run("file.versions", &json!({"compare": idx}));
                    if let Ok(cmp) = cmp {
                        let summary = cmp.get("summary").and_then(Value::as_str).unwrap_or("");
                        let old_w = cmp.get("oldWords").and_then(Value::as_u64).unwrap_or(0);
                        let new_w = cmp.get("newWords").and_then(Value::as_u64).unwrap_or(0);
                        ui.label(egui::RichText::new(format!("Diff: {summary}")).font(semibold(11.0)));
                        ui.label(egui::RichText::new(format!("Words: {old_w} -> {new_w}")).font(regular(10.5)).color(t.text_dim));

                        // Full comparison window trigger button.
                        let full_window_id = egui::Id::new("version_compare_window_target");
                        if ui
                            .button(egui::RichText::new(tl!("Open Full Comparison Window…")).font(semibold(10.5)))
                            .on_hover_text(tl!("View side-by-side or unified paragraph comparison in a wide window"))
                            .clicked()
                        {
                            ui.data_mut(|d| d.insert_temp(full_window_id, Some(idx as usize)));
                        }

                        if let Some(changes) = cmp.get("changes").and_then(Value::as_array) {
                            for change in changes.iter().take(12) {
                                let p_idx = change.get("paragraph").and_then(Value::as_u64).unwrap_or(0);
                                let kind = change.get("kind").and_then(Value::as_str).unwrap_or("modified");
                                let old_text = change.get("old").and_then(Value::as_str).unwrap_or("");
                                let new_text = change.get("new").and_then(Value::as_str).unwrap_or("");

                                egui::Frame::NONE.fill(t.input).stroke(Stroke::new(1.0, t.border)).corner_radius(4).inner_margin(6).show(ui, |ui| {
                                    let old_para = change.get("oldPara").and_then(Value::as_u64);
                                    let new_para = change.get("newPara").and_then(Value::as_u64);
                                    let para_title = match (old_para, new_para) {
                                        (Some(o), Some(n)) if o == n => format!("Paragraph #{}", o),
                                        (Some(o), Some(n)) => format!("Paragraph #{} → #{}", o, n),
                                        (None, Some(n)) => format!("Paragraph +#{}", n),
                                        (Some(o), None) => format!("Paragraph -#{}", o),
                                        _ => format!("Paragraph #{}", p_idx + 1),
                                    };

                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(para_title).font(semibold(10.5)));
                                        let (badge, color) = match kind {
                                            "added" => ("[Added]", t.green),
                                            "deleted" => ("[Deleted]", egui::Color32::from_rgb(220, 60, 60)),
                                            _ => ("[Modified]", egui::Color32::from_rgb(230, 160, 40)),
                                        };
                                        ui.label(egui::RichText::new(badge).font(semibold(10.0)).color(color));
                                    });

                                    if !old_text.is_empty() && kind != "added" {
                                        egui::Frame::NONE
                                            .fill(egui::Color32::from_rgba_unmultiplied(240, 70, 70, 25))
                                            .corner_radius(3)
                                            .inner_margin(4)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.label(
                                                    egui::RichText::new("- Before:").font(semibold(9.5)).color(egui::Color32::from_rgb(200, 50, 50)),
                                                );
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new(old_text).font(regular(10.0)).color(egui::Color32::from_rgb(210, 60, 60)),
                                                    )
                                                    .wrap(),
                                                );
                                            });
                                        ui.add_space(2.0);
                                    }

                                    if !new_text.is_empty() && kind != "deleted" {
                                        egui::Frame::NONE
                                            .fill(egui::Color32::from_rgba_unmultiplied(50, 190, 80, 25))
                                            .corner_radius(3)
                                            .inner_margin(4)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.label(egui::RichText::new("+ After:").font(semibold(9.5)).color(t.green));
                                                ui.add(egui::Label::new(egui::RichText::new(new_text).font(regular(10.0)).color(t.green)).wrap());
                                            });
                                    }
                                });
                                ui.add_space(3.0);
                            }
                            if changes.len() > 12 {
                                ui.label(
                                    egui::RichText::new(format!("… and {} more changes (open full window to see all)", changes.len() - 12))
                                        .font(regular(10.0))
                                        .color(t.text_dim),
                                );
                            }
                        }
                    }
                }
            });

            ui.add_space(6.0);
        }
    });

    // ── Full-screen / Wide Comparison Window ──
    let full_window_id = egui::Id::new("version_compare_window_target");
    let full_target = ui.data(|d| d.get_temp::<Option<usize>>(full_window_id)).flatten();
    if let Some(target_idx) = full_target {
        let cmp_result = app.session.run("file.versions", &json!({"compare": target_idx}));
        if let Ok(cmp) = cmp_result {
            let version_info = cmp.get("version").cloned().unwrap_or_default();
            let v_label = version_info.get("label").and_then(Value::as_str).unwrap_or("Version");
            let v_date = version_info.get("date").and_then(Value::as_str).unwrap_or("");
            let v_author = version_info.get("author").and_then(Value::as_str).unwrap_or("");
            let summary = cmp.get("summary").and_then(Value::as_str).unwrap_or("");
            let old_w = cmp.get("oldWords").and_then(Value::as_u64).unwrap_or(0);
            let new_w = cmp.get("newWords").and_then(Value::as_u64).unwrap_or(0);

            let mut is_open = true;
            let window_title = format!("Compare: \"{}\" vs Current Document", v_label);

            egui::Window::new(window_title)
                .open(&mut is_open)
                .default_size(vec2(880.0, 580.0))
                .min_size(vec2(580.0, 380.0))
                .resizable(true)
                .collapsible(false)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        let mode_id = egui::Id::new("version_diff_view_mode");
                        let mut mode = ui.data(|d| d.get_temp::<String>(mode_id)).unwrap_or_else(|| "side_by_side".into());
                        ui.label(egui::RichText::new("View:").font(semibold(12.0)));
                        if ui.selectable_label(mode == "side_by_side", tl!("Side-by-Side")).clicked() {
                            mode = "side_by_side".into();
                            ui.data_mut(|d| d.insert_temp(mode_id, mode.clone()));
                        }
                        if ui.selectable_label(mode == "unified", tl!("Unified")).clicked() {
                            mode = "unified".into();
                            ui.data_mut(|d| d.insert_temp(mode_id, mode.clone()));
                        }

                        ui.separator();
                        ui.label(egui::RichText::new(format!("Diff: {summary}")).font(semibold(11.5)).color(t.accent));
                        ui.label(egui::RichText::new(format!("(Words: {old_w} -> {new_w})")).font(regular(11.0)).color(t.text_dim));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new(tl!("Restore This Version")).font(semibold(12.0))).clicked() {
                                let _ = app.run("file.versions", json!({"restore": target_idx}));
                            }
                        });
                    });
                    ui.separator();

                    let changes = cmp.get("changes").and_then(Value::as_array).cloned().unwrap_or_default();
                    if changes.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(
                                egui::RichText::new("No text differences between this version and the current document.")
                                    .font(regular(13.0))
                                    .color(t.text_dim),
                            );
                        });
                    } else {
                        let mode_id = egui::Id::new("version_diff_view_mode");
                        let mode = ui.data(|d| d.get_temp::<String>(mode_id)).unwrap_or_else(|| "side_by_side".into());

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if mode == "side_by_side" {
                                ui.columns(2, |cols| {
                                    cols[0].label(
                                        egui::RichText::new(format!("Saved: {v_label}"))
                                            .font(semibold(12.5))
                                            .color(egui::Color32::from_rgb(200, 60, 60)),
                                    );
                                    cols[0].label(egui::RichText::new(format!("{v_date} • {v_author}")).font(regular(10.5)).color(t.text_dim));
                                    cols[1].label(egui::RichText::new("Current Document (Active)").font(semibold(12.5)).color(t.green));
                                    cols[1].label(egui::RichText::new("Being edited").font(regular(10.5)).color(t.text_dim));
                                });
                                ui.separator();

                                for change in &changes {
                                    let p_idx = change.get("paragraph").and_then(Value::as_u64).unwrap_or(0);
                                    let kind = change.get("kind").and_then(Value::as_str).unwrap_or("modified");
                                    let old_text = change.get("old").and_then(Value::as_str).unwrap_or("");
                                    let new_text = change.get("new").and_then(Value::as_str).unwrap_or("");

                                    let old_para = change.get("oldPara").and_then(Value::as_u64);
                                    let new_para = change.get("newPara").and_then(Value::as_u64);
                                    let para_title = match (old_para, new_para) {
                                        (Some(o), Some(n)) if o == n => format!("Paragraph #{}", o),
                                        (Some(o), Some(n)) => format!("Paragraph #{} → #{}", o, n),
                                        (None, Some(n)) => format!("Paragraph +#{}", n),
                                        (Some(o), None) => format!("Paragraph -#{}", o),
                                        _ => format!("Paragraph #{}", p_idx + 1),
                                    };

                                    ui.group(|ui| {
                                        ui.label(egui::RichText::new(format!("{para_title} [{kind}]")).font(semibold(11.0)).color(t.text_dim));
                                        ui.columns(2, |cols| {
                                            if !old_text.is_empty() {
                                                egui::Frame::NONE
                                                    .fill(if kind == "added" {
                                                        t.input
                                                    } else {
                                                        egui::Color32::from_rgba_unmultiplied(240, 70, 70, 25)
                                                    })
                                                    .stroke(Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(220, 60, 60, 60)))
                                                    .inner_margin(8)
                                                    .corner_radius(4)
                                                    .show(&mut cols[0], |ui| {
                                                        ui.set_width(ui.available_width());
                                                        ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(old_text)
                                                                    .font(regular(11.0))
                                                                    .color(egui::Color32::from_rgb(210, 60, 60)),
                                                            )
                                                            .wrap(),
                                                        );
                                                    });
                                            } else {
                                                cols[0].label(egui::RichText::new("(none)").font(regular(10.5)).color(t.text_disabled));
                                            }

                                            if !new_text.is_empty() {
                                                egui::Frame::NONE
                                                    .fill(if kind == "deleted" {
                                                        t.input
                                                    } else {
                                                        egui::Color32::from_rgba_unmultiplied(50, 190, 80, 25)
                                                    })
                                                    .stroke(Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(50, 180, 80, 60)))
                                                    .inner_margin(8)
                                                    .corner_radius(4)
                                                    .show(&mut cols[1], |ui| {
                                                        ui.set_width(ui.available_width());
                                                        ui.add(
                                                            egui::Label::new(egui::RichText::new(new_text).font(regular(11.0)).color(t.green)).wrap(),
                                                        );
                                                    });
                                            } else {
                                                cols[1].label(egui::RichText::new("(deleted)").font(regular(10.5)).color(t.text_disabled));
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
                            } else {
                                for change in &changes {
                                    let p_idx = change.get("paragraph").and_then(Value::as_u64).unwrap_or(0);
                                    let kind = change.get("kind").and_then(Value::as_str).unwrap_or("modified");
                                    let old_text = change.get("old").and_then(Value::as_str).unwrap_or("");
                                    let new_text = change.get("new").and_then(Value::as_str).unwrap_or("");
                                    let old_para = change.get("oldPara").and_then(Value::as_u64);
                                    let new_para = change.get("newPara").and_then(Value::as_u64);
                                    let para_title = match (old_para, new_para) {
                                        (Some(o), Some(n)) if o == n => format!("Paragraph #{}", o),
                                        (Some(o), Some(n)) => format!("Paragraph #{} → #{}", o, n),
                                        (None, Some(n)) => format!("Paragraph +#{}", n),
                                        (Some(o), None) => format!("Paragraph -#{}", o),
                                        _ => format!("Paragraph #{}", p_idx + 1),
                                    };

                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(para_title).font(semibold(11.5)));
                                            let (k_label, k_color) = match kind {
                                                "added" => ("[Added]", t.green),
                                                "deleted" => ("[Deleted]", egui::Color32::from_rgb(220, 60, 60)),
                                                _ => ("[Modified]", egui::Color32::from_rgb(220, 160, 40)),
                                            };
                                            ui.label(egui::RichText::new(k_label).font(semibold(10.5)).color(k_color));
                                        });

                                        if !old_text.is_empty() && kind != "added" {
                                            egui::Frame::NONE
                                                .fill(egui::Color32::from_rgba_unmultiplied(240, 70, 70, 25))
                                                .stroke(Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(220, 60, 60, 60)))
                                                .inner_margin(8)
                                                .corner_radius(4)
                                                .show(ui, |ui| {
                                                    ui.set_width(ui.available_width());
                                                    ui.label(
                                                        egui::RichText::new("- Previous Version:")
                                                            .font(semibold(10.0))
                                                            .color(egui::Color32::from_rgb(220, 60, 60)),
                                                    );
                                                    ui.add(
                                                        egui::Label::new(
                                                            egui::RichText::new(old_text)
                                                                .font(regular(11.0))
                                                                .color(egui::Color32::from_rgb(220, 60, 60)),
                                                        )
                                                        .wrap(),
                                                    );
                                                });
                                            ui.add_space(2.0);
                                        }

                                        if !new_text.is_empty() && kind != "deleted" {
                                            egui::Frame::NONE
                                                .fill(egui::Color32::from_rgba_unmultiplied(50, 190, 80, 25))
                                                .stroke(Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(50, 180, 80, 60)))
                                                .inner_margin(8)
                                                .corner_radius(4)
                                                .show(ui, |ui| {
                                                    ui.set_width(ui.available_width());
                                                    ui.label(egui::RichText::new("+ Current Document:").font(semibold(10.0)).color(t.green));
                                                    ui.add(egui::Label::new(egui::RichText::new(new_text).font(regular(11.0)).color(t.green)).wrap());
                                                });
                                        }
                                    });
                                    ui.add_space(5.0);
                                }
                            }
                        });
                    }
                });

            if !is_open {
                ui.data_mut(|d| d.remove::<Option<usize>>(full_window_id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::comment_stamp;

    #[test]
    fn comment_stamp_keeps_date_and_time() {
        assert_eq!(comment_stamp("2026-10-12T04:44:31Z"), "2026-10-12 04:44");
        assert_eq!(comment_stamp("2026-10-12"), "2026-10-12");
        assert_eq!(comment_stamp(""), "");
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;

    #[test]
    fn long_headings_truncate_without_stretching_navigation() {
        let text = "An extremely long heading with words that must stay inside the navigation pane ".repeat(20);
        for (width, level) in [(260.0, 0), (180.0, 5)] {
            let ctx = egui::Context::default();
            crate::theme::install_fonts(&ctx);
            let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(900.0, 600.0));
            let mut panel_rect = egui::Rect::NOTHING;
            let mut heading_rect = egui::Rect::NOTHING;
            let output = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), ..Default::default() }, |ui| {
                let result = egui::Panel::left("navigation_test")
                    .default_size(width)
                    .resizable(true)
                    .show(ui, |ui| egui::ScrollArea::vertical().show(ui, |ui| nav_heading(ui, level, &text, true)).inner);
                panel_rect = result.response.rect;
                heading_rect = result.inner.rect;
            });

            assert!(panel_rect.width() < width + 20.0, "long heading expanded the {width}px pane to {panel_rect:?}");
            assert!(heading_rect.max.x <= panel_rect.max.x, "heading escaped pane: {heading_rect:?} / {panel_rect:?}");
            assert!(
                output.shapes.iter().any(|s| match &s.shape {
                    egui::Shape::Text(t) => {
                        t.galley.elided && t.galley.rows.last().is_some_and(|r| r.row.glyphs.last().is_some_and(|g| g.chr == '…'))
                    }
                    _ => false,
                }),
                "long heading must render a trailing ellipsis at {width}px"
            );
            output.drop_without_applying_deltas();
        }
    }
}
