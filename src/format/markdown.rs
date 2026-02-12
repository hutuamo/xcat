use crate::document::*;
use crate::format::{FileFormat, FormatError};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::path::Path;
use unicode_width::UnicodeWidthStr;

pub struct MarkdownFormat;

impl FileFormat for MarkdownFormat {
    fn parse(&self, path: &Path) -> Result<Document, FormatError> {
        let content = std::fs::read_to_string(path)?;
        Ok(parse_markdown(&content))
    }

    fn extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mkd"]
    }
}

struct ListContext {
    is_ordered: bool,
    item_index: u64,
}

struct ParseState {
    doc: Document,
    current_line: RenderLine,
    current_style: TextStyle,
    indent_level: u16,
    in_code_block: bool,
    line_has_content: bool,

    // 列表嵌套栈
    list_stack: Vec<ListContext>,

    // 表格状态
    in_table: bool,
    in_table_head: bool,
    in_table_cell: bool,
    table_rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell_text: String,
}

impl ParseState {
    fn new() -> Self {
        Self {
            doc: Document::default(),
            current_line: RenderLine::default(),
            current_style: TextStyle::NONE,
            indent_level: 0,
            in_code_block: false,
            line_has_content: false,
            list_stack: Vec::new(),
            in_table: false,
            in_table_head: false,
            in_table_cell: false,
            table_rows: Vec::new(),
            current_row: Vec::new(),
            current_cell_text: String::new(),
        }
    }

    fn flush_line(&mut self) {
        self.current_line.indent = self.indent_level;
        let line = std::mem::take(&mut self.current_line);
        self.doc.lines.push(line);
        self.line_has_content = false;
    }

    fn add_empty_line(&mut self) {
        self.doc.lines.push(RenderLine::default());
    }

    fn push_span(&mut self, text: String, style: TextStyle) {
        if !text.is_empty() {
            self.current_line.spans.push(TextSpan { text, style });
            self.line_has_content = true;
        }
    }

    fn render_table(&mut self) {
        if self.table_rows.is_empty() {
            return;
        }

        let num_cols = self.table_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        if num_cols == 0 {
            return;
        }

        // 计算每列最大显示宽度
        let mut col_widths = vec![0usize; num_cols];
        for row in &self.table_rows {
            for (i, cell) in row.iter().enumerate() {
                let w = UnicodeWidthStr::width(cell.as_str());
                col_widths[i] = col_widths[i].max(w);
            }
        }

        for (r, row) in self.table_rows.iter().enumerate() {
            let is_header = r == 0;
            let attrs = if is_header {
                TextStyle::BOLD | TextStyle::HEADING
            } else {
                TextStyle::NONE
            };

            let mut line = RenderLine {
                indent: self.indent_level,
                ..Default::default()
            };

            for c in 0..num_cols {
                if c > 0 {
                    line.spans.push(TextSpan {
                        text: "  ".into(),
                        style: TextStyle::NONE,
                    });
                }
                let cell_text = row.get(c).map(|s| s.as_str()).unwrap_or("");
                let padded = pad_to_width(cell_text, col_widths[c]);
                line.spans.push(TextSpan {
                    text: padded,
                    style: attrs,
                });
            }
            self.doc.lines.push(line);

            // 表头后插入分隔线
            if is_header {
                let mut sep = RenderLine {
                    indent: self.indent_level,
                    ..Default::default()
                };
                for c in 0..num_cols {
                    if c > 0 {
                        sep.spans.push(TextSpan {
                            text: "  ".into(),
                            style: TextStyle::NONE,
                        });
                    }
                    let dash = "─".repeat(col_widths[c]);
                    sep.spans.push(TextSpan {
                        text: dash,
                        style: TextStyle::DIM,
                    });
                }
                self.doc.lines.push(sep);
            }
        }

        self.add_empty_line();
        self.table_rows.clear();
    }
}

fn pad_to_width(text: &str, target: usize) -> String {
    let current = UnicodeWidthStr::width(text);
    if current >= target {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(target - current))
    }
}

pub fn parse_markdown(content: &str) -> Document {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut state = ParseState::new();

    for event in parser {
        match event {
            // === Block 级 ===
            Event::Start(Tag::Heading { .. }) => {
                state.current_style.insert(TextStyle::BOLD | TextStyle::HEADING);
            }
            Event::End(TagEnd::Heading(_)) => {
                state.current_style.remove(TextStyle::BOLD | TextStyle::HEADING);
                state.flush_line();
                state.add_empty_line();
            }

            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                if !state.in_table {
                    state.flush_line();
                    state.add_empty_line();
                }
            }

            Event::Start(Tag::CodeBlock(_)) => {
                state.in_code_block = true;
                state.current_style.insert(TextStyle::CODE);
                state.flush_line();
                state.push_span("───".into(), TextStyle::DIM);
                state.flush_line();
            }
            Event::End(TagEnd::CodeBlock) => {
                if state.line_has_content {
                    state.flush_line();
                }
                state.push_span("───".into(), TextStyle::DIM);
                state.flush_line();
                state.in_code_block = false;
                state.current_style.remove(TextStyle::CODE);
            }

            Event::Start(Tag::List(first_index)) => {
                state.list_stack.push(ListContext {
                    is_ordered: first_index.is_some(),
                    item_index: first_index.unwrap_or(1).saturating_sub(1),
                });
                state.indent_level += 4;
            }
            Event::End(TagEnd::List(_)) => {
                state.list_stack.pop();
                state.indent_level = state.indent_level.saturating_sub(4);
                if state.indent_level == 0 {
                    state.add_empty_line();
                }
            }

            Event::Start(Tag::Item) => {
                if let Some(ctx) = state.list_stack.last_mut() {
                    if ctx.is_ordered {
                        ctx.item_index += 1;
                        let marker = format!("{}. ", ctx.item_index);
                        state.push_span(marker, state.current_style);
                    } else {
                        state.push_span("• ".into(), state.current_style);
                    }
                }
            }
            Event::End(TagEnd::Item) => {
                state.flush_line();
            }

            Event::Start(Tag::BlockQuote(_)) => {
                state.indent_level += 2;
                state.current_style.insert(TextStyle::QUOTE);
                state.push_span("│ ".into(), TextStyle::QUOTE | TextStyle::DIM);
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                state.indent_level = state.indent_level.saturating_sub(2);
                state.current_style.remove(TextStyle::QUOTE);
                state.flush_line();
                state.add_empty_line();
            }

            Event::Rule => {
                state.flush_line();
                state.push_span("────────────────────────────────".into(), TextStyle::DIM);
                state.flush_line();
            }

            // === 表格 ===
            Event::Start(Tag::Table(_)) => {
                state.in_table = true;
                state.table_rows.clear();
            }
            Event::End(TagEnd::Table) => {
                state.in_table = false;
                state.render_table();
            }
            Event::Start(Tag::TableHead) => {
                state.in_table_head = true;
            }
            Event::End(TagEnd::TableHead) => {
                state.in_table_head = false;
            }
            Event::Start(Tag::TableRow) => {
                state.current_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                let row = std::mem::take(&mut state.current_row);
                state.table_rows.push(row);
            }
            Event::Start(Tag::TableCell) => {
                state.in_table_cell = true;
                state.current_cell_text.clear();
            }
            Event::End(TagEnd::TableCell) => {
                state.in_table_cell = false;
                let text = std::mem::take(&mut state.current_cell_text);
                state.current_row.push(text);
            }

            // === Inline 级 ===
            Event::Start(Tag::Strong) => {
                state.current_style.insert(TextStyle::BOLD);
            }
            Event::End(TagEnd::Strong) => {
                state.current_style.remove(TextStyle::BOLD);
            }
            Event::Start(Tag::Emphasis) => {
                state.current_style.insert(TextStyle::ITALIC);
            }
            Event::End(TagEnd::Emphasis) => {
                state.current_style.remove(TextStyle::ITALIC);
            }

            // 行内代码（叶子事件）
            Event::Code(text) => {
                if state.in_table_cell {
                    state.current_cell_text.push_str(&text);
                } else {
                    state.push_span(
                        text.into_string(),
                        state.current_style | TextStyle::CODE,
                    );
                }
            }

            // === 文本 ===
            Event::Text(text) => {
                if state.in_table_cell {
                    state.current_cell_text.push_str(&text);
                } else if state.in_code_block {
                    // 代码块按换行拆分
                    let mut first = true;
                    for line in text.split('\n') {
                        if !first {
                            state.flush_line();
                        }
                        if !line.is_empty() {
                            state.push_span(line.to_string(), state.current_style);
                        }
                        first = false;
                    }
                } else {
                    state.push_span(text.into_string(), state.current_style);
                }
            }

            Event::SoftBreak => {
                if !state.in_table_cell {
                    state.push_span(" ".into(), state.current_style);
                }
            }
            Event::HardBreak => {
                if !state.in_table_cell {
                    state.flush_line();
                }
            }

            _ => {}
        }
    }

    if state.line_has_content {
        state.flush_line();
    }

    state.doc
}
