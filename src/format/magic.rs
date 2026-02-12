//! Magic number / file signature 检测模块
//!
//! 通过读取文件头部的 magic bytes 来检测文件格式，比扩展名检测更可靠。

use std::path::Path;

/// 检测到的文件格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedFormat {
    /// 图片格式
    Image(ImageFormat),
    /// 文档格式
    Document(DocumentFormat),
}

/// 支持的图片格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    WebP,
    Tiff,
    Ico,
}

/// 支持的文档格式（为未来扩展准备）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Pdf,
}

/// 文件签名定义
struct FileSignature {
    /// Magic bytes（可能包含通配符）
    magic: &'static [u8],
    /// 起始偏移量
    offset: usize,
    /// 掩码（None 表示精确匹配，Some 表示按位与后比较）
    mask: Option<&'static [u8]>,
    /// 对应的格式
    format: DetectedFormat,
}

impl FileSignature {
    /// 创建精确匹配的签名
    const fn exact(magic: &'static [u8], offset: usize, format: DetectedFormat) -> Self {
        Self {
            magic,
            offset,
            mask: None,
            format,
        }
    }

    /// 创建带掩码的签名
    const fn with_mask(
        magic: &'static [u8],
        offset: usize,
        mask: &'static [u8],
        format: DetectedFormat,
    ) -> Self {
        Self {
            magic,
            offset,
            mask: Some(mask),
            format,
        }
    }

    /// 检查数据是否匹配此签名
    fn matches(&self, data: &[u8]) -> bool {
        let start = self.offset;
        let end = start + self.magic.len();

        if data.len() < end {
            return false;
        }

        let slice = &data[start..end];

        match self.mask {
            Some(mask) => {
                // 带掩码的比较
                for i in 0..self.magic.len() {
                    if (slice[i] & mask[i]) != self.magic[i] {
                        return false;
                    }
                }
                true
            }
            None => {
                // 精确匹配
                slice == self.magic
            }
        }
    }
}

/// 所有支持的文件签名列表
const SIGNATURES: &[FileSignature] = &[
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    FileSignature::exact(
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        0,
        DetectedFormat::Image(ImageFormat::Png),
    ),
    // JPEG: FF D8 FF
    FileSignature::exact(
        &[0xFF, 0xD8, 0xFF],
        0,
        DetectedFormat::Image(ImageFormat::Jpeg),
    ),
    // GIF: "GIF8" (47 49 46 38)
    FileSignature::exact(
        b"GIF8",
        0,
        DetectedFormat::Image(ImageFormat::Gif),
    ),
    // BMP: "BM" (42 4D)
    FileSignature::exact(
        b"BM",
        0,
        DetectedFormat::Image(ImageFormat::Bmp),
    ),
    // WebP: RIFF....WEBP
    // 需要检查 RIFF 在偏移 0，WEBP 在偏移 8
    FileSignature::exact(
        b"RIFF",
        0,
        DetectedFormat::Image(ImageFormat::WebP),
    ),
    // TIFF (little-endian): 49 49 2A 00
    FileSignature::exact(
        &[0x49, 0x49, 0x2A, 0x00],
        0,
        DetectedFormat::Image(ImageFormat::Tiff),
    ),
    // TIFF (big-endian): 4D 4D 00 2A
    FileSignature::exact(
        &[0x4D, 0x4D, 0x00, 0x2A],
        0,
        DetectedFormat::Image(ImageFormat::Tiff),
    ),
    // ICO: 00 00 01 00
    FileSignature::exact(
        &[0x00, 0x00, 0x01, 0x00],
        0,
        DetectedFormat::Image(ImageFormat::Ico),
    ),
    // PDF: "%PDF"
    FileSignature::exact(
        b"%PDF",
        0,
        DetectedFormat::Document(DocumentFormat::Pdf),
    ),
];

/// 需要读取的最大字节数（用于检测）
const MAX_HEADER_SIZE: usize = 32;

/// 通过 magic number 检测文件格式
///
/// # 参数
/// - `data`: 文件头部的字节（至少 MAX_HEADER_SIZE 字节）
///
/// # 返回
/// 检测到的格式，如果无法识别则返回 None
pub fn detect_by_magic(data: &[u8]) -> Option<DetectedFormat> {
    for sig in SIGNATURES {
        if sig.matches(data) {
            // WebP 需要额外验证
            if let DetectedFormat::Image(ImageFormat::WebP) = sig.format {
                if data.len() >= 12 && &data[8..12] == b"WEBP" {
                    return Some(sig.format);
                }
                continue;
            }
            return Some(sig.format);
        }
    }
    None
}

/// 从文件路径检测格式
///
/// # 参数
/// - `path`: 文件路径
///
/// # 返回
/// 检测到的格式，如果无法识别或读取失败则返回 None
pub fn detect_file_format(path: &Path) -> Option<DetectedFormat> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).ok()?;
    let mut buffer = [0u8; MAX_HEADER_SIZE];
    let bytes_read = file.read(&mut buffer).ok()?;

    detect_by_magic(&buffer[..bytes_read])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_detection() {
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(
            detect_by_magic(&png_header),
            Some(DetectedFormat::Image(ImageFormat::Png))
        );
    }

    #[test]
    fn test_jpeg_detection() {
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(
            detect_by_magic(&jpeg_header),
            Some(DetectedFormat::Image(ImageFormat::Jpeg))
        );
    }

    #[test]
    fn test_gif_detection() {
        let gif_header = b"GIF89a";
        assert_eq!(
            detect_by_magic(gif_header),
            Some(DetectedFormat::Image(ImageFormat::Gif))
        );
    }

    #[test]
    fn test_bmp_detection() {
        let bmp_header = b"BM\x00\x00\x00\x00";
        assert_eq!(
            detect_by_magic(bmp_header),
            Some(DetectedFormat::Image(ImageFormat::Bmp))
        );
    }

    #[test]
    fn test_webp_detection() {
        // RIFF....WEBP
        let webp_header = b"RIFF\x00\x00\x00\x00WEBP";
        assert_eq!(
            detect_by_magic(webp_header),
            Some(DetectedFormat::Image(ImageFormat::WebP))
        );
    }

    #[test]
    fn test_tiff_le_detection() {
        let tiff_header = [0x49, 0x49, 0x2A, 0x00];
        assert_eq!(
            detect_by_magic(&tiff_header),
            Some(DetectedFormat::Image(ImageFormat::Tiff))
        );
    }

    #[test]
    fn test_tiff_be_detection() {
        let tiff_header = [0x4D, 0x4D, 0x00, 0x2A];
        assert_eq!(
            detect_by_magic(&tiff_header),
            Some(DetectedFormat::Image(ImageFormat::Tiff))
        );
    }

    #[test]
    fn test_ico_detection() {
        let ico_header = [0x00, 0x00, 0x01, 0x00, 0x01, 0x00];
        assert_eq!(
            detect_by_magic(&ico_header),
            Some(DetectedFormat::Image(ImageFormat::Ico))
        );
    }

    #[test]
    fn test_pdf_detection() {
        let pdf_header = b"%PDF-1.4";
        assert_eq!(
            detect_by_magic(pdf_header),
            Some(DetectedFormat::Document(DocumentFormat::Pdf))
        );
    }

    #[test]
    fn test_unknown_format() {
        let unknown = b"Hello, World!";
        assert_eq!(detect_by_magic(unknown), None);
    }
}