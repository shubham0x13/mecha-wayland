//! `Button`: a pressable, focusable container — the native interactive
//! primitive.
//!
//! Takes **explicit** paint values (colour, border, radii, state quads).
//!
//! # State machine
//!
//! The widget embeds an [`Enter`]/[`Exit`]/[`Press`]/[`Release`] handler
//! set that transitions [`WidgetState`] automatically. Disabled widgets
//! ignore all transitions. Use [`ButtonContext::set_quad`] (or the
//! fine-grained setters) inside an event handler to change the look for
//! each state.
//!
//! ```text
//!  Enabled ──Enter──▶ Hovered ──Press──▶ Pressed
//!     ▲                  ▲                  │
//!     └──────Exit─────────┴───Release────────┘
//! ```
//!
//! # Quick start
//!
//! ```
//! use widgets::prelude::*;
//! use geometry::Color;
//!
//! let btn = button()
//!     .background(Color::rgba(0.4, 0.3, 0.6, 1.0))
//!     .radius(20.0);
//! ```

use crate::state::WidgetState;
use app::{Build, Context, Handle, Spawner, Widget};
use geometry::{Color, Corners, Insets};
use interactivity::prelude::{Enter, Exit, Press, Release};
use layout::LayoutStyle;
use paint::{Paint, PaintContext, Quad};

/// The configurable properties of a native button.
///
/// Serves as the single source of truth for both [`ButtonBuilder`] and [`Button`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonProps {
    /// The default quad painted when the button is in the [`WidgetState::Enabled`] state.
    pub quad: Quad,
    /// Optional quad override for [`WidgetState::Hovered`]. Falls back to `quad` if `None`.
    pub hover_quad: Option<Quad>,
    /// Optional quad override for [`WidgetState::Pressed`]. Falls back to `hover_quad` or `quad` if `None`.
    pub pressed_quad: Option<Quad>,
    /// Optional quad override for [`WidgetState::Focused`]. Falls back to `quad` if `None`.
    pub focused_quad: Option<Quad>,
    /// Optional quad override for [`WidgetState::Disabled`]. Falls back to `quad` if `None`.
    pub disabled_quad: Option<Quad>,
    /// Current interaction state.
    pub state: WidgetState,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            quad: Quad::default(),
            hover_quad: None,
            pressed_quad: None,
            focused_quad: None,
            disabled_quad: None,
            state: WidgetState::Enabled,
        }
    }
}

impl ButtonProps {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the active [`Quad`] for the button's current [`WidgetState`].
    pub fn active_quad(&self) -> Quad {
        match self.state {
            WidgetState::Hovered => self.hover_quad.unwrap_or(self.quad),
            WidgetState::Pressed => self.pressed_quad.or(self.hover_quad).unwrap_or(self.quad),
            WidgetState::Focused => self.focused_quad.unwrap_or(self.quad),
            WidgetState::Disabled => self.disabled_quad.unwrap_or(self.quad),
            WidgetState::Enabled => self.quad,
        }
    }

    /// Returns `true` if the button is currently in the [`WidgetState::Disabled`] state.
    #[inline]
    pub const fn is_disabled(&self) -> bool {
        self.state.is_disabled()
    }
}

/// A pressable, focusable container with an explicit [`Quad`] background.
///
/// Wraps [`ButtonProps`] and implements [`std::ops::Deref`] to [`ButtonProps`]
/// for direct property access.
pub struct Button {
    pub props: ButtonProps,
}

impl std::ops::Deref for Button {
    type Target = ButtonProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl std::ops::DerefMut for Button {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

/// Construct a native [`Button`].
///
/// Defaults: no background, no border, `row().center()` layout,
/// [`Enabled`](WidgetState::Enabled) state.
pub fn button() -> ButtonBuilder {
    ButtonBuilder::new()
}

/// Construct a native [`Button`] with pre-configured [`ButtonProps`].
pub fn button_with_props(props: ButtonProps) -> ButtonBuilder {
    ButtonBuilder::from_props(props)
}

/// Builder for a native [`Button`].
pub struct ButtonBuilder {
    pub props: ButtonProps,
    pub style: LayoutStyle,
}

impl std::ops::Deref for ButtonBuilder {
    type Target = ButtonProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl std::ops::DerefMut for ButtonBuilder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            props: ButtonProps::default(),
            style: LayoutStyle::default().row().center(),
        }
    }

    pub fn from_props(props: ButtonProps) -> Self {
        Self {
            props,
            style: LayoutStyle::default().row().center(),
        }
    }

    /// Override the entire layout style.
    pub fn style(mut self, style: LayoutStyle) -> Self {
        self.style = style;
        self
    }

    /// Background colour for the enabled/default state.
    pub fn background(mut self, color: Color) -> Self {
        self.props.quad.color = color;
        self
    }

    /// Background colour when hovered.
    pub fn hover_background(mut self, color: Color) -> Self {
        let mut q = self.props.hover_quad.unwrap_or(self.props.quad);
        q.color = color;
        self.props.hover_quad = Some(q);
        self
    }

    /// Background colour when pressed.
    pub fn pressed_background(mut self, color: Color) -> Self {
        let mut q = self.props.pressed_quad.unwrap_or(self.props.quad);
        q.color = color;
        self.props.pressed_quad = Some(q);
        self
    }

    /// Background colour when disabled.
    pub fn disabled_background(mut self, color: Color) -> Self {
        let mut q = self.props.disabled_quad.unwrap_or(self.props.quad);
        q.color = color;
        self.props.disabled_quad = Some(q);
        self
    }

    /// Uniform corner radius across all state quads.
    pub fn radius(mut self, radius: f32) -> Self {
        let radii = Corners::all(radius);
        self.props.quad.radii = radii;
        if let Some(q) = &mut self.props.hover_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.pressed_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.disabled_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.focused_quad {
            q.radii = radii;
        }
        self
    }

    /// Per-corner radii across all state quads.
    pub fn radii(mut self, radii: Corners<f32>) -> Self {
        self.props.quad.radii = radii;
        if let Some(q) = &mut self.props.hover_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.pressed_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.disabled_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut self.props.focused_quad {
            q.radii = radii;
        }
        self
    }

    /// Border (width + colour) on the default quad.
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.props.quad.border = Insets::all(width);
        self.props.quad.border_color = color;
        self
    }

    /// Border on the focused quad.
    pub fn focus_border(mut self, width: f32, color: Color) -> Self {
        let mut q = self.props.focused_quad.unwrap_or(self.props.quad);
        q.border = Insets::all(width);
        q.border_color = color;
        self.props.focused_quad = Some(q);
        self
    }

    /// Replace the entire default `Quad`.
    pub fn quad(mut self, quad: Quad) -> Self {
        self.props.quad = quad;
        self
    }

    /// Replace the entire hover `Quad`.
    pub fn hover_quad(mut self, quad: Quad) -> Self {
        self.props.hover_quad = Some(quad);
        self
    }

    /// Replace the entire pressed `Quad`.
    pub fn pressed_quad(mut self, quad: Quad) -> Self {
        self.props.pressed_quad = Some(quad);
        self
    }

    /// Replace the entire disabled `Quad`.
    pub fn disabled_quad(mut self, quad: Quad) -> Self {
        self.props.disabled_quad = Some(quad);
        self
    }

    /// Replace the entire focused `Quad`.
    pub fn focused_quad(mut self, quad: Quad) -> Self {
        self.props.focused_quad = Some(quad);
        self
    }

    /// Initial interaction state.
    pub fn state(mut self, state: WidgetState) -> Self {
        self.props.state = state;
        self
    }

    /// Shorthand: start in `Disabled` or `Enabled`.
    pub fn disabled(self, disabled: bool) -> Self {
        self.state(if disabled {
            WidgetState::Disabled
        } else {
            WidgetState::Enabled
        })
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Build for ButtonBuilder {
    type Widget = Button;
}

impl Widget for Button {
    type Builder = ButtonBuilder;

    fn build(b: ButtonBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        *s.component_mut::<LayoutStyle>(me).unwrap() = b.style;
        *s.component_mut::<Paint>(me).unwrap() = Paint::Quad(b.props.active_quad());

        // ── State machine ─────────────────────────────────────────────────
        // Always registered so the button stays interactive even after a
        // Disabled → Enabled transition at runtime. Each handler guards
        // against the current state before transitioning.
        s.on::<Enter>(me, |ctx: &mut Context<'_, Button>, _| {
            if ctx.me().state == WidgetState::Enabled {
                ctx.set_state(WidgetState::Hovered);
            }
        });
        s.on::<Exit>(me, |ctx: &mut Context<'_, Button>, _| {
            if matches!(ctx.me().state, WidgetState::Hovered | WidgetState::Pressed) {
                ctx.set_state(WidgetState::Enabled);
            }
        });
        s.on::<Press>(me, |ctx: &mut Context<'_, Button>, _| {
            if matches!(
                ctx.me().state,
                WidgetState::Enabled | WidgetState::Hovered | WidgetState::Focused
            ) {
                ctx.set_state(WidgetState::Pressed);
            }
        });
        s.on::<Release>(me, |ctx: &mut Context<'_, Button>, _| {
            if ctx.me().state == WidgetState::Pressed {
                ctx.set_state(WidgetState::Hovered);
            }
        });

        Button { props: b.props }
    }
}

/// Post-spawn mutations for a [`Button`].
pub trait ButtonContext {
    /// Replace the entire default `Quad` (background, border, radii) and synchronize paint.
    fn set_quad(&mut self, quad: Quad);
    /// Replace the hover `Quad` and synchronize paint if currently hovered.
    fn set_hover_quad(&mut self, quad: Option<Quad>);
    /// Replace the pressed `Quad` and synchronize paint if currently pressed.
    fn set_pressed_quad(&mut self, quad: Option<Quad>);
    /// Replace the focused `Quad` and synchronize paint if currently focused.
    fn set_focused_quad(&mut self, quad: Option<Quad>);
    /// Replace the disabled `Quad` and synchronize paint if currently disabled.
    fn set_disabled_quad(&mut self, quad: Option<Quad>);
    /// Change only the default background colour; border and radii are preserved.
    fn set_background(&mut self, color: Color);
    /// Uniform corner radius across all states; all other quad properties are preserved.
    fn set_radius(&mut self, radius: f32);
    /// Per-corner radii across all states; all other quad properties are preserved.
    fn set_radii(&mut self, radii: Corners<f32>);
    /// Border width and colour on default quad; background and radii are preserved.
    fn set_border(&mut self, width: f32, color: Color);
    /// Remove the border on default quad (zero-width, transparent).
    fn clear_border(&mut self);
    /// Explicitly override the interaction state and synchronize paint.
    fn set_state(&mut self, state: WidgetState);
    /// Shorthand for `set_state(Disabled | Enabled)`.
    fn set_disabled(&mut self, disabled: bool);
}

impl ButtonContext for Context<'_, Button> {
    fn set_quad(&mut self, quad: Quad) {
        if self.me().props.quad == quad {
            return;
        }
        self.me().props.quad = quad;
        let active = self.me().active_quad();
        self.set_paint(Paint::Quad(active));
    }

    fn set_hover_quad(&mut self, quad: Option<Quad>) {
        if self.me().props.hover_quad == quad {
            return;
        }
        self.me().props.hover_quad = quad;
        if self.me().state == WidgetState::Hovered {
            let active = self.me().active_quad();
            self.set_paint(Paint::Quad(active));
        }
    }

    fn set_pressed_quad(&mut self, quad: Option<Quad>) {
        if self.me().props.pressed_quad == quad {
            return;
        }
        self.me().props.pressed_quad = quad;
        if self.me().state == WidgetState::Pressed {
            let active = self.me().active_quad();
            self.set_paint(Paint::Quad(active));
        }
    }

    fn set_focused_quad(&mut self, quad: Option<Quad>) {
        if self.me().props.focused_quad == quad {
            return;
        }
        self.me().props.focused_quad = quad;
        if self.me().state == WidgetState::Focused {
            let active = self.me().active_quad();
            self.set_paint(Paint::Quad(active));
        }
    }

    fn set_disabled_quad(&mut self, quad: Option<Quad>) {
        if self.me().props.disabled_quad == quad {
            return;
        }
        self.me().props.disabled_quad = quad;
        if self.me().state == WidgetState::Disabled {
            let active = self.me().active_quad();
            self.set_paint(Paint::Quad(active));
        }
    }

    fn set_background(&mut self, color: Color) {
        if self.me().props.quad.color == color {
            return;
        }
        self.me().props.quad.color = color;
        let active = self.me().active_quad();
        self.set_paint(Paint::Quad(active));
    }

    fn set_radius(&mut self, radius: f32) {
        let radii = Corners::all(radius);
        self.set_radii(radii);
    }

    fn set_radii(&mut self, radii: Corners<f32>) {
        if self.me().props.quad.radii == radii {
            return;
        }
        let me = self.me();
        me.props.quad.radii = radii;
        if let Some(q) = &mut me.props.hover_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut me.props.pressed_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut me.props.disabled_quad {
            q.radii = radii;
        }
        if let Some(q) = &mut me.props.focused_quad {
            q.radii = radii;
        }
        let active = me.active_quad();
        self.set_paint(Paint::Quad(active));
    }

    fn set_border(&mut self, width: f32, color: Color) {
        let insets = Insets::all(width);
        if self.me().props.quad.border == insets && self.me().props.quad.border_color == color {
            return;
        }
        let me = self.me();
        me.props.quad.border = insets;
        me.props.quad.border_color = color;
        let active = me.active_quad();
        self.set_paint(Paint::Quad(active));
    }

    fn clear_border(&mut self) {
        self.set_border(0.0, Color::TRANSPARENT);
    }

    fn set_state(&mut self, state: WidgetState) {
        if self.me().props.state == state {
            return;
        }
        self.me().props.state = state;
        let active = self.me().active_quad();
        self.set_paint(Paint::Quad(active));
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.set_state(if disabled {
            WidgetState::Disabled
        } else {
            WidgetState::Enabled
        });
    }
}

#[cfg(test)]
mod tests {
    use geometry::{Color, Corners, Insets};

    use super::*;

    #[test]
    fn builder_defaults() {
        let b = button();
        assert_eq!(b.state, WidgetState::Enabled);
        assert_eq!(b.quad, Quad::default());
        assert_eq!(b.hover_quad, None);
        assert_eq!(b.pressed_quad, None);
    }

    #[test]
    fn builder_verbs_set_the_right_fields() {
        let b = button()
            .background(Color::BLACK)
            .hover_background(Color::rgba(0.2, 0.2, 0.2, 1.0))
            .pressed_background(Color::rgba(0.1, 0.1, 0.1, 1.0))
            .radius(8.0)
            .border(2.0, Color::WHITE);

        assert_eq!(b.quad.color, Color::BLACK);
        assert_eq!(b.quad.radii, Corners::all(8.0));
        assert_eq!(b.quad.border, Insets::all(2.0));
        assert_eq!(b.quad.border_color, Color::WHITE);

        assert_eq!(b.hover_quad.unwrap().color, Color::rgba(0.2, 0.2, 0.2, 1.0));
        assert_eq!(b.hover_quad.unwrap().radii, Corners::all(8.0));

        assert_eq!(
            b.pressed_quad.unwrap().color,
            Color::rgba(0.1, 0.1, 0.1, 1.0)
        );
        assert_eq!(b.pressed_quad.unwrap().radii, Corners::all(8.0));
    }

    #[test]
    fn disabled_sets_state() {
        assert_eq!(button().disabled(true).state, WidgetState::Disabled);
        assert_eq!(button().disabled(false).state, WidgetState::Enabled);
    }

    #[test]
    fn quad_verb_replaces_entire_quad() {
        let q = Quad::new(Color::BLACK).radius(4.0);
        let b = button().quad(q);
        assert_eq!(b.quad, q);
    }

    #[test]
    fn style_verb_replaces_layout_style() {
        let style = LayoutStyle::default().column();
        let b = button().style(style.clone());
        assert_eq!(b.style, style);
    }

    #[test]
    fn active_quad_resolves_overrides() {
        let mut props = ButtonProps::default();
        props.quad = Quad::new(Color::BLACK);
        props.hover_quad = Some(Quad::new(Color::rgba(0.2, 0.2, 0.2, 1.0)));
        props.pressed_quad = Some(Quad::new(Color::rgba(0.4, 0.4, 0.4, 1.0)));

        props.state = WidgetState::Enabled;
        assert_eq!(props.active_quad().color, Color::BLACK);

        props.state = WidgetState::Hovered;
        assert_eq!(props.active_quad().color, Color::rgba(0.2, 0.2, 0.2, 1.0));

        props.state = WidgetState::Pressed;
        assert_eq!(props.active_quad().color, Color::rgba(0.4, 0.4, 0.4, 1.0));
    }
}
