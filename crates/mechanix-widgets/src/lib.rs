//! Themed widget layer: Material-Design-3-aware primitives built on top of
//! the raw `widgets` crate.
//!
//! # Layer contract
//!
//! **`mechanix-widgets` takes design-system tokens** ([`ColorRole`],
//! [`TextVariant`], [`ButtonVariant`], …) and resolves them to concrete
//! values (`Color`, `f32` px, [`Quad`]) before delegating to the
//! underlying `widgets` primitives.
//!
//! Widgets in this crate:
//!
//! | Widget | Themed concerns |
//! |--------|----------------|
//! | [`Button`] | Variant (Filled/Tonal/Outlined/Elevated/Text), size, state-layer colours |
//! | [`Text`]   | `TextVariant` → font, px, weight; `ColorRole` → colour |
//!
//! # Native widgets
//!
//! The raw, theme-agnostic primitives ([`NativeButton`], `Div`, `Icon`,
//! `Image`, [`NativeText`]) are re-exported from `widgets` for callers that need
//! them alongside themed widgets without an extra `use widgets::…` import.
//!
//! [`Quad`]: paint::Quad
//! [`ColorRole`]: theme::ColorRole
//! [`TextVariant`]: theme::TextVariant

pub mod button;
pub mod color;
pub mod font;
pub mod state;
pub mod text;

pub use button::*;
pub use color::*;
pub use font::*;
pub use state::*;
pub use text::*;

// Re-export native primitives so callers have a single import path.
pub use widgets::{
    Button as NativeButton, ButtonBuilder as NativeButtonBuilder,
    ButtonContext as NativeButtonContext, ButtonProps as NativeButtonProps, Div, DivBuilder,
    DivContext, Icon, IconBuilder, IconContext, Image, ImageBuilder, ImageContext,
    Text as NativeText, TextAlign, TextBuilder as NativeTextBuilder,
    TextContext as NativeTextContext, TextDecoration, TextOverflow, TextProps as NativeTextProps,
    TextWrap, VerticalTrim, WidgetState, button as native_button,
    button_with_props as native_button_with_props, div, icon, image, text as native_text,
    text_with_props as native_text_with_props,
};

pub mod prelude {
    pub use crate::*;
    pub use theme::{ColorRole, FontWeight, TextVariant};
}
