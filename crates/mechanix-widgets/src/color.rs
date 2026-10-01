use geometry::Color;
use theme::{ColorRole, ThemeReader};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSource {
    Exact(Color),
    Role(ColorRole),
}

impl ColorSource {
    #[inline]
    pub fn resolve(self, reader: &impl ThemeReader) -> Color {
        match self {
            Self::Exact(color) => color,
            Self::Role(role) => reader.color(role),
        }
    }
}

impl From<Color> for ColorSource {
    #[inline]
    fn from(color: Color) -> Self {
        Self::Exact(color)
    }
}

impl From<ColorRole> for ColorSource {
    #[inline]
    fn from(role: ColorRole) -> Self {
        Self::Role(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_conversions() {
        let exact: ColorSource = Color::WHITE.into();
        assert_eq!(exact, ColorSource::Exact(Color::WHITE));

        let role: ColorSource = ColorRole::Primary.into();
        assert_eq!(role, ColorSource::Role(ColorRole::Primary));
    }
}
