use crate::color::ColorSource;
use crate::font::FontContextExt;
use app::{Build, Context, Handle, Spawner, Widget};
use layout::LayoutStyle;
use theme::{ColorRole, FontWeight, SpawnerThemeExt, TextVariant, ThemeReader};
use widgets::prelude::{
    Text as NativeText, TextAlign, TextContext as NativeTextContext, TextDecoration, TextOverflow,
    TextWrap, VerticalTrim, text as native_text,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub(crate) native: Handle<NativeText>,
    pub(crate) string: String,
    pub(crate) variant: TextVariant,
    pub(crate) emphasised: bool,

    // Theme overrides (None = inherit from active variant/theme)
    pub(crate) color: Option<ColorSource>,
    pub(crate) size: Option<f32>,
    pub(crate) weight: Option<FontWeight>,
    pub(crate) letter_spacing: Option<f32>,
    pub(crate) line_height: Option<f32>,

    // Text layout & formatting
    pub(crate) align: TextAlign,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
    pub(crate) max_lines: Option<usize>,
    pub(crate) vertical_trim: VerticalTrim,
    pub(crate) decoration: TextDecoration,
}

impl Text {
    pub fn native(&self) -> Handle<NativeText> {
        self.native
    }

    pub fn text(&self) -> &str {
        &self.string
    }

    pub fn variant(&self) -> TextVariant {
        self.variant
    }

    pub fn is_emphasised(&self) -> bool {
        self.emphasised
    }

    pub fn color(&self) -> Option<ColorSource> {
        self.color
    }

    pub fn size(&self) -> Option<f32> {
        self.size
    }

    pub fn weight(&self) -> Option<FontWeight> {
        self.weight
    }

    pub fn letter_spacing(&self) -> Option<f32> {
        self.letter_spacing
    }

    pub fn line_height(&self) -> Option<f32> {
        self.line_height
    }

    pub fn align(&self) -> TextAlign {
        self.align
    }

    pub fn wrap(&self) -> TextWrap {
        self.wrap
    }

    pub fn overflow(&self) -> TextOverflow {
        self.overflow
    }

    pub fn max_lines(&self) -> Option<usize> {
        self.max_lines
    }

    pub fn vertical_trim(&self) -> VerticalTrim {
        self.vertical_trim
    }

    pub fn decoration(&self) -> TextDecoration {
        self.decoration
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

pub fn text(s: impl Into<String>) -> TextBuilder {
    TextBuilder::new(s)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextBuilder {
    string: String,
    variant: TextVariant,
    emphasised: bool,
    color: Option<ColorSource>,
    size: Option<f32>,
    weight: Option<FontWeight>,
    letter_spacing: Option<f32>,
    line_height: Option<f32>,
    align: TextAlign,
    wrap: TextWrap,
    overflow: TextOverflow,
    max_lines: Option<usize>,
    vertical_trim: VerticalTrim,
    decoration: TextDecoration,
    style: LayoutStyle,
}

impl TextBuilder {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            string: s.into(),
            variant: TextVariant::BodyLarge,
            emphasised: false,
            color: None,
            size: None,
            weight: None,
            letter_spacing: None,
            line_height: None,
            align: TextAlign::Left,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
            max_lines: None,
            vertical_trim: VerticalTrim::Normal,
            decoration: TextDecoration::None,
            style: LayoutStyle::default(),
        }
    }

    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn emphasised(mut self) -> Self {
        self.emphasised = true;
        self
    }

    pub fn color(mut self, color: impl Into<ColorSource>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn size(mut self, px: f32) -> Self {
        self.size = Some(px);
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = Some(spacing);
        self
    }

    pub fn line_height(mut self, height: f32) -> Self {
        self.line_height = Some(height);
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn max_lines(mut self, n: usize) -> Self {
        self.max_lines = Some(n);
        self
    }

    pub fn vertical_trim(mut self, trim: VerticalTrim) -> Self {
        self.vertical_trim = trim;
        self
    }

    pub fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.decoration = decoration;
        self
    }

    pub fn style(mut self, style: LayoutStyle) -> Self {
        self.style = style;
        self
    }
}

impl Build for TextBuilder {
    type Widget = Text;
}

impl Widget for Text {
    type Builder = TextBuilder;

    fn build(b: TextBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        let typo = s.typography(b.variant);
        let effective_weight = b.weight.unwrap_or(if b.emphasised {
            typo.weight_emphasised
        } else {
            typo.weight
        });
        let font = s.font(effective_weight);
        let px = b.size.unwrap_or(typo.font_size).round() as u16;
        let letter_spacing = b.letter_spacing.unwrap_or(typo.letter_spacing);
        let line_height = b.line_height.unwrap_or(typo.line_height);
        let color = resolve_color(b.color, s);

        *s.component_mut::<LayoutStyle>(me).unwrap() = b.style;

        let native = s.spawn(
            me,
            native_text(font, &b.string)
                .size(px)
                .color(color)
                .letter_spacing(letter_spacing)
                .line_height(line_height)
                .align(b.align)
                .wrap(b.wrap)
                .overflow(b.overflow)
                .max_lines(b.max_lines.unwrap_or(usize::MAX))
                .vertical_trim(b.vertical_trim)
                .decoration(b.decoration),
        );

        s.on_theme(me, |ctx| {
            let (variant, size_ov, letter_ov, line_ov, emphasised, weight, color, native) = {
                let me = ctx.me();
                (
                    me.variant,
                    me.size,
                    me.letter_spacing,
                    me.line_height,
                    me.emphasised,
                    me.weight,
                    me.color,
                    me.native,
                )
            };
            let typo = ctx.typography(variant);
            let px = size_ov.unwrap_or(typo.font_size).round() as u16;
            let effective_weight = weight.unwrap_or(if emphasised {
                typo.weight_emphasised
            } else {
                typo.weight
            });
            let new_font = ctx.font(effective_weight);
            let new_color = resolve_color(color, ctx);
            let letter_spacing = letter_ov.unwrap_or(typo.letter_spacing);
            let line_height = line_ov.unwrap_or(typo.line_height);

            if let Some(mut c) = ctx.at(native) {
                c.set_size(px);
                c.set_font(new_font);
                c.set_color(new_color);
                c.set_letter_spacing(letter_spacing);
                c.set_line_height(line_height);
            }
        });

        Text {
            native,
            string: b.string,
            variant: b.variant,
            emphasised: b.emphasised,
            color: b.color,
            size: b.size,
            weight: b.weight,
            letter_spacing: b.letter_spacing,
            line_height: b.line_height,
            align: b.align,
            wrap: b.wrap,
            overflow: b.overflow,
            max_lines: b.max_lines,
            vertical_trim: b.vertical_trim,
            decoration: b.decoration,
        }
    }
}

pub trait TextContextExt {
    fn set_text(&mut self, text: impl Into<String>);
    fn set_variant(&mut self, variant: TextVariant);
    fn set_emphasised(&mut self, emphasised: bool);

    fn set_color(&mut self, color: impl Into<ColorSource>);
    fn clear_color(&mut self);
    fn set_size(&mut self, px: f32);
    fn clear_size(&mut self);
    fn set_weight(&mut self, weight: FontWeight);
    fn clear_weight(&mut self);
    fn set_letter_spacing(&mut self, spacing: f32);
    fn clear_letter_spacing(&mut self);
    fn set_line_height(&mut self, height: f32);
    fn clear_line_height(&mut self);
    fn clear_overrides(&mut self);

    fn set_align(&mut self, align: TextAlign);
    fn set_wrap(&mut self, wrap: TextWrap);
    fn set_overflow(&mut self, overflow: TextOverflow);
    fn set_max_lines(&mut self, max_lines: Option<usize>);
    fn set_vertical_trim(&mut self, trim: VerticalTrim);
    fn set_decoration(&mut self, decoration: TextDecoration);
}

impl TextContextExt for Context<'_, Text> {
    fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.me().string = text.clone();
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_text(text);
        }
    }

    fn set_variant(&mut self, variant: TextVariant) {
        self.me().variant = variant;
        let (size_ov, letter_ov, line_ov, emphasised, weight) = {
            let me = self.me();
            (
                me.size,
                me.letter_spacing,
                me.line_height,
                me.emphasised,
                me.weight,
            )
        };
        let typo = self.typography(variant);
        let px = size_ov.unwrap_or(typo.font_size).round() as u16;
        let effective_weight = weight.unwrap_or(if emphasised {
            typo.weight_emphasised
        } else {
            typo.weight
        });
        let new_font = self.font(effective_weight);
        let letter_spacing = letter_ov.unwrap_or(typo.letter_spacing);
        let line_height = line_ov.unwrap_or(typo.line_height);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_size(px);
            c.set_font(new_font);
            c.set_letter_spacing(letter_spacing);
            c.set_line_height(line_height);
        }
    }

    fn set_emphasised(&mut self, emphasised: bool) {
        self.me().emphasised = emphasised;
        let (variant, weight) = (self.me().variant, self.me().weight);
        let typo = self.typography(variant);
        let effective_weight = weight.unwrap_or(if emphasised {
            typo.weight_emphasised
        } else {
            typo.weight
        });
        let new_font = self.font(effective_weight);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_font(new_font);
        }
    }

    fn set_color(&mut self, color: impl Into<ColorSource>) {
        let source = color.into();
        self.me().color = Some(source);
        let resolved = source.resolve(self);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_color(resolved);
        }
    }

    fn clear_color(&mut self) {
        self.me().color = None;
        let native = self.me().native;
        let resolved = resolve_color(None, self);
        if let Some(mut c) = self.at(native) {
            c.set_color(resolved);
        }
    }

    fn set_size(&mut self, px: f32) {
        self.me().size = Some(px);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_size(px.round() as u16);
        }
    }

    fn clear_size(&mut self) {
        self.me().size = None;
        let variant = self.me().variant;
        let typo = self.typography(variant);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_size(typo.font_size.round() as u16);
        }
    }

    fn set_weight(&mut self, weight: FontWeight) {
        self.me().weight = Some(weight);
        let new_font = self.font(weight);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_font(new_font);
        }
    }

    fn clear_weight(&mut self) {
        self.me().weight = None;
        let (variant, emphasised) = (self.me().variant, self.me().emphasised);
        let typo = self.typography(variant);
        let effective_weight = if emphasised {
            typo.weight_emphasised
        } else {
            typo.weight
        };
        let new_font = self.font(effective_weight);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_font(new_font);
        }
    }

    fn set_letter_spacing(&mut self, spacing: f32) {
        self.me().letter_spacing = Some(spacing);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_letter_spacing(spacing);
        }
    }

    fn clear_letter_spacing(&mut self) {
        self.me().letter_spacing = None;
        let variant = self.me().variant;
        let typo = self.typography(variant);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_letter_spacing(typo.letter_spacing);
        }
    }

    fn set_line_height(&mut self, height: f32) {
        self.me().line_height = Some(height);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_line_height(height);
        }
    }

    fn clear_line_height(&mut self) {
        self.me().line_height = None;
        let variant = self.me().variant;
        let typo = self.typography(variant);
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_line_height(typo.line_height);
        }
    }

    fn clear_overrides(&mut self) {
        let me = self.me();
        me.color = None;
        me.size = None;
        me.weight = None;
        me.letter_spacing = None;
        me.line_height = None;

        let (variant, emphasised, native) = (me.variant, me.emphasised, me.native);
        let typo = self.typography(variant);
        let effective_weight = if emphasised {
            typo.weight_emphasised
        } else {
            typo.weight
        };
        let new_font = self.font(effective_weight);
        let new_color = resolve_color(None, self);

        if let Some(mut c) = self.at(native) {
            c.set_size(typo.font_size.round() as u16);
            c.set_font(new_font);
            c.set_color(new_color);
            c.set_letter_spacing(typo.letter_spacing);
            c.set_line_height(typo.line_height);
        }
    }

    fn set_align(&mut self, align: TextAlign) {
        self.me().align = align;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_align(align);
        }
    }

    fn set_wrap(&mut self, wrap: TextWrap) {
        self.me().wrap = wrap;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_wrap(wrap);
        }
    }

    fn set_overflow(&mut self, overflow: TextOverflow) {
        self.me().overflow = overflow;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_overflow(overflow);
        }
    }

    fn set_max_lines(&mut self, max_lines: Option<usize>) {
        self.me().max_lines = max_lines;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_max_lines(max_lines);
        }
    }

    fn set_vertical_trim(&mut self, trim: VerticalTrim) {
        self.me().vertical_trim = trim;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_vertical_trim(trim);
        }
    }

    fn set_decoration(&mut self, decoration: TextDecoration) {
        self.me().decoration = decoration;
        let native = self.me().native;
        if let Some(mut c) = self.at(native) {
            c.set_decoration(decoration);
        }
    }
}

fn resolve_color(color: Option<ColorSource>, reader: &impl ThemeReader) -> geometry::Color {
    color
        .unwrap_or_else(|| ColorSource::from(ColorRole::OnSurface))
        .resolve(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults() {
        let b = text("Hello");
        assert_eq!(b.string, "Hello");
        assert_eq!(b.variant, TextVariant::BodyLarge);
        assert!(!b.emphasised);
        assert_eq!(b.color, None);
        assert_eq!(b.size, None);
        assert_eq!(b.weight, None);
        assert_eq!(b.letter_spacing, None);
        assert_eq!(b.line_height, None);
        assert_eq!(b.align, TextAlign::Left);
        assert_eq!(b.wrap, TextWrap::NoWrap);
        assert_eq!(b.overflow, TextOverflow::Clip);
        assert_eq!(b.max_lines, None);
    }

    #[test]
    fn builder_overrides() {
        let b = text("Custom")
            .variant(TextVariant::HeadlineMedium)
            .emphasised()
            .color(ColorRole::Primary)
            .size(28.0)
            .weight(FontWeight::Bold)
            .letter_spacing(0.5)
            .line_height(34.0)
            .align(TextAlign::Center)
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Ellipsis)
            .max_lines(3);

        assert_eq!(b.variant, TextVariant::HeadlineMedium);
        assert!(b.emphasised);
        assert_eq!(b.color, Some(ColorRole::Primary.into()));
        assert_eq!(b.size, Some(28.0));
        assert_eq!(b.weight, Some(FontWeight::Bold));
        assert_eq!(b.letter_spacing, Some(0.5));
        assert_eq!(b.line_height, Some(34.0));
        assert_eq!(b.align, TextAlign::Center);
        assert_eq!(b.wrap, TextWrap::Word);
        assert_eq!(b.overflow, TextOverflow::Ellipsis);
        assert_eq!(b.max_lines, Some(3));
    }
}
