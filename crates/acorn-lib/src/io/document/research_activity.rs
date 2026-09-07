//! Research activity Markdown document indexing.
use super::{DocumentEntry, DocumentParser, SourceDocument};
use acorn_core::prelude::{format, Vec};
use acorn_core::util::MarkdownSupport;
use acorn_schema::research_activity::ResearchActivity;
use convert_case::{Case, Casing};

pub(crate) struct MarkdownParser;
impl DocumentParser for MarkdownParser {
    fn try_entries(&self, document: &SourceDocument) -> Option<Vec<DocumentEntry>> {
        (document.is_markdown() && ResearchActivity::is_markdown(document.content.as_str())).then(|| self.entries(document))
    }
    fn entries(&self, document: &SourceDocument) -> Vec<DocumentEntry> {
        let lines = document
            .content
            .split_inclusive('\n')
            .scan(0usize, |start, line| {
                let offset = *start;
                *start = start.saturating_add(line.len());
                Some((offset, line.trim_end_matches(['\r', '\n'])))
            })
            .collect::<Vec<_>>();
        let body_start = lines
            .iter()
            .enumerate()
            .filter(|(_, (_, line))| line.trim() == "---")
            .nth(1)
            .map(|(index, _)| index.saturating_add(1))
            .unwrap_or_default();
        let body = lines.get(body_start..).unwrap_or_default();
        let title = body
            .iter()
            .find(|(_, line)| line.starts_with("# "))
            .and_then(|(start, line)| Self::line("title", *start, line, "# "));
        let subtitle = body
            .iter()
            .take_while(|(_, line)| !line.starts_with("## "))
            .filter_map(|(start, line)| Self::line("subtitle", *start, line, "> "));
        let headings = body
            .iter()
            .enumerate()
            .filter_map(|(index, (_, line))| line.strip_prefix("## ").map(|name| (index, name.trim())))
            .collect::<Vec<_>>();
        title
            .into_iter()
            .chain(subtitle)
            .chain(headings.iter().enumerate().flat_map(|(offset, (start, name))| {
                let end = headings.get(offset.saturating_add(1)).map(|(index, _)| *index).unwrap_or(body.len());
                Self::sections(&document.content, name, body.get(start.saturating_add(1)..end).unwrap_or_default())
            }))
            .collect()
    }
    fn line(path: &str, start: usize, line: &str, prefix: &str) -> Option<DocumentEntry> {
        line.strip_prefix(prefix)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .and_then(|value| {
                line.find(value).map(|relative| {
                    DocumentEntry::markdown(
                        path,
                        value.decode_markdown_text(),
                        start.saturating_add(relative)..start.saturating_add(relative).saturating_add(value.len()),
                    )
                })
            })
    }
    fn lists(path: &str, lines: &[(usize, &str)]) -> Vec<DocumentEntry> {
        lines
            .iter()
            .filter_map(|(start, line)| {
                line.strip_prefix("- ")
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| (*start, *line, value))
            })
            .enumerate()
            .filter_map(|(index, (start, line, value))| {
                line.find(value).map(|relative| {
                    DocumentEntry::markdown(
                        &format!("{path}[{index}]"),
                        value.decode_markdown_text(),
                        start.saturating_add(relative)..start.saturating_add(relative).saturating_add(value.len()),
                    )
                })
            })
            .collect()
    }
    fn scalar(content: &str, path: &str, lines: &[(usize, &str)]) -> Option<DocumentEntry> {
        let populated = lines.iter().filter(|(_, line)| !line.trim().is_empty()).collect::<Vec<_>>();
        populated.first().and_then(|(first_start, first_line)| {
            populated.last().and_then(|(last_start, last_line)| {
                let first = first_line.trim();
                let last = last_line.trim();
                let start = first_start.saturating_add(first_line.find(first).unwrap_or_default());
                let end = last_start
                    .saturating_add(last_line.find(last).unwrap_or_default())
                    .saturating_add(last.len());
                content
                    .get(start..end)
                    .map(|value| DocumentEntry::markdown(path, value.decode_markdown_text(), start..end))
            })
        })
    }
    fn sections(content: &str, name: &str, lines: &[(usize, &str)]) -> Vec<DocumentEntry> {
        match name {
            | "Mission" | "Challenge" | "Focus" => Self::scalar(content, &format!("sections.{}", name.to_case(Case::Snake)), lines)
                .into_iter()
                .collect(),
            | "Approach" | "Impact" | "Achievement" | "Capabilities" | "Areas" => {
                Self::lists(&format!("sections.{}", name.to_case(Case::Snake)), lines)
            }
            | "Contact" => Self::contact_entries(lines),
            | _ => Vec::new(),
        }
    }
}
impl MarkdownParser {
    fn contact_entries(lines: &[(usize, &str)]) -> Vec<DocumentEntry> {
        lines
            .iter()
            .filter_map(|(start, line)| {
                line.strip_prefix("- ")
                    .and_then(|entry| entry.split_once(':'))
                    .and_then(|(label, raw)| Self::contact_entry(*start, line, label.trim(), raw.trim()))
            })
            .collect()
    }
    fn contact_entry(start: usize, line: &str, label: &str, raw: &str) -> Option<DocumentEntry> {
        let field = label.to_case(Case::Snake);
        let displayed = (field == "email")
            .then(|| raw.strip_prefix('[').and_then(|value| value.split_once("](")).map(|(value, _)| value))
            .flatten()
            .unwrap_or(raw);
        line.find(displayed).map(|relative| {
            DocumentEntry::markdown(
                &format!("contact.{field}"),
                displayed.decode_markdown_text(),
                start.saturating_add(relative)..start.saturating_add(relative).saturating_add(displayed.len()),
            )
        })
    }
}
