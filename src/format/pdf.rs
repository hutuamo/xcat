use crate::document::*;
use crate::format::{FileFormat, FormatError};
use std::path::Path;

pub struct PdfFormat;

impl FileFormat for PdfFormat {
    fn parse(&self, path: &Path) -> Result<Document, FormatError> {
        let pages = pdf_extract::extract_text_by_pages(path)
            .map_err(|e| FormatError::Parse(format!("PDF 解析失败: {e}")))?;

        let mut doc = Document::default();

        for (i, page_text) in pages.iter().enumerate() {
            if i > 0 {
                doc.lines.push(RenderLine::default());
                doc.lines.push(RenderLine {
                    spans: vec![TextSpan {
                        text: format!("── 第 {} 页 ──", i + 1),
                        style: TextStyle::DIM,
                    }],
                    indent: 0,
                });
                doc.lines.push(RenderLine::default());
            }

            for line in page_text.lines() {
                doc.lines.push(RenderLine {
                    spans: vec![TextSpan {
                        text: line.to_string(),
                        style: TextStyle::NONE,
                    }],
                    indent: 0,
                });
            }
        }

        Ok(doc)
    }

    fn extensions(&self) -> &[&str] {
        &["pdf"]
    }
}
