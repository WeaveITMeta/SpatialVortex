// Document Parser for RAG System
// Handles PDF, DOCX, and Excel files

use anyhow::{Context, Result};
use std::path::Path;

/// Supported document types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocumentType {
    Pdf,
    Docx,
    Excel,
    Text,
    Unknown,
}

impl DocumentType {
    /// Detect document type from file extension
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("pdf") => DocumentType::Pdf,
            Some("docx") => DocumentType::Docx,
            Some("xlsx") | Some("xls") => DocumentType::Excel,
            Some("txt") | Some("md") | Some("rs") | Some("js") | Some("ts") | Some("py") => {
                DocumentType::Text
            }
            _ => DocumentType::Unknown,
        }
    }

    /// Detect from MIME type
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            "application/pdf" => DocumentType::Pdf,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                DocumentType::Docx
            }
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                DocumentType::Excel
            }
            "text/plain" | "text/markdown" => DocumentType::Text,
            _ => DocumentType::Unknown,
        }
    }
}

/// Parsed document with extracted text and metadata
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub content: String,
    pub doc_type: DocumentType,
    pub page_count: Option<usize>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// Main document parser
pub struct DocumentParser;

impl DocumentParser {
    /// Parse document from bytes
    pub fn parse_from_bytes(bytes: &[u8], doc_type: DocumentType) -> Result<ParsedDocument> {
        match doc_type {
            DocumentType::Pdf => Self::parse_pdf(bytes),
            DocumentType::Docx => Self::parse_docx(bytes),
            DocumentType::Excel => Self::parse_excel(bytes),
            DocumentType::Text => Ok(ParsedDocument {
                content: String::from_utf8_lossy(bytes).to_string(),
                doc_type: DocumentType::Text,
                page_count: None,
                metadata: DocumentMetadata::default(),
            }),
            DocumentType::Unknown => anyhow::bail!("Unknown document type"),
        }
    }

    /// Parse document from file path
    pub fn parse_from_file(path: &Path) -> Result<ParsedDocument> {
        let doc_type = DocumentType::from_path(path);
        let bytes = std::fs::read(path).context("Failed to read file")?;
        Self::parse_from_bytes(&bytes, doc_type)
    }

    /// Parse PDF document
    fn parse_pdf(bytes: &[u8]) -> Result<ParsedDocument> {
        use pdf_extract::*;

        // Create temporary file for pdf-extract (it needs a file path)
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("temp_pdf_{}.pdf", uuid::Uuid::new_v4()));
        std::fs::write(&temp_path, bytes).context("Failed to write temp PDF")?;

        let content = extract_text(&temp_path)
            .context("Failed to extract PDF text")?;

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        // Estimate page count from content length (rough estimate)
        let page_count = Some((content.len() / 2000).max(1));

        Ok(ParsedDocument {
            content,
            doc_type: DocumentType::Pdf,
            page_count,
            metadata: DocumentMetadata::default(),
        })
    }

    /// Parse DOCX document
    fn parse_docx(bytes: &[u8]) -> Result<ParsedDocument> {
        use docx_rs::*;
        let docx = read_docx(bytes).context("Failed to parse DOCX")?;

        // Extract text from all paragraphs
        let mut content = String::new();
        for child in docx.document.children {
            if let DocumentChild::Paragraph(para) = child {
                for run in para.children {
                    if let ParagraphChild::Run(run_data) = run {
                        for run_child in run_data.children {
                            if let RunChild::Text(text) = run_child {
                                content.push_str(&text.text);
                                content.push(' ');
                            }
                        }
                    }
                }
                content.push('\n');
            }
        }

        // Extract metadata - use DocumentMetadata::default() as CoreProps structure may vary
        let metadata = DocumentMetadata::default();

        Ok(ParsedDocument {
            content: content.trim().to_string(),
            doc_type: DocumentType::Docx,
            page_count: None,
            metadata,
        })
    }

    /// Parse Excel document
    fn parse_excel(bytes: &[u8]) -> Result<ParsedDocument> {
        use calamine::{Reader, Xlsx};
        use std::io::Cursor;

        let cursor = Cursor::new(bytes);
        let mut workbook: Xlsx<_> = Xlsx::new(cursor).context("Failed to parse Excel file")?;

        let mut content = String::new();

        // Iterate through all sheets
        for sheet_name in workbook.sheet_names().to_vec() {
            content.push_str(&format!("## {}\n\n", sheet_name));

            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // Convert each row to text
                for row in range.rows() {
                    let row_text: Vec<String> = row
                        .iter()
                        .map(|cell| format!("{}", cell))
                        .collect();
                    content.push_str(&row_text.join(" | "));
                    content.push('\n');
                }
                content.push('\n');
            }
        }

        Ok(ParsedDocument {
            content: content.trim().to_string(),
            doc_type: DocumentType::Excel,
            page_count: None,
            metadata: DocumentMetadata::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_detection() {
        assert_eq!(
            DocumentType::from_path(Path::new("test.pdf")),
            DocumentType::Pdf
        );
        assert_eq!(
            DocumentType::from_path(Path::new("doc.docx")),
            DocumentType::Docx
        );
        assert_eq!(
            DocumentType::from_path(Path::new("data.xlsx")),
            DocumentType::Excel
        );
    }

    #[test]
    fn test_text_parsing() {
        let text = b"Hello, world!";
        let result = DocumentParser::parse_from_bytes(text, DocumentType::Text).unwrap();
        assert_eq!(result.content, "Hello, world!");
    }
}
