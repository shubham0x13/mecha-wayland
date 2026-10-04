//! Material Design 3 themed Button widget.
//!
//! Wraps [`widgets::Button`] as the underlying native interactive primitive,
//! resolving design system tokens ([`ButtonVariant`], [`ButtonSize`], [`ColorRole`])
//! into concrete quads and typography.

use crate::color::{ColorSource, IntoColorSource};
use crate::state::{StateLayer, WidgetState};
use crate::text::{Text, TextContextExt, text};
use app::{Build, Context, Handle, Spawner, Widget};
use atlas::SpriteId;
use geometry::{Color, Corners, Insets};
use layout::{LayoutStyle, Val, px};
use paint::Quad;
use std::ops::{Deref, DerefMut};
use theme::{ColorRole, Shape, Spacing, SpawnerThemeExt, TextVariant, ThemeReader};
use widgets::{
    Button as NativeButton, ButtonContext as NativeButtonContext, ButtonProps as NativeButtonProps,
    Icon, IconContext, button_with_props as native_button_with_props,
};

/// Material Design 3 button variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Outlined,
    Elevated,
    Text,
}

impl ButtonVariant {
    #[inline]
    pub fn fg_role(self) -> ColorRole {
        match self {
            Self::Filled => ColorRole::OnPrimary,
            Self::Tonal => ColorRole::OnSecondaryContainer,
            Self::Outlined | Self::Elevated | Self::Text => ColorRole::Primary,
        }
    }
}

/// Standard button sizes according to M3 specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl ButtonSize {
    #[inline]
    pub fn height(self) -> f32 {
        match self {
            Self::XSmall => 24.0,
            Self::Small => 32.0,
            Self::Medium => 40.0,
            Self::Large => 48.0,
            Self::XLarge => 56.0,
        }
    }

    #[inline]
    pub fn text_variant(self) -> TextVariant {
        match self {
            Self::XSmall => TextVariant::LabelSmall,
            Self::Small => TextVariant::LabelMedium,
            Self::Medium => TextVariant::LabelLarge,
            Self::Large => TextVariant::TitleMedium,
            Self::XLarge => TextVariant::TitleLarge,
        }
    }

    #[inline]
    pub fn shape(self) -> Shape {
        match self {
            Self::XSmall | Self::Small => Shape::ExtraSmall,
            Self::Medium | Self::Large => Shape::Small,
            Self::XLarge => Shape::Medium,
        }
    }

    #[inline]
    pub fn icon_size(self) -> f32 {
        match self {
            Self::XSmall => 12.0,
            Self::Small => 16.0,
            Self::Medium => 18.0,
            Self::Large => 22.0,
            Self::XLarge => 26.0,
        }
    }

    #[inline]
    pub fn padding(self) -> Insets<Val> {
        match self {
            Self::XSmall => Insets::new(
                px(Spacing::Space25.dp()),
                px(Spacing::Space100.dp()),
                px(Spacing::Space25.dp()),
                px(Spacing::Space100.dp()),
            ),
            Self::Small => Insets::new(
                px(Spacing::Space50.dp()),
                px(Spacing::Space150.dp()),
                px(Spacing::Space50.dp()),
                px(Spacing::Space150.dp()),
            ),
            Self::Medium => Insets::new(
                px(Spacing::Space100.dp()),
                px(Spacing::Space200.dp()),
                px(Spacing::Space100.dp()),
                px(Spacing::Space200.dp()),
            ),
            Self::Large => Insets::new(
                px(Spacing::Space150.dp()),
                px(Spacing::Space250.dp()),
                px(Spacing::Space150.dp()),
                px(Spacing::Space250.dp()),
            ),
            Self::XLarge => Insets::new(
                px(Spacing::Space200.dp()),
                px(Spacing::Space300.dp()),
                px(Spacing::Space200.dp()),
                px(Spacing::Space300.dp()),
            ),
        }
    }

    #[inline]
    pub fn gap(self) -> Val {
        match self {
            Self::XSmall => px(Spacing::Space50.dp()),
            Self::Small => px(Spacing::Space75.dp()),
            Self::Medium => px(Spacing::Space100.dp()),
            Self::Large => px(Spacing::Space125.dp()),
            Self::XLarge => px(Spacing::Space150.dp()),
        }
    }
}

/// Overrides for explicit styling outside the theme defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ButtonOverrides {
    pub color: Option<ColorSource>,
    pub text_color: Option<ColorSource>,
    pub border_color: Option<ColorSource>,
    pub border_width: Option<f32>,
    pub radii: Option<Corners<f32>>,
}

/// The internal styling rules and state layers for a button variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonStyle {
    pub bg: Option<ColorRole>,
    pub fg: ColorRole,
    pub border: Option<ColorRole>,
    pub border_thickness: f32,

    pub hover_layer: StateLayer,
    pub focus_layer: StateLayer,
    pub pressed_layer: StateLayer,

    pub focus_border: Option<ColorRole>,
    pub focus_border_thickness: f32,

    pub disabled_bg: Option<ColorRole>,
    pub disabled_bg_opacity: f32,
    pub disabled_fg: ColorRole,
    pub disabled_fg_opacity: f32,
    pub disabled_border: Option<ColorRole>,
    pub disabled_border_opacity: f32,
}

impl ButtonStyle {
    pub fn filled() -> Self {
        Self {
            bg: Some(ColorRole::Primary),
            fg: ColorRole::OnPrimary,
            border: None,
            border_thickness: 0.0,
            hover_layer: StateLayer::new(ColorRole::OnPrimary, 0.08),
            focus_layer: StateLayer::new(ColorRole::OnPrimary, 0.10),
            pressed_layer: StateLayer::new(ColorRole::OnPrimary, 0.12),
            focus_border: None,
            focus_border_thickness: 2.0,
            disabled_bg: Some(ColorRole::OnSurface),
            disabled_bg_opacity: 0.12,
            disabled_fg: ColorRole::OnSurface,
            disabled_fg_opacity: 0.38,
            disabled_border: None,
            disabled_border_opacity: 0.0,
        }
    }

    pub fn tonal() -> Self {
        Self {
            bg: Some(ColorRole::SecondaryContainer),
            fg: ColorRole::OnSecondaryContainer,
            border: None,
            border_thickness: 0.0,
            hover_layer: StateLayer::new(ColorRole::OnSecondaryContainer, 0.08),
            focus_layer: StateLayer::new(ColorRole::OnSecondaryContainer, 0.10),
            pressed_layer: StateLayer::new(ColorRole::OnSecondaryContainer, 0.12),
            focus_border: None,
            focus_border_thickness: 2.0,
            disabled_bg: Some(ColorRole::OnSurface),
            disabled_bg_opacity: 0.12,
            disabled_fg: ColorRole::OnSurface,
            disabled_fg_opacity: 0.38,
            disabled_border: None,
            disabled_border_opacity: 0.0,
        }
    }

    pub fn outlined() -> Self {
        Self {
            bg: None,
            fg: ColorRole::Primary,
            border: Some(ColorRole::Outline),
            border_thickness: 1.0,
            hover_layer: StateLayer::new(ColorRole::Primary, 0.08),
            focus_layer: StateLayer::new(ColorRole::Primary, 0.10),
            pressed_layer: StateLayer::new(ColorRole::Primary, 0.12),
            focus_border: Some(ColorRole::Primary),
            focus_border_thickness: 2.0,
            disabled_bg: None,
            disabled_bg_opacity: 0.0,
            disabled_fg: ColorRole::OnSurface,
            disabled_fg_opacity: 0.38,
            disabled_border: Some(ColorRole::OnSurface),
            disabled_border_opacity: 0.12,
        }
    }

    pub fn elevated() -> Self {
        Self {
            bg: Some(ColorRole::SurfaceContainerLow),
            fg: ColorRole::Primary,
            border: None,
            border_thickness: 0.0,
            hover_layer: StateLayer::new(ColorRole::Primary, 0.08),
            focus_layer: StateLayer::new(ColorRole::Primary, 0.10),
            pressed_layer: StateLayer::new(ColorRole::Primary, 0.12),
            focus_border: Some(ColorRole::Primary),
            focus_border_thickness: 2.0,
            disabled_bg: Some(ColorRole::OnSurface),
            disabled_bg_opacity: 0.12,
            disabled_fg: ColorRole::OnSurface,
            disabled_fg_opacity: 0.38,
            disabled_border: None,
            disabled_border_opacity: 0.0,
        }
    }

    pub fn text() -> Self {
        Self {
            bg: None,
            fg: ColorRole::Primary,
            border: None,
            border_thickness: 0.0,
            hover_layer: StateLayer::new(ColorRole::Primary, 0.08),
            focus_layer: StateLayer::new(ColorRole::Primary, 0.10),
            pressed_layer: StateLayer::new(ColorRole::Primary, 0.12),
            focus_border: Some(ColorRole::Outline),
            focus_border_thickness: 2.0,
            disabled_bg: None,
            disabled_bg_opacity: 0.0,
            disabled_fg: ColorRole::OnSurface,
            disabled_fg_opacity: 0.38,
            disabled_border: None,
            disabled_border_opacity: 0.0,
        }
    }

    pub fn for_variant(variant: ButtonVariant) -> Self {
        match variant {
            ButtonVariant::Filled => Self::filled(),
            ButtonVariant::Tonal => Self::tonal(),
            ButtonVariant::Outlined => Self::outlined(),
            ButtonVariant::Elevated => Self::elevated(),
            ButtonVariant::Text => Self::text(),
        }
    }

    pub fn resolve_quad(
        &self,
        state: WidgetState,
        ov: ButtonOverrides,
        size: ButtonSize,
        reader: &impl ThemeReader,
    ) -> Quad {
        let scheme = &reader.theme().colors;

        let mut bg = ov.color.map(|c| c.resolve(reader)).unwrap_or_else(|| {
            self.bg
                .map(|r| r.resolve(scheme))
                .unwrap_or(Color::TRANSPARENT)
        });

        let mut border_color = ov
            .border_color
            .map(|c| c.resolve(reader))
            .unwrap_or_else(|| {
                self.border
                    .map(|r| r.resolve(scheme))
                    .unwrap_or(Color::TRANSPARENT)
            });

        let mut border_thickness = ov.border_width.unwrap_or(self.border_thickness);

        match state {
            WidgetState::Enabled => {}
            WidgetState::Hovered => {
                bg = self.hover_layer.blend_over(bg, scheme);
            }
            WidgetState::Focused => {
                bg = self.focus_layer.blend_over(bg, scheme);
                border_color = self
                    .focus_border
                    .unwrap_or(ColorRole::Outline)
                    .resolve(scheme);
                border_thickness = self.focus_border_thickness;
            }
            WidgetState::Pressed => {
                bg = self.pressed_layer.blend_over(bg, scheme);
            }
            WidgetState::Disabled => {
                let fade = |c: Color, a: f32| Color::rgba(c.r, c.g, c.b, c.a * a);
                let pick = |role: Option<ColorRole>, opacity: f32| {
                    role.map(|r| fade(r.resolve(scheme), opacity))
                        .unwrap_or(Color::TRANSPARENT)
                };
                bg = pick(self.disabled_bg, self.disabled_bg_opacity);
                border_color = pick(self.disabled_border, self.disabled_border_opacity);
            }
            _ => {}
        }

        let radii = ov
            .radii
            .unwrap_or_else(|| Corners::all(size.shape().resolve_radius_dp(size.height())));

        Quad::new(bg)
            .radii(radii)
            .border(border_thickness, border_color)
    }

    pub fn resolve_fg(
        &self,
        state: WidgetState,
        ov: ButtonOverrides,
        reader: &impl ThemeReader,
    ) -> Color {
        let scheme = &reader.theme().colors;
        if state == WidgetState::Disabled {
            let fade = |c: Color, a: f32| Color::rgba(c.r, c.g, c.b, c.a * a);
            fade(self.disabled_fg.resolve(scheme), self.disabled_fg_opacity)
        } else {
            ov.text_color
                .map(|c| c.resolve(reader))
                .unwrap_or_else(|| self.fg.resolve(scheme))
        }
    }
}

/// Create a themed [`ButtonBuilder`] with default (Filled) styling.
pub fn button() -> ButtonBuilder {
    ButtonBuilder::new()
}

/// Create a themed [`ButtonBuilder`] initialized with a text label.
pub fn standard_button(label: impl Into<String>) -> ButtonBuilder {
    button().label(label)
}

/// Create a filled M3 button with a text label.
pub fn filled_button(label: impl Into<String>) -> ButtonBuilder {
    button().variant(ButtonVariant::Filled).label(label)
}

/// Create a tonal M3 button with a text label.
pub fn tonal_button(label: impl Into<String>) -> ButtonBuilder {
    button().variant(ButtonVariant::Tonal).label(label)
}

/// Create an outlined M3 button with a text label.
pub fn outlined_button(label: impl Into<String>) -> ButtonBuilder {
    button().variant(ButtonVariant::Outlined).label(label)
}

/// Create an elevated M3 button with a text label.
pub fn elevated_button(label: impl Into<String>) -> ButtonBuilder {
    button().variant(ButtonVariant::Elevated).label(label)
}

/// Create a text-only M3 button with a label.
pub fn text_button(label: impl Into<String>) -> ButtonBuilder {
    button().variant(ButtonVariant::Text).label(label)
}

/// Builder for a themed [`Button`].
///
/// Wraps [`NativeButtonProps`] as the single source of truth for the native
/// button's geometry and interaction state. Implements [`Deref`] and [`DerefMut`]
/// to [`NativeButtonProps`].
#[derive(Debug, Clone)]
pub struct ButtonBuilder {
    pub props: NativeButtonProps,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub label: Option<String>,
    pub icon: Option<SpriteId>,
    pub overrides: ButtonOverrides,
    pub style: Option<LayoutStyle>,
}

impl Deref for ButtonBuilder {
    type Target = NativeButtonProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl DerefMut for ButtonBuilder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            props: NativeButtonProps::default(),
            variant: ButtonVariant::Filled,
            size: ButtonSize::Medium,
            label: None,
            icon: None,
            overrides: ButtonOverrides::default(),
            style: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn text(self, text: impl Into<String>) -> Self {
        self.label(text)
    }

    pub fn icon(mut self, icon: SpriteId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn state(mut self, state: WidgetState) -> Self {
        self.props.state = state;
        self
    }

    pub fn disabled(self, disabled: bool) -> Self {
        self.state(if disabled {
            WidgetState::Disabled
        } else {
            WidgetState::Enabled
        })
    }

    pub fn focused(self, focused: bool) -> Self {
        self.state(if focused {
            WidgetState::Focused
        } else {
            WidgetState::Enabled
        })
    }

    pub fn color(mut self, color: impl IntoColorSource) -> Self {
        self.overrides.color = color.into_color_source();
        self
    }

    pub fn text_color(mut self, color: impl IntoColorSource) -> Self {
        self.overrides.text_color = color.into_color_source();
        self
    }

    pub fn border_color(mut self, color: impl IntoColorSource) -> Self {
        self.overrides.border_color = color.into_color_source();
        self
    }

    pub fn border(mut self, width: impl Into<Option<f32>>, color: impl IntoColorSource) -> Self {
        self.overrides.border_width = width.into();
        self.overrides.border_color = color.into_color_source();
        self
    }

    pub fn radius(mut self, radius: impl Into<Option<f32>>) -> Self {
        self.overrides.radii = radius.into().map(Corners::all);
        self
    }

    pub fn radii(mut self, radii: impl Into<Option<Corners<f32>>>) -> Self {
        self.overrides.radii = radii.into();
        self
    }

    pub fn shape(mut self, shape: impl Into<Option<Shape>>) -> Self {
        self.overrides.radii = shape
            .into()
            .map(|s| Corners::all(s.resolve_radius_dp(self.size.height())));
        self
    }

    pub fn style(mut self, style: LayoutStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Resolves theme colors, state layers, and shapes, producing concrete [`NativeButtonProps`]
    /// and foreground colours.
    pub fn resolve_props(&self, reader: &impl ThemeReader) -> (NativeButtonProps, Color, Color) {
        let style = ButtonStyle::for_variant(self.variant);

        let enabled_quad =
            style.resolve_quad(WidgetState::Enabled, self.overrides, self.size, reader);
        let hover_quad =
            style.resolve_quad(WidgetState::Hovered, self.overrides, self.size, reader);
        let pressed_quad =
            style.resolve_quad(WidgetState::Pressed, self.overrides, self.size, reader);
        let focused_quad =
            style.resolve_quad(WidgetState::Focused, self.overrides, self.size, reader);
        let disabled_quad =
            style.resolve_quad(WidgetState::Disabled, self.overrides, self.size, reader);

        let fg = style.resolve_fg(WidgetState::Enabled, self.overrides, reader);
        let disabled_fg = style.resolve_fg(WidgetState::Disabled, self.overrides, reader);

        let mut props = self.props;
        props.quad = enabled_quad;
        props.hover_quad = Some(hover_quad);
        props.pressed_quad = Some(pressed_quad);
        props.focused_quad = Some(focused_quad);
        props.disabled_quad = Some(disabled_quad);

        (props, fg, disabled_fg)
    }
}

impl Build for ButtonBuilder {
    type Widget = Button;
}

/// Themed Material Design 3 Button widget.
///
/// Wraps an underlying [`NativeButton`] and coordinates visual styling, optional
/// icon, and optional text label. Implements [`Deref`] and [`DerefMut`] to [`NativeButtonProps`].
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub(crate) props: NativeButtonProps,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) overrides: ButtonOverrides,
    pub(crate) fg: Color,
    pub(crate) native: Handle<NativeButton>,
    pub(crate) label_handle: Option<Handle<Text>>,
    pub(crate) icon_handle: Option<Handle<Icon>>,
}

impl Deref for Button {
    type Target = NativeButtonProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl DerefMut for Button {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

impl Button {
    #[inline]
    pub fn native(&self) -> Handle<NativeButton> {
        self.native
    }

    #[inline]
    pub fn variant(&self) -> ButtonVariant {
        self.variant
    }

    #[inline]
    pub fn size(&self) -> ButtonSize {
        self.size
    }

    #[inline]
    pub fn overrides(&self) -> &ButtonOverrides {
        &self.overrides
    }

    #[inline]
    pub fn fg(&self) -> Color {
        self.fg
    }

    #[inline]
    pub fn label(&self) -> Option<Handle<Text>> {
        self.label_handle
    }

    #[inline]
    pub fn icon(&self) -> Option<Handle<Icon>> {
        self.icon_handle
    }
}

impl Widget for Button {
    type Builder = ButtonBuilder;

    fn build(b: ButtonBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        let (native_props, fg, disabled_fg) = b.resolve_props(s);
        let effective_fg = if b.props.is_disabled() {
            disabled_fg
        } else {
            fg
        };

        *s.component_mut::<LayoutStyle>(me).unwrap() = LayoutStyle::default();

        let button_style = b.style.unwrap_or_else(|| {
            LayoutStyle::default()
                .row()
                .center()
                .column_gap(b.size.gap())
                .padding(b.size.padding())
                .min_height(px(b.size.height()))
        });

        let native = s.spawn(
            me,
            native_button_with_props(native_props).style(button_style),
        );

        let icon_handle = b.icon.map(|sprite| {
            let icon_px = px(b.size.icon_size());
            s.spawn(
                native,
                widgets::icon(sprite)
                    .color(effective_fg)
                    .style(LayoutStyle::default().size(icon_px, icon_px)),
            )
        });

        let label_handle = b.label.map(|txt| {
            s.spawn(
                native,
                text(txt).variant(b.size.text_variant()).color(effective_fg),
            )
        });

        s.on_theme(me, sync_visuals);

        Button {
            props: native_props,
            variant: b.variant,
            size: b.size,
            overrides: b.overrides,
            fg,
            native,
            label_handle,
            icon_handle,
        }
    }
}

/// Synchronizes underlying native button, text, and icon visuals with the active theme and state.
fn sync_visuals(ctx: &mut Context<'_, Button>) {
    let (variant, size, state, overrides, native, label_h, icon_h) = {
        let me = ctx.me();
        (
            me.variant,
            me.size,
            me.props.state,
            me.overrides,
            me.native,
            me.label_handle,
            me.icon_handle,
        )
    };

    let style = ButtonStyle::for_variant(variant);
    let enabled_quad = style.resolve_quad(WidgetState::Enabled, overrides, size, ctx);
    let hover_quad = style.resolve_quad(WidgetState::Hovered, overrides, size, ctx);
    let pressed_quad = style.resolve_quad(WidgetState::Pressed, overrides, size, ctx);
    let focused_quad = style.resolve_quad(WidgetState::Focused, overrides, size, ctx);
    let disabled_quad = style.resolve_quad(WidgetState::Disabled, overrides, size, ctx);

    let fg = style.resolve_fg(WidgetState::Enabled, overrides, ctx);
    let disabled_fg = style.resolve_fg(WidgetState::Disabled, overrides, ctx);
    let effective_fg = if state.is_disabled() { disabled_fg } else { fg };

    ctx.me().fg = fg;
    ctx.me().props.quad = enabled_quad;
    ctx.me().props.hover_quad = Some(hover_quad);
    ctx.me().props.pressed_quad = Some(pressed_quad);
    ctx.me().props.focused_quad = Some(focused_quad);
    ctx.me().props.disabled_quad = Some(disabled_quad);

    if let Some(mut native_ctx) = ctx.at(native) {
        native_ctx.set_quad(enabled_quad);
        native_ctx.set_hover_quad(Some(hover_quad));
        native_ctx.set_pressed_quad(Some(pressed_quad));
        native_ctx.set_focused_quad(Some(focused_quad));
        native_ctx.set_disabled_quad(Some(disabled_quad));
        native_ctx.set_state(state);
    }

    if let Some(lbl) = label_h
        && let Some(mut lbl_ctx) = ctx.at(lbl)
    {
        lbl_ctx.set_variant(size.text_variant());
        lbl_ctx.set_color(effective_fg);
    }

    if let Some(icn) = icon_h
        && let Some(mut icn_ctx) = ctx.at(icn)
    {
        icn_ctx.set_color(effective_fg);
    }
}

/// Extension trait for mutating a spawned [`Button`].
pub trait ButtonContextExt {
    fn set_label(&mut self, label: impl Into<String>);
    fn set_variant(&mut self, variant: ButtonVariant);
    fn set_size(&mut self, size: ButtonSize);
    fn set_state(&mut self, state: WidgetState);
    fn set_disabled(&mut self, disabled: bool);
    fn set_color(&mut self, color: impl IntoColorSource);
    fn set_text_color(&mut self, color: impl IntoColorSource);
    fn set_border_color(&mut self, color: impl IntoColorSource);
    fn set_border(&mut self, width: impl Into<Option<f32>>, color: impl IntoColorSource);
    fn set_radius(&mut self, radius: impl Into<Option<f32>>);
    fn set_radii(&mut self, radii: impl Into<Option<Corners<f32>>>);
    fn set_shape(&mut self, shape: impl Into<Option<Shape>>);
    fn clear_overrides(&mut self);
}

impl ButtonContextExt for Context<'_, Button> {
    fn set_label(&mut self, label: impl Into<String>) {
        if let Some(lbl) = self.me().label_handle
            && let Some(mut c) = self.at(lbl)
        {
            c.set_text(label);
        }
    }

    fn set_variant(&mut self, variant: ButtonVariant) {
        if self.me().variant == variant {
            return;
        }
        self.me().variant = variant;
        sync_visuals(self);
    }

    fn set_size(&mut self, size: ButtonSize) {
        if self.me().size == size {
            return;
        }
        self.me().size = size;
        sync_visuals(self);
    }

    fn set_state(&mut self, state: WidgetState) {
        if self.me().props.state == state {
            return;
        }
        self.me().props.state = state;
        sync_visuals(self);
    }

    fn set_disabled(&mut self, disabled: bool) {
        let target = if disabled {
            WidgetState::Disabled
        } else {
            WidgetState::Enabled
        };
        self.set_state(target);
    }

    fn set_color(&mut self, color: impl IntoColorSource) {
        let cs = color.into_color_source();
        if self.me().overrides.color == cs {
            return;
        }
        self.me().overrides.color = cs;
        sync_visuals(self);
    }

    fn set_text_color(&mut self, color: impl IntoColorSource) {
        let cs = color.into_color_source();
        if self.me().overrides.text_color == cs {
            return;
        }
        self.me().overrides.text_color = cs;
        sync_visuals(self);
    }

    fn set_border_color(&mut self, color: impl IntoColorSource) {
        let cs = color.into_color_source();
        if self.me().overrides.border_color == cs {
            return;
        }
        self.me().overrides.border_color = cs;
        sync_visuals(self);
    }

    fn set_border(&mut self, width: impl Into<Option<f32>>, color: impl IntoColorSource) {
        let w = width.into();
        let cs = color.into_color_source();
        if self.me().overrides.border_width == w && self.me().overrides.border_color == cs {
            return;
        }
        self.me().overrides.border_width = w;
        self.me().overrides.border_color = cs;
        sync_visuals(self);
    }

    fn set_radius(&mut self, radius: impl Into<Option<f32>>) {
        let r = radius.into().map(Corners::all);
        if self.me().overrides.radii == r {
            return;
        }
        self.me().overrides.radii = r;
        sync_visuals(self);
    }

    fn set_radii(&mut self, radii: impl Into<Option<Corners<f32>>>) {
        let r = radii.into();
        if self.me().overrides.radii == r {
            return;
        }
        self.me().overrides.radii = r;
        sync_visuals(self);
    }

    fn set_shape(&mut self, shape: impl Into<Option<Shape>>) {
        let r = shape
            .into()
            .map(|s| Corners::all(s.resolve_radius_dp(self.me().size.height())));
        if self.me().overrides.radii == r {
            return;
        }
        self.me().overrides.radii = r;
        sync_visuals(self);
    }

    fn clear_overrides(&mut self) {
        if self.me().overrides == ButtonOverrides::default() {
            return;
        }
        self.me().overrides = ButtonOverrides::default();
        sync_visuals(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_styles() {
        let filled = ButtonStyle::for_variant(ButtonVariant::Filled);
        assert_eq!(filled.bg, Some(ColorRole::Primary));
        assert_eq!(filled.fg, ColorRole::OnPrimary);

        let tonal = ButtonStyle::for_variant(ButtonVariant::Tonal);
        assert_eq!(tonal.bg, Some(ColorRole::SecondaryContainer));
        assert_eq!(tonal.fg, ColorRole::OnSecondaryContainer);

        let outlined = ButtonStyle::for_variant(ButtonVariant::Outlined);
        assert_eq!(outlined.bg, None);
        assert_eq!(outlined.fg, ColorRole::Primary);
        assert_eq!(outlined.border, Some(ColorRole::Outline));

        let elevated = ButtonStyle::for_variant(ButtonVariant::Elevated);
        assert_eq!(elevated.bg, Some(ColorRole::SurfaceContainerLow));
        assert_eq!(elevated.fg, ColorRole::Primary);

        let text_style = ButtonStyle::for_variant(ButtonVariant::Text);
        assert_eq!(text_style.bg, None);
        assert_eq!(text_style.fg, ColorRole::Primary);
        assert_eq!(text_style.border, None);
    }

    #[test]
    fn size_properties() {
        assert_eq!(ButtonSize::XSmall.height(), 24.0);
        assert_eq!(ButtonSize::Small.height(), 32.0);
        assert_eq!(ButtonSize::Medium.height(), 40.0);
        assert_eq!(ButtonSize::Large.height(), 48.0);
        assert_eq!(ButtonSize::XLarge.height(), 56.0);

        assert_eq!(ButtonSize::XSmall.shape(), Shape::ExtraSmall);
        assert_eq!(ButtonSize::Medium.shape(), Shape::Small);
        assert_eq!(ButtonSize::XLarge.shape(), Shape::Medium);
    }

    #[test]
    fn variant_fg_roles() {
        assert_eq!(ButtonVariant::Filled.fg_role(), ColorRole::OnPrimary);
        assert_eq!(
            ButtonVariant::Tonal.fg_role(),
            ColorRole::OnSecondaryContainer
        );
        assert_eq!(ButtonVariant::Outlined.fg_role(), ColorRole::Primary);
        assert_eq!(ButtonVariant::Elevated.fg_role(), ColorRole::Primary);
        assert_eq!(ButtonVariant::Text.fg_role(), ColorRole::Primary);
    }

    #[test]
    fn builder_defaults() {
        let b = button();
        assert_eq!(b.variant, ButtonVariant::Filled);
        assert_eq!(b.size, ButtonSize::Medium);
        assert_eq!(b.label, None);
        assert_eq!(b.icon, None);
        assert_eq!(b.state, WidgetState::Enabled);
        assert_eq!(b.overrides, ButtonOverrides::default());
    }

    #[test]
    fn builder_overrides_and_props_deref() {
        let b = button()
            .variant(ButtonVariant::Tonal)
            .size(ButtonSize::Small)
            .label("Submit")
            .icon(SpriteId(42))
            .disabled(true)
            .radius(8.0);

        assert_eq!(b.variant, ButtonVariant::Tonal);
        assert_eq!(b.size, ButtonSize::Small);
        assert_eq!(b.label, Some("Submit".to_string()));
        assert_eq!(b.icon, Some(SpriteId(42)));
        assert_eq!(b.state, WidgetState::Disabled);
        assert_eq!(b.overrides.radii, Some(Corners::all(8.0)));
        assert!(b.is_disabled());

        let b2 = button().border(2.0, ColorRole::Error);
        assert_eq!(b2.overrides.border_width, Some(2.0));
        assert!(b2.overrides.border_color.is_some());
    }

    #[test]
    fn variant_constructors() {
        assert_eq!(filled_button("A").variant, ButtonVariant::Filled);
        assert_eq!(tonal_button("B").variant, ButtonVariant::Tonal);
        assert_eq!(outlined_button("C").variant, ButtonVariant::Outlined);
        assert_eq!(elevated_button("D").variant, ButtonVariant::Elevated);
        assert_eq!(text_button("E").variant, ButtonVariant::Text);
        assert_eq!(standard_button("F").label, Some("F".to_string()));
    }

    #[test]
    fn button_widget_spawns_and_syncs_on_update_and_theme() {
        use crate::NativeText;
        use app::App;
        use atlas::Atlas;
        use layout::LayoutModule;
        use paint::PaintModule;
        use theme::prelude::*;

        let mut app = App::new();
        app.add_module(MechanixTheme::dark());
        app.add_module(LayoutModule);
        app.add_module(PaintModule);

        let mut atlas = Atlas::new();
        let font_id = atlas
            .add_font(include_bytes!(
                "../../../crates/atlas/tests/fixtures/Inter-Regular.ttf"
            ))
            .unwrap();
        app.insert_resource(atlas);
        app.insert_resource(crate::font::FontBook::new(font_id));

        let btn_h = app.spawn(app.root(), filled_button("Click Me"));
        app.tick();

        let btn = app.widget::<Button>(btn_h).unwrap();
        let native_h = btn.native();
        let label_h = btn.label().expect("label handle present");

        let native_btn = app.widget::<NativeButton>(native_h).unwrap();
        assert_eq!(native_btn.state, WidgetState::Enabled);
        let dark_bg = native_btn.quad.color;
        assert_eq!(dark_bg, app.color(ColorRole::Primary));

        let text_w = app
            .widget::<NativeText>(app.widget::<Text>(label_h).unwrap().native())
            .unwrap();
        assert_eq!(text_w.string, "Click Me");
        assert_eq!(text_w.color, app.color(ColorRole::OnPrimary));

        // Mutate label and variant via context
        struct Poke;
        impl app::Event for Poke {}

        struct Controller;
        struct ControllerBuilder(Box<dyn FnMut(&mut Context<'_, Controller>)>);
        impl Build for ControllerBuilder {
            type Widget = Controller;
        }
        impl Widget for Controller {
            type Builder = ControllerBuilder;
            fn build(b: ControllerBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
                let mut action = b.0;
                s.on::<Poke>(me, move |ctx, _| action(ctx));
                Controller
            }
        }

        let controller = app.spawn(
            app.root(),
            ControllerBuilder(Box::new(move |ctx: &mut Context<'_, Controller>| {
                let mut c = ctx.at(btn_h).unwrap();
                c.set_label("Updated Label");
                c.set_variant(ButtonVariant::Tonal);
            })),
        );

        app.emit(Poke, controller.id());
        app.flush();
        app.tick();

        let native_btn = app.widget::<NativeButton>(native_h).unwrap();
        assert_eq!(
            native_btn.quad.color,
            app.color(ColorRole::SecondaryContainer)
        );

        let text_w = app
            .widget::<NativeText>(app.widget::<Text>(label_h).unwrap().native())
            .unwrap();
        assert_eq!(text_w.string, "Updated Label");
        assert_eq!(text_w.color, app.color(ColorRole::OnSecondaryContainer));

        // Switch theme to light
        app.set_theme(MechanixTheme::light());
        app.tick();

        let native_btn = app.widget::<NativeButton>(native_h).unwrap();
        let light_bg = native_btn.quad.color;
        assert_eq!(light_bg, app.color(ColorRole::SecondaryContainer));
        assert_ne!(light_bg, dark_bg);
    }

    #[test]
    fn button_setters_accept_values_and_none_to_clear() {
        use app::App;
        use atlas::Atlas;
        use layout::LayoutModule;
        use paint::PaintModule;
        use theme::prelude::*;

        let mut app = App::new();
        app.add_module(MechanixTheme::dark());
        app.add_module(LayoutModule);
        app.add_module(PaintModule);

        let mut atlas = Atlas::new();
        let font_id = atlas
            .add_font(include_bytes!(
                "../../../crates/atlas/tests/fixtures/Inter-Regular.ttf"
            ))
            .unwrap();
        app.insert_resource(atlas);
        app.insert_resource(crate::font::FontBook::new(font_id));

        let btn_h = app.spawn(app.root(), filled_button("Test"));
        app.tick();

        struct Poke;
        impl app::Event for Poke {}

        struct Controller;
        struct ControllerBuilder(Box<dyn FnMut(&mut Context<'_, Controller>)>);
        impl Build for ControllerBuilder {
            type Widget = Controller;
        }
        impl Widget for Controller {
            type Builder = ControllerBuilder;
            fn build(b: ControllerBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
                let mut action = b.0;
                s.on::<Poke>(me, move |ctx, _| action(ctx));
                Controller
            }
        }

        // Set explicit color, radius, and border
        let controller1 = app.spawn(
            app.root(),
            ControllerBuilder(Box::new(move |ctx: &mut Context<'_, Controller>| {
                let mut c = ctx.at(btn_h).unwrap();
                c.set_color(ColorRole::Error);
                c.set_radius(16.0);
                c.set_border(2.0, ColorRole::Primary);
            })),
        );
        app.emit(Poke, controller1.id());
        app.flush();
        app.tick();

        let btn = app.widget::<Button>(btn_h).unwrap();
        assert_eq!(
            btn.overrides.color,
            Some(ColorSource::Role(ColorRole::Error))
        );
        assert_eq!(btn.overrides.radii, Some(Corners::all(16.0)));
        assert_eq!(btn.overrides.border_width, Some(2.0));
        assert_eq!(
            btn.overrides.border_color,
            Some(ColorSource::Role(ColorRole::Primary))
        );

        // Now clear them using None instead of clear_* methods!
        let controller2 = app.spawn(
            app.root(),
            ControllerBuilder(Box::new(move |ctx: &mut Context<'_, Controller>| {
                let mut c = ctx.at(btn_h).unwrap();
                c.set_color(None);
                c.set_radius(None);
                c.set_border(None, None);
            })),
        );
        app.emit(Poke, controller2.id());
        app.flush();
        app.tick();

        let btn = app.widget::<Button>(btn_h).unwrap();
        assert_eq!(btn.overrides.color, None);
        assert_eq!(btn.overrides.radii, None);
        assert_eq!(btn.overrides.border_width, None);
        assert_eq!(btn.overrides.border_color, None);
    }
}
