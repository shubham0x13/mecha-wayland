//! The `widgets` crate: primitive building blocks.
//!
//! # Widgets
//!
//! | Widget  | Role |
//! |---------|------|
//! | [`Div`]    | Container box — background, border, radii |
//! | [`Button`] | Pressable container — embedded state machine, explicit paint |
//! | [`Text`]   | Single-line shaped text run |
//! | [`Icon`]   | Tinted monochrome sprite |
//! | [`Image`]  | Full-colour sprite |
//!
//! # Model
//!
//! - Widgets write only `LayoutStyle`, `Measure` and `Paint` — components
//!   that `layout` and `paint` already register.  `widgets` installs no
//!   `Module`, `Component` or `Resource` of its own, except that `Button`
//!   requires `interactivity` events to drive its state machine.
//! - Using any widget requires the caller to have installed `LayoutModule`
//!   and `PaintModule` and inserted an `Atlas` resource.  `Button` also
//!   requires `InteractivityModule`.
//! - Mutation after spawn happens through each widget's own `*Context`
//!   trait ([`DivContext`], [`ButtonContext`], [`TextContext`],
//!   [`IconContext`], [`ImageContext`]), plus the generic
//!   `layout::StyleContext` and `paint::PaintContext`.
//!
//! # Quick start
//!
//! ```
//! use app::prelude::*;
//! use atlas::prelude::*;
//! use layout::prelude::*;
//! use paint::prelude::*;
//! use widgets::prelude::*;
//!
//! let mut app = App::new();
//! app.add_module(LayoutModule).add_module(PaintModule);
//! app.insert_resource(Atlas::new());
//!
//! let root = app.spawn_with(
//!     app.root(),
//!     div().style(LayoutStyle::default().size(px(200.0), px(100.0))),
//!     (LayoutRoot(true),),
//! );
//!
//! let font = app
//!     .resource_mut::<Atlas>()
//!     .add_font(include_bytes!("../../atlas/tests/fixtures/Inter-Regular.ttf"))
//!     .unwrap();
//!
//! let label = app.spawn(root, text(font, "hi"));
//! app.tick();
//! assert!(app.component::<Layout>(label).unwrap().rect.width() > 0.0);
//! ```

mod button;
mod div;
mod icon;
mod image;
mod state;
mod text;

pub use button::{Button, ButtonBuilder, ButtonContext, ButtonProps, button, button_with_props};
pub use div::{Div, DivBuilder, DivContext, div};
pub use icon::{Icon, IconBuilder, IconContext, icon};
pub use image::{Image, ImageBuilder, ImageContext, image};
pub use state::WidgetState;
pub use text::{
    Text, TextAlign, TextBuilder, TextContext, TextDecoration, TextOverflow, TextProps, TextWrap,
    VerticalTrim, text, text_with_props,
};

pub mod prelude {
    pub use crate::{
        Button, ButtonBuilder, ButtonContext, ButtonProps, Div, DivBuilder, DivContext, Icon,
        IconBuilder, IconContext, Image, ImageBuilder, ImageContext, Text, TextAlign, TextBuilder,
        TextContext, TextDecoration, TextOverflow, TextProps, TextWrap, VerticalTrim, WidgetState,
        button, button_with_props, div, icon, image, text, text_with_props,
    };
}
