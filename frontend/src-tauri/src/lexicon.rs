use crate::db::{ChapterNode, CorrectionRule, LexiconProfile, LexiconSourceChunk, LexiconTerm};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const MAX_TEXTBOOK_UPLOAD_CHARS: usize = 120_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadSelection {
    pub total_document_chars: usize,
    pub budget_chars: usize,
    pub selected_chars: usize,
    pub chunks: Vec<LexiconSourceChunk>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedLexicon {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub chapters: Vec<GeneratedChapter>,
    #[serde(default)]
    pub terms: Vec<GeneratedTerm>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedChapter {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub parent_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedTerm {
    #[serde(default)]
    pub canonical_term: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub abbreviation: Option<String>,
    #[serde(default)]
    pub english_name: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    #[serde(default)]
    pub chapter_titles: Vec<String>,
    #[serde(default)]
    pub common_asr_errors: Vec<String>,
    #[serde(default)]
    pub source_references: Vec<String>,
}

pub struct ParsedLexicon {
    pub profile: LexiconProfile,
    pub managed_path: PathBuf,
    pub file_type: String,
    pub extracted_chars: i64,
    pub extraction_quality: String,
    pub chunks: Vec<(String, String)>,
}

pub fn upload_budget(total_document_chars: usize) -> usize {
    (total_document_chars / 10).min(MAX_TEXTBOOK_UPLOAD_CHARS)
}

pub fn select_upload_chunks(
    chunks: &[LexiconSourceChunk],
    total_document_chars: usize,
) -> UploadSelection {
    let budget_chars = upload_budget(total_document_chars);
    if budget_chars == 0 {
        return UploadSelection {
            total_document_chars,
            budget_chars,
            selected_chars: 0,
            chunks: Vec::new(),
        };
    }
    let mut ranked = chunks
        .iter()
        .filter(|chunk| !chunk.text.trim().is_empty())
        .map(|chunk| {
            let text = chunk.text.to_ascii_lowercase();
            let label = chunk
                .source_label
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase();
            let mut score = 10_i32;
            if chunk.ordinal < 6 {
                score += 80;
            }
            if text.contains("目录")
                || text.contains("contents")
                || text.contains("table of contents")
                || text.contains("版权")
                || text.contains("copyright")
                || text.contains("isbn")
            {
                score += 60;
            }
            if text.lines().count() <= 3
                || text.starts_with('#')
                || text.contains("章节")
                || label.contains("heading")
            {
                score += 35;
            }
            if text.contains('：') || text.contains(':') || text.contains("definition") {
                score += 20;
            }
            (score, chunk.ordinal, chunk)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let mut selected = Vec::new();
    let mut selected_chars = 0usize;
    for (_, _, chunk) in ranked {
        if selected_chars >= budget_chars {
            break;
        }
        let remaining = budget_chars - selected_chars;
        let text = chunk.text.chars().take(remaining).collect::<String>();
        let chars = text.chars().count();
        if chars == 0 {
            continue;
        }
        selected.push(LexiconSourceChunk {
            id: chunk.id.clone(),
            ordinal: chunk.ordinal,
            source_label: chunk.source_label.clone(),
            text,
            char_count: chars as i64,
            selected_for_upload: true,
        });
        selected_chars += chars;
    }
    selected.sort_by_key(|chunk| chunk.ordinal);
    UploadSelection {
        total_document_chars,
        budget_chars,
        selected_chars,
        chunks: selected,
    }
}

pub fn merge_generated_profile(
    current: &LexiconProfile,
    generated: GeneratedLexicon,
) -> Result<LexiconProfile, String> {
    let mut profile = current.clone();
    if let Some(title) = generated.title.filter(|value| !value.trim().is_empty()) {
        profile.textbook_title = title.trim().chars().take(300).collect();
        profile.name = profile.textbook_title.clone();
    }
    let mut chapters = current.chapters.clone();
    let mut chapter_ids = std::collections::HashMap::<String, String>::new();
    for chapter in &chapters {
        chapter_ids.insert(chapter.title.to_lowercase(), chapter.id.clone());
    }
    for generated_chapter in generated.chapters.into_iter().take(256) {
        let title = generated_chapter.title.trim();
        if title.is_empty() {
            continue;
        }
        let key = title.to_lowercase();
        if chapter_ids.contains_key(&key) {
            continue;
        }
        let id = format!("chapter-generated-{}", Uuid::new_v4());
        let parent_id = generated_chapter
            .parent_title
            .as_deref()
            .and_then(|parent| chapter_ids.get(&parent.to_lowercase()))
            .cloned();
        chapter_ids.insert(key, id.clone());
        chapters.push(ChapterNode {
            id,
            parent_id,
            order: chapters.len() as i64,
            title: title.chars().take(300).collect(),
            label: generated_chapter
                .label
                .map(|value| value.chars().take(80).collect()),
            source_document_id: current.source_document_id.clone(),
            source_page: None,
            source_slide: None,
        });
    }
    let mut terms = current.terms.clone();
    let mut term_ids = std::collections::HashMap::<String, usize>::new();
    for (index, term) in terms.iter().enumerate() {
        term_ids.insert(term.canonical_term.to_lowercase(), index);
    }
    for generated_term in generated.terms.into_iter().take(512) {
        let canonical = generated_term.canonical_term.trim();
        if !(2..=200).contains(&canonical.chars().count()) {
            continue;
        }
        let chapter_ids_for_term = generated_term
            .chapter_titles
            .iter()
            .filter_map(|title| chapter_ids.get(&title.to_lowercase()).cloned())
            .collect::<Vec<_>>();
        let key = canonical.to_lowercase();
        if let Some(index) = term_ids.get(&key).copied() {
            let term = &mut terms[index];
            term.aliases = merge_strings(&term.aliases, &generated_term.aliases, 32);
            term.common_asr_errors = merge_strings(
                &term.common_asr_errors,
                &generated_term.common_asr_errors,
                32,
            );
            if term.definition.is_none() {
                term.definition = generated_term
                    .definition
                    .map(|value| value.chars().take(1_000).collect());
            }
            if term.chapter_ids.is_empty() {
                term.chapter_ids = chapter_ids_for_term;
            }
            continue;
        }
        let index = terms.len();
        term_ids.insert(key, index);
        terms.push(LexiconTerm {
            id: Uuid::new_v4().to_string(),
            canonical_term: canonical.chars().take(200).collect(),
            aliases: generated_term.aliases.into_iter().take(32).collect(),
            abbreviation: generated_term
                .abbreviation
                .map(|value| value.chars().take(80).collect()),
            english_name: generated_term
                .english_name
                .map(|value| value.chars().take(160).collect()),
            definition: generated_term
                .definition
                .map(|value| value.chars().take(1_000).collect()),
            chapter_ids: chapter_ids_for_term,
            common_asr_errors: generated_term
                .common_asr_errors
                .into_iter()
                .take(32)
                .collect(),
            source_references: generated_term
                .source_references
                .into_iter()
                .take(32)
                .collect(),
            confirmed_by_user: false,
        });
    }
    profile.version = current.version.saturating_add(1);
    profile.chapters = chapters;
    profile.terms = terms;
    profile.updated_at = chrono::Utc::now().to_rfc3339();
    Ok(profile)
}

fn merge_strings(left: &[String], right: &[String], max: usize) -> Vec<String> {
    let mut values = left.to_vec();
    for value in right {
        let value = value.trim();
        if !value.is_empty() && !values.iter().any(|existing| existing == value) {
            values.push(value.to_string());
        }
        if values.len() >= max {
            break;
        }
    }
    values
}

pub fn parse_and_copy(
    data_dir: &Path,
    source_path: &str,
    requested_name: Option<&str>,
) -> Result<ParsedLexicon, String> {
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err("SOURCE_DOCUMENT_READ_FAILED".to_string());
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "pdf" | "docx" | "pptx" | "txt" | "md" | "markdown"
    ) {
        return Err("SOURCE_DOCUMENT_UNSUPPORTED".to_string());
    }
    let text = extract_text(&source, &extension)?;
    let text = normalise_text(&text);
    if text.chars().count() < 20 {
        return Err("SOURCE_DOCUMENT_TEXT_NOT_FOUND".to_string());
    }
    let document_id = Uuid::new_v4().to_string();
    let managed_dir = data_dir.join("documents").join(&document_id);
    std::fs::create_dir_all(&managed_dir).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let safe_extension = if extension == "markdown" {
        "md"
    } else {
        extension.as_str()
    };
    let managed_path = managed_dir.join(format!("source.{safe_extension}"));
    std::fs::copy(&source, &managed_path).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let title = requested_name
        .filter(|value| !value.trim().is_empty())
        .map(str::trim)
        .unwrap_or_else(|| first_non_empty_line(&text).unwrap_or("未命名教材"));
    let chapters = extract_chapters(&text, &document_id);
    let terms = extract_terms(&text, &chapters);
    let now = chrono::Utc::now().to_rfc3339();
    let profile = LexiconProfile {
        id: Uuid::new_v4().to_string(),
        name: title.to_string(),
        version: 1,
        textbook_title: title.to_string(),
        source_document_id: document_id,
        chapters,
        terms,
        correction_rules: Vec::<CorrectionRule>::new(),
        created_at: now.clone(),
        updated_at: now,
    };
    let chunks = text
        .split("\n\n")
        .enumerate()
        .filter_map(|(index, value)| {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some((format!("paragraph-{}", index + 1), value.to_string()))
            }
        })
        .collect::<Vec<_>>();
    Ok(ParsedLexicon {
        profile,
        managed_path,
        file_type: extension,
        extracted_chars: text.chars().count() as i64,
        extraction_quality: "text_layer".to_string(),
        chunks,
    })
}

fn extract_text(path: &Path, extension: &str) -> Result<String, String> {
    match extension {
        "txt" | "md" | "markdown" => std::fs::read(path)
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string()),
        "pdf" => {
            pdf_extract::extract_text(path).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())
        }
        "docx" => extract_ooxml(path, "word/document.xml"),
        "pptx" => extract_pptx(path),
        _ => Err("SOURCE_DOCUMENT_UNSUPPORTED".to_string()),
    }
}

fn extract_ooxml(path: &Path, main_part: &str) -> Result<String, String> {
    let file = File::open(path).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let mut xml = String::new();
    archive
        .by_name(main_part)
        .map_err(|_| "SOURCE_DOCUMENT_TEXT_NOT_FOUND".to_string())?
        .read_to_string(&mut xml)
        .map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let text = strip_xml(&xml);
    if text.trim().is_empty() {
        Err("SOURCE_DOCUMENT_TEXT_NOT_FOUND".to_string())
    } else {
        Ok(text)
    }
}

fn extract_pptx(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
    let mut slide_names = (0..archive.len())
        .filter_map(|index| {
            archive
                .by_index(index)
                .ok()
                .map(|entry| entry.name().to_string())
        })
        .filter(|name| name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
        .collect::<Vec<_>>();
    slide_names.sort();
    let mut result = String::new();
    for name in slide_names {
        let mut xml = String::new();
        archive
            .by_name(&name)
            .map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?
            .read_to_string(&mut xml)
            .map_err(|_| "SOURCE_DOCUMENT_READ_FAILED".to_string())?;
        let slide_text = strip_xml(&xml);
        if !slide_text.trim().is_empty() {
            result.push_str(&format!("\n\n[{}]\n{}", name, slide_text));
        }
    }
    if result.trim().is_empty() {
        Err("SOURCE_DOCUMENT_TEXT_NOT_FOUND".to_string())
    } else {
        Ok(result)
    }
}

fn strip_xml(xml: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in xml.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    decode_entities(&output)
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn normalise_text(value: &str) -> String {
    value
        .replace('\u{00a0}', " ")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_non_empty_line(text: &str) -> Option<&str> {
    text.lines().map(str::trim).find(|line| !line.is_empty())
}

fn extract_chapters(text: &str, document_id: &str) -> Vec<ChapterNode> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            let is_heading = line.starts_with('#')
                || line.contains("章") && line.chars().count() <= 80
                || line.to_ascii_lowercase().starts_with("chapter ")
                || line
                    .as_bytes()
                    .first()
                    .map(|value| value.is_ascii_digit())
                    .unwrap_or(false)
                    && (line.contains('.') || line.contains('、'));
            if !is_heading {
                return None;
            }
            let title = line.trim_start_matches('#').trim().to_string();
            if title.len() < 2 {
                return None;
            }
            Some(ChapterNode {
                id: format!("chapter-{}", index + 1),
                parent_id: None,
                order: index as i64,
                title,
                label: None,
                source_document_id: document_id.to_string(),
                source_page: None,
                source_slide: None,
            })
        })
        .collect()
}

fn extract_terms(text: &str, chapters: &[ChapterNode]) -> Vec<LexiconTerm> {
    let mut counts = BTreeMap::<String, usize>::new();
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|character: char| {
            !character.is_ascii_alphanumeric()
                && !is_cjk(character)
                && character != '-'
                && character != '_'
        });
        if cleaned.len() >= 2
            && cleaned.len() <= 40
            && cleaned
                .chars()
                .any(|character| character.is_ascii_uppercase())
            && cleaned
                .chars()
                .any(|character| character.is_ascii_alphabetic())
        {
            *counts.entry(cleaned.to_string()).or_default() += 1;
        }
    }
    for line in text.lines() {
        let line = line.trim();
        if line.contains(':') || line.contains('：') || line.starts_with('#') {
            for piece in line.split(|character| {
                character == ':'
                    || character == '：'
                    || character == ','
                    || character == '，'
                    || character == '、'
            }) {
                let cleaned = piece.trim().trim_start_matches('#').trim();
                if (2..=18).contains(&cleaned.chars().count())
                    && cleaned.chars().all(|character| {
                        is_cjk(character)
                            || character.is_ascii_alphabetic()
                            || character == '-'
                            || character.is_ascii_digit()
                    })
                    && !is_common_word(cleaned)
                {
                    *counts.entry(cleaned.to_string()).or_default() += 1;
                }
            }
        }
    }
    let mut seen = HashSet::new();
    counts
        .into_iter()
        .filter(|(term, count)| *count >= 1 && seen.insert(term.clone()))
        .take(128)
        .map(|(term, _)| LexiconTerm {
            id: Uuid::new_v4().to_string(),
            canonical_term: term,
            aliases: Vec::new(),
            abbreviation: None,
            english_name: None,
            definition: None,
            chapter_ids: chapters
                .iter()
                .take(1)
                .map(|chapter| chapter.id.clone())
                .collect(),
            common_asr_errors: Vec::new(),
            source_references: Vec::new(),
            confirmed_by_user: false,
        })
        .collect()
}

fn is_cjk(character: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&character)
}

fn is_common_word(value: &str) -> bool {
    matches!(
        value,
        "本章" | "内容" | "定义" | "说明" | "例如" | "目录" | "关键词" | "重点"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &str, ordinal: i64, text: &str) -> LexiconSourceChunk {
        LexiconSourceChunk {
            id: id.to_string(),
            ordinal,
            source_label: Some(format!("paragraph-{ordinal}")),
            text: text.to_string(),
            char_count: text.chars().count() as i64,
            selected_for_upload: false,
        }
    }

    #[test]
    fn upload_selection_never_exceeds_unicode_budget() {
        let chunks = vec![
            chunk("a", 0, "目录：第一章 网络基础"),
            chunk("b", 1, &"术语".repeat(40)),
            chunk("c", 2, &"正文".repeat(40)),
        ];
        let selection = select_upload_chunks(&chunks, 100);
        assert_eq!(selection.budget_chars, 10);
        assert!(selection.selected_chars <= selection.budget_chars);
        assert_eq!(
            selection.selected_chars,
            selection
                .chunks
                .iter()
                .map(|value| value.text.chars().count())
                .sum::<usize>()
        );
    }

    #[test]
    fn upload_budget_caps_large_document_at_120k() {
        assert_eq!(upload_budget(2_000_000), 120_000);
        assert_eq!(upload_budget(99), 9);
    }
}
