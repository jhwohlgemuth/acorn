//! DOCX input utilities.
use crate::io::ApiResult;
use crate::prelude::{read, PathBuf};
use color_eyre::eyre::eyre;
use docx_rs::{
    read_docx, Delete, DeleteChild, DocumentChild, Hyperlink, Insert, InsertChild, MoveTo, MoveToChild, Paragraph, ParagraphChild, Run, RunChild,
    Table, TableCell, TableCellContent, TableChild, TableRow, TableRowChild,
};

fn extract_from_table(table: &Table) -> String {
    table
        .rows
        .iter()
        .map(|row| match row {
            | TableChild::TableRow(value) => extract_from_table_row(value),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn extract_from_table_row(row: &TableRow) -> String {
    row.cells
        .iter()
        .map(|cell| match cell {
            | TableRowChild::TableCell(value) => extract_from_table_cell(value),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn extract_from_table_cell(cell: &TableCell) -> String {
    cell.children
        .iter()
        .map(|content| match content {
            | TableCellContent::Paragraph(paragraph) => extract_from_paragraph(paragraph),
            | TableCellContent::Table(table) => extract_from_table(table),
            | TableCellContent::StructuredDataTag(_) | TableCellContent::TableOfContents(_) => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn extract_from_paragraph(paragraph: &Paragraph) -> String {
    paragraph
        .children
        .iter()
        .map(|child| match child {
            | ParagraphChild::Run(run) => extract_from_run(run),
            | ParagraphChild::Hyperlink(hyperlink) => extract_from_hyperlink(hyperlink),
            | ParagraphChild::Insert(insert) => extract_from_insert(insert),
            | ParagraphChild::Delete(delete) => extract_from_delete(delete),
            | ParagraphChild::MoveFrom(_)
            | ParagraphChild::BookmarkStart(_)
            | ParagraphChild::BookmarkEnd(_)
            | ParagraphChild::CommentStart(_)
            | ParagraphChild::CommentEnd(_)
            | ParagraphChild::StructuredDataTag(_) => "".to_string(),
            | ParagraphChild::MoveTo(move_to) => extract_from_move_to(move_to),
            | ParagraphChild::PageNum(_) | ParagraphChild::NumPages(_) => " ".to_string(),
        })
        .collect::<String>()
}
fn extract_from_hyperlink(hyperlink: &Hyperlink) -> String {
    hyperlink
        .children
        .iter()
        .map(|child| match child {
            | ParagraphChild::Run(run) => extract_from_run(run),
            | ParagraphChild::Insert(insert) => extract_from_insert(insert),
            | ParagraphChild::Delete(delete) => extract_from_delete(delete),
            | ParagraphChild::MoveFrom(_)
            | ParagraphChild::Hyperlink(_)
            | ParagraphChild::BookmarkStart(_)
            | ParagraphChild::BookmarkEnd(_)
            | ParagraphChild::CommentStart(_)
            | ParagraphChild::CommentEnd(_)
            | ParagraphChild::StructuredDataTag(_) => "".to_string(),
            | ParagraphChild::MoveTo(move_to) => extract_from_move_to(move_to),
            | ParagraphChild::PageNum(_) | ParagraphChild::NumPages(_) => " ".to_string(),
        })
        .collect::<String>()
}
fn extract_from_move_to(move_to: &MoveTo) -> String {
    move_to
        .children
        .iter()
        .map(|child| match child {
            | MoveToChild::Run(run) => extract_from_run(run),
            | MoveToChild::Delete(delete) => extract_from_delete(delete),
            | MoveToChild::CommentStart(_) | MoveToChild::CommentEnd(_) => "".to_string(),
        })
        .collect::<String>()
}
fn extract_from_insert(insert: &Insert) -> String {
    insert
        .children
        .iter()
        .map(|child| match child {
            | InsertChild::Run(run) => extract_from_run(run),
            | InsertChild::Delete(delete) => extract_from_delete(delete),
            | InsertChild::CommentStart(_) | InsertChild::CommentEnd(_) => "".to_string(),
        })
        .collect::<String>()
}
fn extract_from_delete(delete: &Delete) -> String {
    delete
        .children
        .iter()
        .map(|child| match child {
            | DeleteChild::Run(run) => extract_from_run(run),
            | DeleteChild::CommentStart(_) | DeleteChild::CommentEnd(_) => "".to_string(),
        })
        .collect::<String>()
}
fn extract_from_run(run: &Run) -> String {
    run.children
        .iter()
        .map(|child| match child {
            | RunChild::Text(text) => text.text.clone(),
            | RunChild::InstrTextString(text) => text.clone(),
            | RunChild::Tab(_) => "\t".to_string(),
            | RunChild::Break(_) | RunChild::CarriageReturn(_) => "\n".to_string(),
            | RunChild::Sym(_)
            | RunChild::DeleteText(_)
            | RunChild::PTab(_)
            | RunChild::Drawing(_)
            | RunChild::Shape(_)
            | RunChild::CommentStart(_)
            | RunChild::CommentEnd(_)
            | RunChild::FieldChar(_)
            | RunChild::InstrText(_)
            | RunChild::DeleteInstrText(_)
            | RunChild::FootnoteReference(_)
            | RunChild::Shading(_) => "".to_string(),
        })
        .collect::<String>()
}
/// Extract visible text from a DOCX file.
///
/// The extracted output is paragraph-oriented and joined with newline separators.
pub fn extract_text_from_path<P>(path: P) -> ApiResult<String>
where
    P: Into<PathBuf>,
{
    let path = path.into();
    let bytes = read(path.clone()).map_err(|why| eyre!("Failed to read DOCX file {} - {why}", path.display()))?;
    let docx = read_docx(&bytes).map_err(|why| eyre!("Failed to parse DOCX file {} - {why}", path.display()))?;
    let text = docx
        .document
        .children
        .iter()
        .filter_map(|child| match child {
            | DocumentChild::Paragraph(p) => {
                let text = extract_from_paragraph(p).trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            | DocumentChild::Table(t) => {
                let text = extract_from_table(t).trim().to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            | _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(text)
}

#[cfg(test)]
mod tests;
