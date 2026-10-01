use crate::color::ColorSource;
use crate::state::{StateLayer, WidgetState};
use crate::text::{Text, TextContextExt, text};
use app::{Build, Context, Handle, Spawner, Widget};
use atlas::SpriteId;
use geometry::{Color, Corners, Insets, Size};
use interactivity::prelude::{Enter, Exit, Press, Release};
use layout::{LayoutStyle, Val, px};
use paint::{Paint, PaintContext, Quad};
use theme::{ColorRole, Shape, Spacing, SpawnerThemeExt, TextVariant, ThemeReader};
use widgets::{Icon, IconContext, icon};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Outlined,
    Text,
}

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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ButtonOverrides {
    pub color: Option<ColorSource>,
    pub text_color: Option<ColorSource>,
    pub border_color: Option<ColorSource>,
    pub border_width: Option<f32>,
    pub radii: Option<Corners<f32>>,
}

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
            ButtonVariant::Text => Self::text(),
        }
    }

    pub(crate) fn resolve(
        &self,
        state: WidgetState,
        ov: ButtonOverrides,
        size: ButtonSize,
        reader: &impl ThemeReader,
    ) -> (Quad, Color) {
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

        let mut fg = ov
            .text_color
            .map(|c| c.resolve(reader))
            .unwrap_or_else(|| self.fg.resolve(scheme));

        match state {
            WidgetState::Enabled => {}
            WidgetState::Hovered => {
                bg = self.hover_layer.apply(bg, scheme);
            }
            WidgetState::Focused => {
                bg = self.focus_layer.apply(bg, scheme);
                border_color = self
                    .focus_border
                    .unwrap_or(ColorRole::Outline)
                    .resolve(scheme);
                border_thickness = self.focus_border_thickness;
            }
            WidgetState::Pressed => {
                bg = self.pressed_layer.apply(bg, scheme);
            }
            WidgetState::Disabled => {
                let fade = |c: Color, a: f32| Color::rgba(c.r, c.g, c.b, c.a * a);
                let pick = |role: Option<ColorRole>, opacity: f32| {
                    role.map(|r| fade(r.resolve(scheme), opacity))
                        .unwrap_or(Color::TRANSPARENT)
                };
                bg = pick(self.disabled_bg, self.disabled_bg_opacity);
                border_color = pick(self.disabled_border, self.disabled_border_opacity);
                fg = fade(self.disabled_fg.resolve(scheme), self.disabled_fg_opacity);
            }
        }

        let radii = ov
            .radii
            .unwrap_or_else(|| Corners::all(size.shape().resolve_radius_dp(size.height())));

        (
            Quad::new(bg)
                .radii(radii)
                .border(border_thickness, border_color),
            fg,
        )
    }
}

pub fn button(label: impl Into<String>) -> ButtonBuilder {
    ButtonBuilder::new(label)
}

pub struct ButtonBuilder {
    label: String,
    variant: ButtonVariant,
    size: ButtonSize,
    state: WidgetState,
    icon: Option<SpriteId>,
    overrides: ButtonOverrides,
    style: Option<LayoutStyle>,
}

impl ButtonBuilder {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Filled,
            size: ButtonSize::Medium,
            state: WidgetState::Enabled,
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

    pub fn state(mut self, state: WidgetState) -> Self {
        self.state = state;
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

    pub fn icon(mut self, sprite: SpriteId) -> Self {
        self.icon = Some(sprite);
        self
    }

    pub fn color(mut self, color: impl Into<ColorSource>) -> Self {
        self.overrides.color = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<ColorSource>) -> Self {
        self.overrides.text_color = Some(color.into());
        self
    }

    pub fn border_color(mut self, color: impl Into<ColorSource>) -> Self {
        self.overrides.border_color = Some(color.into());
        self
    }

    pub fn border(mut self, width: f32, color: impl Into<ColorSource>) -> Self {
        self.overrides.border_width = Some(width);
        self.overrides.border_color = Some(color.into());
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.overrides.radii = Some(Corners::all(radius));
        self
    }

    pub fn radii(mut self, radii: Corners<f32>) -> Self {
        self.overrides.radii = Some(radii);
        self
    }

    pub fn shape(mut self, shape: Shape) -> Self {
        let r = shape.resolve_radius_dp(self.size.height());
        self.overrides.radii = Some(Corners::all(r));
        self
    }

    pub fn style(mut self, style: LayoutStyle) -> Self {
        self.style = Some(style);
        self
    }
}

impl Build for ButtonBuilder {
    type Widget = Button;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub(crate) style: ButtonStyle,
    pub(crate) label: Handle<Text>,
    pub(crate) icon: Option<Handle<Icon>>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) state: WidgetState,
    pub(crate) overrides: ButtonOverrides,
}

impl Button {
    pub fn label(&self) -> Handle<Text> {
        self.label
    }
    pub fn icon(&self) -> Option<Handle<Icon>> {
        self.icon
    }
    pub fn variant(&self) -> ButtonVariant {
        self.variant
    }
    pub fn size(&self) -> ButtonSize {
        self.size
    }
    pub fn state(&self) -> WidgetState {
        self.state
    }
    pub fn is_disabled(&self) -> bool {
        self.state.is_disabled()
    }
    pub fn overrides(&self) -> &ButtonOverrides {
        &self.overrides
    }
}

impl Widget for Button {
    type Builder = ButtonBuilder;

    fn build(b: ButtonBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        let style = ButtonStyle::for_variant(b.variant);
        let (quad, fg) = style.resolve(b.state, b.overrides, b.size, s);

        *s.component_mut::<Paint>(me).unwrap() = Paint::Quad(quad);
        *s.component_mut::<LayoutStyle>(me).unwrap() = b.style.unwrap_or_else(|| {
            LayoutStyle::default()
                .row()
                .center()
                .column_gap(b.size.gap())
                .padding(b.size.padding())
                .min_height(px(b.size.height()))
        });

        let icon_handle = b.icon.map(|sprite| {
            s.spawn(
                me,
                icon(sprite).color(fg).style(
                    LayoutStyle::default().size(px(b.size.icon_size()), px(b.size.icon_size())),
                ),
            )
        });

        let label = s.spawn(me, text(b.label).variant(b.size.text_variant()).color(fg));

        // Pointer state machine — always registered so the button remains interactive
        // if re-enabled at runtime. Guards inside each handler check the current state.
        s.on::<Enter>(me, |ctx: &mut Context<'_, Button>, _| {
            if matches!(ctx.me().state, WidgetState::Enabled) {
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

        s.on_theme(me, sync_visuals);

        Button {
            style,
            label,
            icon: icon_handle,
            variant: b.variant,
            size: b.size,
            state: b.state,
            overrides: b.overrides,
        }
    }
}

pub trait ButtonContextExt {
    fn set_label(&mut self, text: impl Into<String>);
    fn set_variant(&mut self, variant: ButtonVariant);
    fn set_size(&mut self, size: ButtonSize);
    fn set_state(&mut self, state: WidgetState);
    fn set_disabled(&mut self, disabled: bool);
    fn set_color(&mut self, color: impl Into<ColorSource>);
    fn clear_color(&mut self);
    fn set_text_color(&mut self, color: impl Into<ColorSource>);
    fn clear_text_color(&mut self);
    fn set_border_color(&mut self, color: impl Into<ColorSource>);
    fn clear_border_color(&mut self);
    fn set_border(&mut self, width: f32, color: impl Into<ColorSource>);
    fn clear_border(&mut self);
    fn set_radius(&mut self, radius: f32);
    fn set_radii(&mut self, radii: Corners<f32>);
    fn set_shape(&mut self, shape: Shape);
    fn clear_radius(&mut self);
    fn clear_overrides(&mut self);
}

impl ButtonContextExt for Context<'_, Button> {
    fn set_label(&mut self, text: impl Into<String>) {
        let label = self.me().label;
        if let Some(mut c) = self.at(label) {
            c.set_text(text);
        }
    }

    fn set_variant(&mut self, variant: ButtonVariant) {
        self.me().variant = variant;
        self.me().style = ButtonStyle::for_variant(variant);
        sync_visuals(self);
    }

    fn set_size(&mut self, size: ButtonSize) {
        self.me().size = size;
        let (tv, icon_size) = (size.text_variant(), size.icon_size());
        let (label, icon) = (self.me().label, self.me().icon);
        if let Some(mut c) = self.at(label) {
            c.set_variant(tv);
        }
        if let Some(h) = icon
            && let Some(mut c) = self.at(h)
        {
            c.set_size(Size::new(icon_size, icon_size));
        }
        sync_visuals(self);
    }

    fn set_state(&mut self, state: WidgetState) {
        if self.me().state == state {
            return;
        }
        self.me().state = state;
        sync_visuals(self);
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.set_state(if disabled {
            WidgetState::Disabled
        } else {
            WidgetState::Enabled
        });
    }

    fn set_color(&mut self, color: impl Into<ColorSource>) {
        self.me().overrides.color = Some(color.into());
        sync_visuals(self);
    }

    fn clear_color(&mut self) {
        self.me().overrides.color = None;
        sync_visuals(self);
    }

    fn set_text_color(&mut self, color: impl Into<ColorSource>) {
        self.me().overrides.text_color = Some(color.into());
        sync_visuals(self);
    }

    fn clear_text_color(&mut self) {
        self.me().overrides.text_color = None;
        sync_visuals(self);
    }

    fn set_border_color(&mut self, color: impl Into<ColorSource>) {
        self.me().overrides.border_color = Some(color.into());
        sync_visuals(self);
    }

    fn clear_border_color(&mut self) {
        self.me().overrides.border_color = None;
        sync_visuals(self);
    }

    fn set_border(&mut self, width: f32, color: impl Into<ColorSource>) {
        self.me().overrides.border_width = Some(width);
        self.me().overrides.border_color = Some(color.into());
        sync_visuals(self);
    }

    fn clear_border(&mut self) {
        self.me().overrides.border_width = None;
        self.me().overrides.border_color = None;
        sync_visuals(self);
    }

    fn set_radius(&mut self, radius: f32) {
        self.me().overrides.radii = Some(Corners::all(radius));
        sync_visuals(self);
    }

    fn set_radii(&mut self, radii: Corners<f32>) {
        self.me().overrides.radii = Some(radii);
        sync_visuals(self);
    }

    fn set_shape(&mut self, shape: Shape) {
        let r = shape.resolve_radius_dp(self.me().size.height());
        self.me().overrides.radii = Some(Corners::all(r));
        sync_visuals(self);
    }

    fn clear_radius(&mut self) {
        self.me().overrides.radii = None;
        sync_visuals(self);
    }

    fn clear_overrides(&mut self) {
        self.me().overrides = ButtonOverrides::default();
        sync_visuals(self);
    }
}

fn sync_visuals(ctx: &mut Context<'_, Button>) {
    let (style, state, overrides, size, label, icon) = {
        let me = ctx.me();
        (me.style, me.state, me.overrides, me.size, me.label, me.icon)
    };

    let (quad, fg) = style.resolve(state, overrides, size, ctx);

    ctx.set_paint(Paint::Quad(quad));

    if let Some(mut c) = ctx.at(label) {
        c.set_color(fg);
    }
    if let Some(h) = icon
        && let Some(mut c) = ctx.at(h)
    {
        c.set_color(fg);
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
    fn builder_defaults() {
        let b = button("Go");
        assert_eq!(b.variant, ButtonVariant::Filled);
        assert_eq!(b.size, ButtonSize::Medium);
        assert_eq!(b.state, WidgetState::Enabled);
        assert_eq!(b.overrides, ButtonOverrides::default());
    }

    #[test]
    fn builder_overrides() {
        let b = button("Action")
            .variant(ButtonVariant::Tonal)
            .size(ButtonSize::Small)
            .disabled(true)
            .radius(8.0);

        assert_eq!(b.variant, ButtonVariant::Tonal);
        assert_eq!(b.size, ButtonSize::Small);
        assert_eq!(b.state, WidgetState::Disabled);
        assert_eq!(b.overrides.radii, Some(Corners::all(8.0)));

        let b2 = button("Cancel").border(2.0, ColorRole::Error);
        assert_eq!(b2.overrides.border_width, Some(2.0));
        assert!(b2.overrides.border_color.is_some());
    }

    #[test]
    fn overrides_defaults() {
        let ov = ButtonOverrides::default();
        assert_eq!(ov.color, None);
        assert_eq!(ov.text_color, None);
        assert_eq!(ov.border_width, None);
        assert_eq!(ov.border_color, None);
        assert_eq!(ov.radii, None);
    }
}
