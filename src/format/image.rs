use std::io;
use std::path::Path;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// 在终端中显示图片，按任意键退出
pub fn display(path: &Path) -> io::Result<()> {
    let conf = viuer::Config {
        absolute_offset: false,
        ..Default::default()
    };

    viuer::print_from_file(path, &conf).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("图片显示失败: {e}"))
    })?;

    // 等待按键退出
    enable_raw_mode()?;
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                _ => break,
            }
        }
    }
    disable_raw_mode()?;

    Ok(())
}
