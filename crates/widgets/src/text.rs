//! `Text`: one line of glyphs, shaped once at build and re-shaped by its
//! `Context` setters. Single-line only.

use app::{Build, Context, Handle, Spawner, Widget};
use atlas::{Atlas, FontId};
use geometry::{Color, Point, Size};
use layout::{LayoutStyle, Measure};
use paint::{MonochromeSprite, Paint, PaintContext};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalTrim {
    #[default]
    Normal,
    CapHeight,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
    Visible,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Strikethrough,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextWrap {
    #[default]
    NoWrap,
    Word,
    BreakAll, // or Char
}

/// The configurable properties of a native text widget.
#[derive(Debug, Clone, PartialEq)]
pub struct TextProps {
    pub font: FontId,
    pub px: u16,
    pub color: Color,
    pub string: String,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub vertical_trim: VerticalTrim,
    pub align: TextAlign,
    pub overflow: TextOverflow,
    pub decoration: TextDecoration,
    pub wrap: TextWrap,
    pub max_lines: Option<usize>,
}

impl TextProps {
    pub fn new(font: FontId, s: impl Into<String>) -> Self {
        Self {
            font,
            string: s.into(),
            px: 16,
            color: Color::WHITE,
            line_height: None,
            letter_spacing: None,
            vertical_trim: VerticalTrim::Normal,
            align: TextAlign::Left,
            overflow: TextOverflow::Clip,
            decoration: TextDecoration::None,
            wrap: TextWrap::NoWrap,
            max_lines: None,
        }
    }
}

pub struct Text {
    pub props: TextProps,
}

impl std::ops::Deref for Text {
    type Target = TextProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl std::ops::DerefMut for Text {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

pub fn text(font: FontId, s: impl Into<String>) -> TextBuilder {
    TextBuilder {
        props: TextProps::new(font, s),
        style: LayoutStyle::default(),
    }
}

pub fn text_with_props(props: TextProps) -> TextBuilder {
    TextBuilder::from_props(props)
}

pub struct TextBuilder {
    pub props: TextProps,
    pub style: LayoutStyle,
}

impl std::ops::Deref for TextBuilder {
    type Target = TextProps;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.props
    }
}

impl std::ops::DerefMut for TextBuilder {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}

impl TextBuilder {
    pub fn from_props(props: TextProps) -> Self {
        Self {
            props,
            style: LayoutStyle::default(),
        }
    }

    pub fn font(mut self, font: FontId) -> Self {
        self.props.font = font;
        self
    }
    pub fn style(mut self, style: LayoutStyle) -> Self {
        self.style = style;
        self
    }
    pub fn size(mut self, px: u16) -> Self {
        self.props.px = px;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }
    pub fn line_height(mut self, height: f32) -> Self {
        self.props.line_height = Some(height);
        self
    }
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.props.letter_spacing = Some(spacing);
        self
    }
    pub fn vertical_trim(mut self, trim: VerticalTrim) -> Self {
        self.props.vertical_trim = trim;
        self
    }
    pub fn align(mut self, align: TextAlign) -> Self {
        self.props.align = align;
        self
    }
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.props.overflow = overflow;
        self
    }
    pub fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.props.decoration = decoration;
        self
    }
    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.props.wrap = wrap;
        self
    }
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.props.max_lines = Some(max_lines);
        self
    }
}

impl Build for TextBuilder {
    type Widget = Text;
}

impl Widget for Text {
    type Builder = TextBuilder;
    fn build(b: TextBuilder, me: Handle<Text>, s: &mut Spawner<'_, Text>) -> Text {
        *s.component_mut::<LayoutStyle>(me).unwrap() = b.style;
        let (sprites, size) = {
            let mut atlas = s.resource_mut::<Atlas>();
            shape(&mut atlas, b.props.font, b.props.px, &b.props.string, b.props.color)
        };
        *s.component_mut::<Paint>(me).unwrap() = Paint::Monochrome(sprites);
        *s.component_mut::<Measure>(me).unwrap() = Measure::fixed(size);

        Text { props: b.props }
    }
}

/// One line of `text` in `font` at `px`, tinted `color`: a sprite per
/// glyph with ink, pen-advanced left to right, baselined by the font's
/// ascent, and the line's natural size (`pen` wide, `ascent - descent +
/// gap` tall). A character missing from the font's cmap is skipped
/// entirely — it contributes no width and no sprite, as if it were never
/// in the string. A character present in the cmap but with no ink (such
/// as a space) still advances the pen; only the sprite is skipped.
fn shape(
    atlas: &mut Atlas,
    font: FontId,
    px: u16,
    text: &str,
    color: Color,
) -> (Vec<MonochromeSprite>, Size) {
    let line = atlas.line(font, px);
    let baseline = line.ascent;
    let mut pen = 0.0f32;
    let mut sprites = Vec::new();
    for ch in text.chars() {
        let Some((f, id)) = atlas.lookup(&[font], ch) else {
            continue;
        };
        let g = atlas.glyph(f, id, px);
        if g.tile.bounds.width() > 0.0 {
            sprites.push(MonochromeSprite::new(
                g.tile,
                Point::new(pen + g.left, baseline - g.top),
                Size::new(g.tile.bounds.width(), g.tile.bounds.height()),
                color,
            ));
        }
        pen += g.advance;
    }
    let height = line.ascent - line.descent + line.gap;
    (sprites, Size::new(pen, height))
}

pub trait TextContext {
    fn set_text(&mut self, text: impl Into<String>);
    fn set_size(&mut self, px: u16);
    fn set_color(&mut self, color: Color);
    fn set_font(&mut self, font: FontId);
    fn set_line_height(&mut self, height: f32);
    fn set_letter_spacing(&mut self, spacing: f32);
    fn set_vertical_trim(&mut self, trim: VerticalTrim);
    fn set_align(&mut self, align: TextAlign);
    fn set_overflow(&mut self, overflow: TextOverflow);
    fn set_decoration(&mut self, decoration: TextDecoration);
    fn set_wrap(&mut self, wrap: TextWrap);
    fn set_max_lines(&mut self, max_lines: Option<usize>);
}

impl TextContext for Context<'_, Text> {

    fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.me().string == text {
            return;
        }
        self.me().string = text;
        reshape_text(self);
    }

    fn set_size(&mut self, px: u16) {
        if self.me().px == px {
            return;
        }
        self.me().px = px;
        reshape_text(self);
    }

    fn set_color(&mut self, color: Color) {
        if self.me().color == color {
            return;
        }
        self.me().color = color;
        retint_text(self, color);
    }

    fn set_font(&mut self, font: FontId) {
        if self.me().font == font {
            return;
        }
        self.me().font = font;
        reshape_text(self);
    }

    // TODO: apply line-height to the Measure component once multi-line
    // shaping is supported.  Single-line text ignores this value at render time.
    fn set_line_height(&mut self, height: f32) {
        if self.me().line_height == Some(height) {
            return;
        }
        self.me().line_height = Some(height);
    }

    // TODO: re-shape with per-glyph advance adjustment when letter-spacing
    // is set.  Needs a modified `shape()` that adds `spacing` to each
    // `g.advance`.  Deferred alongside `set_font`.
    fn set_letter_spacing(&mut self, spacing: f32) {
        if self.me().letter_spacing == Some(spacing) {
            return;
        }
        self.me().letter_spacing = Some(spacing);
    }

    // TODO: adjust the Measure height to use cap-height instead of the full
    // ascender when `VerticalTrim::CapHeight` is set.
    fn set_vertical_trim(&mut self, trim: VerticalTrim) {
        if self.me().vertical_trim == trim {
            return;
        }
        self.me().vertical_trim = trim;
    }

    // TODO: horizontal alignment only matters for multi-line runs where
    // individual lines are shorter than the container.  Implement once
    // multi-line shaping lands.
    fn set_align(&mut self, align: TextAlign) {
        if self.me().align == align {
            return;
        }
        self.me().align = align;
    }

    // TODO: apply clip / ellipsis at render time using the layout bounds
    // once the renderer consults TextOverflow.
    fn set_overflow(&mut self, overflow: TextOverflow) {
        if self.me().overflow == overflow {
            return;
        }
        self.me().overflow = overflow;
    }

    // TODO: draw underline / strikethrough rules as extra sprites in the
    // paint list once the renderer supports it.
    fn set_decoration(&mut self, decoration: TextDecoration) {
        if self.me().decoration == decoration {
            return;
        }
        self.me().decoration = decoration;
    }

    // TODO: requires multi-line text shaping — re-shape with line-break
    // logic when wrap mode changes.
    fn set_wrap(&mut self, wrap: TextWrap) {
        if self.me().wrap == wrap {
            return;
        }
        self.me().wrap = wrap;
    }

    // TODO: requires multi-line text shaping — truncate or clip when the
    // shaped run exceeds `max_lines`.
    fn set_max_lines(&mut self, max_lines: Option<usize>) {
        if self.me().max_lines == max_lines {
            return;
        }
        self.me().max_lines = max_lines;
    }
}

fn reshape_text(ctx: &mut Context<'_, Text>) {
    let (font, px, color, string) = {
        let w = ctx.me();
        (w.font, w.px, w.color, w.string.clone())
    };
    let (sprites, size) = {
        let mut atlas = ctx.resource_mut::<Atlas>();
        shape(&mut atlas, font, px, &string, color)
    };
    ctx.set_paint(Paint::Monochrome(sprites));
    *ctx.component_mut::<Measure>().unwrap() = Measure::fixed(size);
}

fn retint_text(ctx: &mut Context<'_, Text>, color: Color) {
    let mut paint = ctx.paint().clone();
    if let Paint::Monochrome(sprites) = &mut paint {
        for sprite in sprites.iter_mut() {
            sprite.color = color;
        }
    }
    ctx.set_paint(paint);
}

#[cfg(test)]
mod tests {
    use atlas::Atlas;
    use geometry::Color;

    use super::*;

    const INTER: &[u8] = include_bytes!("../../atlas/tests/fixtures/Inter-Regular.ttf");

    fn inter() -> (Atlas, FontId) {
        let mut atlas = Atlas::new();
        let font = atlas.add_font(INTER).unwrap();
        (atlas, font)
    }

    #[test]
    fn shape_places_one_sprite_per_glyph_with_ink() {
        let (mut atlas, font) = inter();
        let (sprites, size) = shape(&mut atlas, font, 14, "ab", Color::WHITE);
        assert_eq!(sprites.len(), 2);
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
        assert_eq!(sprites[0].color, Color::WHITE);
    }

    #[test]
    fn shape_skips_a_space_but_still_advances_the_pen() {
        let (mut atlas, font) = inter();
        let (with_space, wide) = shape(&mut atlas, font, 14, "a b", Color::WHITE);
        let (without_space, narrow) = shape(&mut atlas, font, 14, "ab", Color::WHITE);
        assert_eq!(with_space.len(), 2, "the space has no ink");
        assert_eq!(without_space.len(), 2);
        assert!(
            wide.width > narrow.width,
            "the space's advance still counts"
        );
    }

    #[test]
    fn shape_of_empty_text_is_a_zero_width_run_at_the_line_height() {
        let (mut atlas, font) = inter();
        let (sprites, size) = shape(&mut atlas, font, 14, "", Color::WHITE);
        assert!(sprites.is_empty());
        assert_eq!(size.width, 0.0);
        assert!(
            size.height > 0.0,
            "the line height does not depend on content"
        );
    }

    #[test]
    fn shape_skips_a_character_missing_from_the_cmap_and_contributes_no_width() {
        let (mut atlas, font) = inter();
        // U+4E2D ('中') is not in Inter-Regular's cmap.
        assert!(
            atlas.lookup(&[font], '\u{4e2d}').is_none(),
            "test assumes Inter-Regular has no glyph for U+4E2D"
        );
        let (with_missing, wider) = shape(&mut atlas, font, 14, "a\u{4e2d}b", Color::WHITE);
        let (_, narrower) = shape(&mut atlas, font, 14, "ab", Color::WHITE);
        assert_eq!(
            with_missing.len(),
            2,
            "the missing character paints no sprite"
        );
        assert_eq!(
            wider.width, narrower.width,
            "a character missing from the cmap contributes zero width, unlike a space"
        );
    }

    #[test]
    fn builder_verbs_set_the_right_fields() {
        let (_, font) = inter();
        let b = text(font, "hi")
            .size(24)
            .color(Color::BLACK)
            .line_height(32.0)
            .letter_spacing(1.5)
            .vertical_trim(VerticalTrim::CapHeight)
            .align(TextAlign::Center)
            .overflow(TextOverflow::Ellipsis)
            .decoration(TextDecoration::Underline)
            .wrap(TextWrap::Word)
            .max_lines(3);
        assert_eq!(b.string, "hi");
        assert_eq!(b.px, 24);
        assert_eq!(b.color, Color::BLACK);
        assert_eq!(b.font, font);
        assert_eq!(b.line_height, Some(32.0));
        assert_eq!(b.letter_spacing, Some(1.5));
        assert_eq!(b.vertical_trim, VerticalTrim::CapHeight);
        assert_eq!(b.align, TextAlign::Center);
        assert_eq!(b.overflow, TextOverflow::Ellipsis);
        assert_eq!(b.decoration, TextDecoration::Underline);
        assert_eq!(b.wrap, TextWrap::Word);
        assert_eq!(b.max_lines, Some(3));
    }
}
