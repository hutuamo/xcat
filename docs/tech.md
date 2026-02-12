# xcat 技术架构

## 概述
xcat 是一个终端 Markdown 预览工具，使用 Rust 实现，支持 macOS 和 Linux。

## 技术栈
- **语言**: Rust (edition 2024)
- **构建**: Cargo
- **终端UI**: ratatui + crossterm
- **Markdown解析**: pulldown-cmark (GFM tables 扩展)
- **字符宽度**: unicode-width (CJK 宽字符支持)

## 架构

### 渲染管线
```
Markdown文件 → pulldown-cmark解析 → Document(Vec<RenderLine>) → ratatui屏幕输出
```

### 模块划分

| 模块 | 文件 | 职责 |
|------|------|------|
| main | main.rs | 参数解析、格式检测、启动Viewer |
| document | document.rs | 核心数据模型定义 |
| format | format/mod.rs | FileFormat trait、格式检测 |
| markdown | format/markdown.rs | pulldown-cmark事件 → Document转换 |
| renderer | renderer.rs | ratatui样式映射、文档绘制、状态栏 |
| viewer | viewer.rs | 终端生命周期、事件循环、按键处理 |

### 数据模型
- `TextStyle`: 位标志样式属性(BOLD/ITALIC/DIM/HEADING/QUOTE/CODE)
- `TextSpan`: 带样式的文本片段 = String + TextStyle
- `RenderLine`: 一行渲染内容 = Vec<TextSpan> + indent
- `Document`: 整个文档 = Vec<RenderLine>

### 可扩展性
通过 `FileFormat` trait 支持多种文件格式：
```rust
pub trait FileFormat {
    fn parse(&self, path: &Path) -> Result<Document, FormatError>;
    fn extensions(&self) -> &[&str];
}
```
新增格式只需在 `format/` 下添加实现并注册到 `detect_format`。

### 依赖
- pulldown-cmark: Cargo 管理
- ratatui + crossterm: Cargo 管理
- unicode-width: Cargo 管理

## 构建
```bash
cargo build --release
```

## 按键映射
| 按键 | 功能 |
|------|------|
| j | 下滚一行 |
| k | 上滚一行 |
| h | 左滚一列 |
| l | 右滚一列 |
| d | 下滚半页 |
| u | 上滚半页 |
| g | 跳到顶部 |
| G | 跳到底部 |
| q/Q | 退出 |
