/// 样式属性位标志
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextStyle(u32);

impl TextStyle {
    pub const NONE: Self = Self(0);
    pub const BOLD: Self = Self(1 << 0);
    pub const ITALIC: Self = Self(1 << 1);
    pub const DIM: Self = Self(1 << 2);
    pub const HEADING: Self = Self(1 << 3);
    pub const QUOTE: Self = Self(1 << 4);
    pub const CODE: Self = Self(1 << 5);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl std::ops::BitOr for TextStyle {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// 带样式的文本片段
#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub style: TextStyle,
}

/// 一行渲染内容
#[derive(Clone, Debug, Default)]
pub struct RenderLine {
    pub spans: Vec<TextSpan>,
    pub indent: u16,
}

/// 文档 = 渲染行列表
#[derive(Clone, Debug, Default)]
pub struct Document {
    pub lines: Vec<RenderLine>,
}
