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
pub use widgets::{TextAlign, TextDecoration, TextOverflow, TextWrap, VerticalTrim};

pub mod prelude {
    pub use crate::{button::*, color::*, font::*, state::*, text::*};
    pub use theme::{ColorRole, FontWeight, TextVariant};
    pub use widgets::{TextAlign, TextDecoration, TextOverflow, TextWrap, VerticalTrim};
}
