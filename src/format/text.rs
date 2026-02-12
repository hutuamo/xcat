//! 纯文本格式处理模块
//!
//! 文本文件使用直接模式显示（不进入 TUI 预览）

use std::fs;
use std::io;
use std::path::Path;

/// 直接在终端打印文本文件内容
pub fn display(path: &Path) -> io::Result<()> {
    let content = fs::read_to_string(path)?;
    println!("{}", content);
    Ok(())
}