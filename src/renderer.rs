use crate::document::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// TextStyle → ratatui Style
pub fn to_style(ts: TextStyle) -> Style {
    let mut s = Style::default();

    if ts.contains(TextStyle::HEADING) {
        s = s.fg(Color::Cyan);
    } else if ts.contains(TextStyle::QUOTE) {
        s = s.fg(Color::Yellow);
    } else if ts.contains(TextStyle::CODE) {
        s = s.fg(Color::Green);
    }

    if ts.contains(TextStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if ts.contains(TextStyle::ITALIC) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }
    if ts.contains(TextStyle::DIM) {
        s = s.add_modifier(Modifier::DIM);
    }

    s
}

/// 绘制文档内容
pub fn draw_document(
    frame: &mut Frame,
    doc: &Document,
    area: Rect,
    top_line: usize,
    _left_col: usize,
    cursor_line: usize,
) {
    let rows = area.height as usize;

    for row in 0..rows {
        let doc_line_idx = top_line + row;
        let y = area.y + row as u16;
        let line_area = Rect::new(area.x, y, area.width, 1);
        let is_cursor = doc_line_idx == cursor_line;

        if doc_line_idx >= doc.lines.len() {
            let tilde = Line::from(Span::styled(
                "~",
                Style::default().add_modifier(Modifier::DIM),
            ));
            frame.render_widget(Paragraph::new(tilde), line_area);
            continue;
        }

        // 当前行高亮背景
        if is_cursor {
            let bg = Paragraph::new("").style(Style::default().bg(Color::DarkGray));
            frame.render_widget(bg, line_area);
        }

        let render_line = &doc.lines[doc_line_idx];
        let indent = render_line.indent as usize;
        let mut spans: Vec<Span> = Vec::new();

        if indent > 0 {
            spans.push(Span::raw(" ".repeat(indent)));
        }

        for text_span in &render_line.spans {
            let mut style = to_style(text_span.style);
            if is_cursor {
                style = style.bg(Color::DarkGray);
            }
            spans.push(Span::styled(text_span.text.clone(), style));
        }

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), line_area);
    }
}

/// 绘制底部状态栏
pub fn draw_status_bar(
    frame: &mut Frame,
    area: Rect,
    filename: &str,
    current_line: usize,
    total_lines: usize,
) {
    let width = area.width as usize;
    let left = format!(" {}", filename);
    let right = format!("{}/{} ", current_line + 1, total_lines);
    let padding = width.saturating_sub(left.len() + right.len());
    let bar_text = format!("{}{}{}", left, " ".repeat(padding), right);

    let style = Style::default().fg(Color::Black).bg(Color::White);
    let bar = Line::from(Span::styled(bar_text, style));
    frame.render_widget(Paragraph::new(bar), area);
}
