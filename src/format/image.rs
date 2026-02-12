use std::io;
use std::path::Path;

/// 在终端中直接显示图片
pub fn display(path: &Path) -> io::Result<()> {
    let conf = viuer::Config {
        absolute_offset: false,
        ..Default::default()
    };

    viuer::print_from_file(path, &conf).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("图片显示失败: {e}"))
    })?;

    Ok(())
}
