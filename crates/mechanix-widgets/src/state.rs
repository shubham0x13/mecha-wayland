use geometry::Color;
use theme::{ColorRole, ColorScheme};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WidgetState {
    #[default]
    Enabled,
    Hovered,
    Focused,
    Pressed,
    Disabled,
}

impl WidgetState {
    #[inline]
    pub fn is_disabled(self) -> bool {
        matches!(self, WidgetState::Disabled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateLayer {
    pub role: ColorRole,
    pub opacity: f32,
}

impl StateLayer {
    #[inline]
    pub const fn new(role: ColorRole, opacity: f32) -> Self {
        Self { role, opacity }
    }

    #[inline]
    pub fn apply(self, base_color: Color, scheme: &ColorScheme) -> Color {
        let tint = self.role.resolve(scheme);
        let overlay = Color::rgba(tint.r, tint.g, tint.b, self.opacity);
        overlay.over(base_color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_state_disabled() {
        assert!(!WidgetState::Enabled.is_disabled());
        assert!(WidgetState::Disabled.is_disabled());
    }

    #[test]
    fn state_layer_apply() {
        let scheme = ColorScheme::baseline_dark();
        let layer = StateLayer::new(ColorRole::Primary, 0.08);
        let blended = layer.apply(Color::BLACK, &scheme);
        assert_ne!(blended, Color::BLACK);
    }
}
