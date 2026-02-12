use crate::document::Document;
use crate::renderer;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::io::{self, stdout};

pub struct Viewer {
    doc: Document,
    filename: String,
    top_line: usize,
    left_col: usize,
    cursor_line: usize,
}

impl Viewer {
    pub fn new(doc: Document, filename: String) -> Self {
        Self {
            doc,
            filename,
            top_line: 0,
            left_col: 0,
            cursor_line: 0,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let result = self.event_loop(&mut terminal);

        terminal.show_cursor()?;
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                let size = frame.area();
                let content_area = Rect::new(0, 0, size.width, size.height.saturating_sub(1));
                let status_area =
                    Rect::new(0, size.height.saturating_sub(1), size.width, 1);

                renderer::draw_document(
                    frame,
                    &self.doc,
                    content_area,
                    self.top_line,
                    self.left_col,
                    self.cursor_line,
                );
                renderer::draw_status_bar(
                    frame,
                    status_area,
                    &self.filename,
                    self.cursor_line,
                    self.doc.lines.len(),
                );
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    _ => self.handle_key(key.code),
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) {
        let page = self.content_rows();
        let max_line = self.doc.lines.len().saturating_sub(1);

        match code {
            KeyCode::Char('j') => {
                self.cursor_line = (self.cursor_line + 1).min(max_line);
            }
            KeyCode::Char('k') => {
                self.cursor_line = self.cursor_line.saturating_sub(1);
            }
            KeyCode::Char('l') => self.left_col += 1,
            KeyCode::Char('h') => self.left_col = self.left_col.saturating_sub(1),
            KeyCode::Char('d') => {
                self.cursor_line = (self.cursor_line + page / 2).min(max_line);
            }
            KeyCode::Char('u') => {
                self.cursor_line = self.cursor_line.saturating_sub(page / 2);
            }
            KeyCode::Char('g') => {
                self.cursor_line = 0;
                self.left_col = 0;
            }
            KeyCode::Char('G') => {
                self.cursor_line = max_line;
            }
            _ => {}
        }

        self.scroll_to_cursor();
    }

    /// 确保 cursor_line 在可见区域内，必要时滚动
    fn scroll_to_cursor(&mut self) {
        let page = self.content_rows();
        if self.cursor_line < self.top_line {
            self.top_line = self.cursor_line;
        } else if self.cursor_line >= self.top_line + page {
            self.top_line = self.cursor_line.saturating_sub(page.saturating_sub(1));
        }
    }

    /// 内容区域行数（总行数减去状态栏）
    fn content_rows(&self) -> usize {
        let (_, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        (rows as usize).saturating_sub(1)
    }
}
