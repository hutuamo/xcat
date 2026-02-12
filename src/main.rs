mod document;
mod format;
mod renderer;
mod viewer;

use format::FormatKind;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("用法: {} <file>", args[0]);
        process::exit(1);
    }

    let path = Path::new(&args[1]);

    if !path.exists() {
        eprintln!("错误: 文件不存在 - {}", path.display());
        process::exit(1);
    }

    if !path.is_file() {
        eprintln!("错误: 不是普通文件 - {}", path.display());
        process::exit(1);
    }

    let format_kind = match format::detect_format(path) {
        Some(k) => k,
        None => {
            eprintln!("错误: 不支持的文件格式 - {}", path.display());
            process::exit(1);
        }
    };

    match format_kind {
        FormatKind::Image => {
            // 图片：直接模式
            if let Err(e) = format::image::display(path) {
                eprintln!("错误: {} - {}", e, path.display());
                process::exit(1);
            }
        }
        FormatKind::Text => {
            // 纯文本：直接模式
            if let Err(e) = format::text::display(path) {
                eprintln!("错误: {} - {}", e, path.display());
                process::exit(1);
            }
        }
        FormatKind::Document(formatter) => {
            // Markdown/PDF：预览模式
            let doc = match formatter.parse(path) {
                Ok(doc) => doc,
                Err(e) => {
                    eprintln!("错误: {} - {}", e, path.display());
                    process::exit(1);
                }
            };

            if doc.lines.is_empty() {
                eprintln!("错误: 文件为空或无法解析 - {}", path.display());
                process::exit(1);
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let mut viewer = viewer::Viewer::new(doc, filename);
            if let Err(e) = viewer.run() {
                eprintln!("错误: {}", e);
                process::exit(1);
            }
        }
    }
}
