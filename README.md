# xcat

终端文件预览工具，支持 Markdown 渲染和图片显示。

## 核心功能

- **Markdown 预览** — 在终端中渲染标题、粗体、斜体、代码块、列表、引用、表格等元素，支持 GFM 规范
- **图片显示** — 支持 PNG、JPEG、GIF、BMP、WebP、TIFF、ICO 格式
- **Vim 风格导航** — `hjkl` 移动光标，`d/u` 翻页，`g/G` 跳转首尾，`q` 退出
- **智能格式识别** — 优先通过文件魔数（magic number）检测格式，扩展名作为回退
- **可扩展架构** — 通过 `FileFormat` trait 轻松添加新格式支持

## 构建

```bash
# 编译
cargo build --release

# 运行
cargo run --release -- <file>

# 示例
cargo run --release -- README.md
```

## 依赖

| 库 | 用途 |
|---|---|
| crossterm | 终端原始模式与事件处理 |
| ratatui | TUI 渲染框架 |
| pulldown-cmark | GFM Markdown 解析 |
| viuer | 终端图片显示 |
| unicode-width | CJK 宽字符支持 |

## 项目结构

```
src/
├── main.rs           # 入口，CLI 参数解析与格式分发
├── document.rs       # 数据模型（TextStyle, TextSpan, RenderLine, Document）
├── viewer.rs         # 终端事件循环与 Vim 导航
├── renderer.rs       # ratatui 样式映射与屏幕绘制
└── format/
    ├── mod.rs        # FileFormat trait 与格式检测
    ├── magic.rs      # 文件魔数签名识别
    ├── markdown.rs   # Markdown 解析器
    └── image.rs      # 图片显示
```

## 键位

| 键 | 功能 |
|---|---|
| `j` / `k` | 上 / 下移一行 |
| `h` / `l` | 左 / 右移一列 |
| `d` / `u` | 下 / 上翻半页 |
| `g` / `G` | 跳转到文档首 / 尾 |
| `q` / `Q` | 退出 |
