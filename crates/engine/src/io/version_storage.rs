//! Local offline persistence for document Version History.
//!
//! Stores version snapshots and metadata under `<settings>/versions/<doc_key>/`.
//! Everything is stored offline-first in the user's WordCraft config directory;
//! no duplicate or hidden files are created in the user's document directory.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::session::{Session, VersionEntry, VersionKind};

/// Simple 64-bit FNV-1a hash for paths.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// The base versions storage directory: `<settings>/versions`.
pub fn versions_base_dir() -> Option<PathBuf> {
    if std::env::var_os("WORDCRAFT_NO_PREFS").is_some() {
        return None;
    }
    wordcraft_control_key::settings_dir().map(|p| p.join("versions"))
}

/// A stable folder path for a document: `<settings>/versions/<safe_stem>_<hash>/`.
pub fn document_version_dir(base: &Path, path: Option<&Path>, untitled_id: &str) -> PathBuf {
    if let Some(p) = path {
        let normalized = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        let norm_str = normalized.to_string_lossy();
        let hash = fnv1a_64(norm_str.as_bytes());
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
        let safe_stem: String = stem.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).take(32).collect();
        base.join(format!("{safe_stem}_{hash:016x}"))
    } else {
        let safe_id: String = untitled_id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).take(40).collect();
        base.join(format!("untitled_{safe_id}"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedIndex {
    #[serde(default)]
    pub doc_path: Option<String>,
    #[serde(default)]
    pub branch: String,
    pub versions: Vec<PersistedEntryMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedEntryMeta {
    pub file: String,
    pub label: String,
    pub message: String,
    pub date: String,
    pub author: String,
    pub branch: String,
    pub kind: String,
    pub word_count: usize,
    pub para_count: usize,
    pub diff_summary: String,
}

/// Persist all versions in `s.versions` to the document's versions directory.
pub fn persist_versions(s: &Session) {
    let Some(base) = versions_base_dir() else { return };
    let dir = document_version_dir(&base, s.path.as_deref(), &s.document_uuid);

    if let Err(e) = std::fs::create_dir_all(&dir) {
        log::warn!("versions: failed to create dir {}: {e}", dir.display());
        return;
    }

    let mut metas = Vec::with_capacity(s.versions.len());

    for (idx, entry) in s.versions.iter().enumerate() {
        let filename = format!("v{idx}.docx");
        let file_path = dir.join(&filename);

        // Only write snapshot file if it does not already exist
        if !file_path.exists() {
            match wordcraft_docx::write(&entry.doc) {
                Ok(bytes) => {
                    let tmp = dir.join(format!("v{idx}.tmp"));
                    if std::fs::write(&tmp, &bytes).is_ok() {
                        let _ = std::fs::rename(&tmp, &file_path);
                    }
                }
                Err(e) => {
                    log::warn!("versions: failed to serialize snapshot v{idx}: {e}");
                    // Fall back to JSON snapshot if docx serialization fails
                    let json_name = format!("v{idx}.json");
                    let json_path = dir.join(&json_name);
                    if let Ok(json_bytes) = serde_json::to_vec(&entry.doc) {
                        let _ = std::fs::write(&json_path, json_bytes);
                    }
                }
            }
        }

        let kind_str = match entry.kind {
            VersionKind::Auto => "Auto",
            VersionKind::Branch => "Branch",
            VersionKind::Manual => "Manual",
        };

        metas.push(PersistedEntryMeta {
            file: filename,
            label: entry.label.clone(),
            message: entry.message.clone(),
            date: entry.date.clone(),
            author: entry.author.clone(),
            branch: entry.branch.clone(),
            kind: kind_str.into(),
            word_count: entry.word_count,
            para_count: entry.para_count,
            diff_summary: entry.diff_summary.clone(),
        });
    }

    let index =
        PersistedIndex { doc_path: s.path.as_ref().map(|p| p.to_string_lossy().to_string()), branch: s.version_branch.clone(), versions: metas };

    if let Ok(json_bytes) = serde_json::to_vec_pretty(&index) {
        let tmp = dir.join("index.json.tmp");
        if std::fs::write(&tmp, json_bytes).is_ok() {
            let _ = std::fs::rename(&tmp, dir.join("index.json"));
        }
    }

    // If untitled, also update autorecover snapshot as emergency backup
    if s.path.is_none()
        && let Some(settings) = wordcraft_control_key::settings_dir()
    {
        let rec_dir = settings.join("autorecover");
        let _ = std::fs::create_dir_all(&rec_dir);
        if let Ok(bytes) = wordcraft_docx::write(&s.doc) {
            let _ = std::fs::write(rec_dir.join("autorecover_unsaved.docx"), bytes);
        }
    }
}

/// Load persisted versions from disk for the current document.
pub fn load_versions(s: &mut Session) {
    let Some(base) = versions_base_dir() else { return };
    let dir = document_version_dir(&base, s.path.as_deref(), &s.document_uuid);
    let index_file = dir.join("index.json");

    let Ok(bytes) = std::fs::read(&index_file) else { return };
    let Ok(index) = serde_json::from_slice::<PersistedIndex>(&bytes) else { return };

    if !index.branch.is_empty() {
        s.version_branch = index.branch;
    }

    let mut loaded = Vec::new();
    for meta in index.versions {
        let file_path = dir.join(&meta.file);
        let doc = if meta.file.ends_with(".docx") {
            let Ok(file_bytes) = std::fs::read(&file_path) else { continue };
            match wordcraft_docx::read(&file_bytes) {
                Ok(d) => d,
                Err(e) => {
                    log::warn!("versions: failed to read {}: {e}", file_path.display());
                    continue;
                }
            }
        } else if meta.file.ends_with(".json") {
            let Ok(file_bytes) = std::fs::read(&file_path) else { continue };
            match serde_json::from_slice::<wordcraft_doc::Document>(&file_bytes) {
                Ok(d) => d,
                Err(e) => {
                    log::warn!("versions: failed to parse {}: {e}", file_path.display());
                    continue;
                }
            }
        } else {
            continue;
        };

        let kind = match meta.kind.as_str() {
            "Auto" => VersionKind::Auto,
            "Branch" => VersionKind::Branch,
            _ => VersionKind::Manual,
        };

        loaded.push(VersionEntry {
            label: meta.label,
            message: meta.message,
            date: meta.date,
            author: meta.author,
            branch: meta.branch,
            kind,
            word_count: meta.word_count,
            para_count: meta.para_count,
            diff_summary: meta.diff_summary,
            doc,
        });
    }

    if !loaded.is_empty() {
        s.versions = loaded;
    }
}

/// When an untitled document is saved to a file, migrate its versions to the file's folder.
pub fn migrate_untitled_versions(s: &Session, new_path: &Path) {
    let Some(base) = versions_base_dir() else { return };
    let old_dir = document_version_dir(&base, None, &s.document_uuid);
    let new_dir = document_version_dir(&base, Some(new_path), &s.document_uuid);

    if old_dir.exists() && old_dir != new_dir {
        let _ = std::fs::create_dir_all(&new_dir);
        if let Ok(entries) = std::fs::read_dir(&old_dir) {
            for entry in entries.flatten() {
                let target = new_dir.join(entry.file_name());
                let _ = std::fs::copy(entry.path(), target);
            }
        }
        let _ = std::fs::remove_dir_all(&old_dir);
    }
}
