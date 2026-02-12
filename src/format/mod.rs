pub mod image;
pub mod magic;
pub mod markdown;
pub mod pdf;

use crate::document::Document;
use std::fmt;
use std::path::Path;

/// 文件格式解析 trait（文档类格式）
pub trait FileFormat {
    fn parse(&self, path: &Path) -> Result<Document, FormatError>;
    fn extensions(&self) -> &[&str];
}

/// 格式分类
pub enum FormatKind {
    Document(Box<dyn FileFormat>),
    Image,
}

#[derive(Debug)]
pub enum FormatError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for FormatError {
    fn from(e: std::io::Error) -> Self {
        FormatError::Io(e)
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Io(e) => write!(f, "IO错误: {e}"),
            FormatError::Parse(msg) => write!(f, "解析错误: {msg}"),
        }
    }
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif", "ico",
];

/// 检测文件格式
///
/// 检测策略：
/// 1. 优先使用 magic number（文件签名）检测 - 更可靠
/// 2. 如果 magic number 检测失败，回退到扩展名检测
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回
/// 检测到的格式类型，如果无法识别则返回 None
pub fn detect_format(path: &Path) -> Option<FormatKind> {
    // 1. 优先使用 magic number 检测
    if let Some(detected) = magic::detect_file_format(path) {
        return match detected {
            magic::DetectedFormat::Image(_) => Some(FormatKind::Image),
            magic::DetectedFormat::Document(magic::DocumentFormat::Pdf) => {
                Some(FormatKind::Document(Box::new(pdf::PdfFormat)))
            }
        };
    }

    // 2. 回退到扩展名检测
    detect_format_by_extension(path)
}

/// 根据文件扩展名检测格式（作为 fallback）
fn detect_format_by_extension(path: &Path) -> Option<FormatKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Some(FormatKind::Image);
    }

    let formats: Vec<Box<dyn FileFormat>> = vec![
        Box::new(markdown::MarkdownFormat),
        Box::new(pdf::PdfFormat),
    ];

    formats
        .into_iter()
        .find(|f| f.extensions().contains(&ext.as_str()))
        .map(FormatKind::Document)
}
