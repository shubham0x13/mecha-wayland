use app::{App, Context, Spawner, Widget};
use geometry::Color;

use crate::color::ColorRole;
use crate::typography::{TextVariant, TypographyStyle};
use crate::{MechanixTheme, ThemeMode};

/// Signal sent when `MechanixTheme` is updated.
///
/// Dispatched immediately in the current tick so that widget [`ApplyTheme`]
/// updates occur before `PostTick`'s paint drain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ThemeChanged;
impl app::Signal for ThemeChanged {}

/// Event emitted to all live widgets when the theme changes.
///
/// Widgets listen to this with `s.on_theme(me, |ctx| { ... })` to
/// update their paints, styles, and colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ApplyTheme;
impl app::Event for ApplyTheme {}

/// System triggered whenever `ThemeChanged` is signaled.
///
/// Emits [`ApplyTheme`] to all live nodes in the widget tree.
pub fn on_theme_changed(app: &mut App, _: &ThemeChanged) {
    let mut targets = vec![app.root()];
    targets.extend(app.tree().descendants(app.root()));
    app.emit(ApplyTheme, targets);
}

// ── Theme reading trait ───────────────────────────────────────────────

/// Common read access to the active [`MechanixTheme`].
///
/// Implemented for [`Context`], [`Spawner`], and [`App`].
pub trait ThemeReader {
    /// Borrow the theme inside a closure without cloning.
    fn with_theme<R>(&self, f: impl FnOnce(&MechanixTheme) -> R) -> R;

    /// Clone the entire current theme.
    #[inline]
    fn theme(&self) -> MechanixTheme {
        self.with_theme(|t| t.clone())
    }

    /// Read the active mode (`Dark` or `Light`).
    #[inline]
    fn theme_mode(&self) -> ThemeMode {
        self.with_theme(|t| t.mode)
    }

    /// Resolve a color from a [`ColorRole`], exact [`Color`], or [`ColorSource`].
    #[inline]
    fn color(&self, source: impl Into<crate::color::ColorSource>) -> Color {
        let source = source.into();
        self.with_theme(|t| source.resolve(&t.colors))
    }

    /// Resolve a [`TextVariant`] to a [`TypographyStyle`].
    #[inline]
    fn typography(&self, variant: TextVariant) -> TypographyStyle {
        self.with_theme(|t| t.typography(variant))
    }
}

// ── Context theme extension ───────────────────────────────────────────

pub trait ContextThemeExt: ThemeReader {
    /// Replace the theme. Changes will be broadcast to all widgets at `PostTick`.
    fn set_theme(&mut self, theme: MechanixTheme);

    /// Mutate the theme in-place.
    fn update_theme<F: FnOnce(&mut MechanixTheme)>(&mut self, f: F);
}

impl<W: Widget> ThemeReader for Context<'_, W> {
    #[inline]
    fn with_theme<R>(&self, f: impl FnOnce(&MechanixTheme) -> R) -> R {
        f(self.resource::<MechanixTheme>())
    }
}

impl<W: Widget> ContextThemeExt for Context<'_, W> {
    fn set_theme(&mut self, theme: MechanixTheme) {
        *self.resource_mut::<MechanixTheme>() = theme;
        self.signal(ThemeChanged);
    }

    fn update_theme<F: FnOnce(&mut MechanixTheme)>(&mut self, f: F) {
        f(&mut self.resource_mut::<MechanixTheme>());
        self.signal(ThemeChanged);
    }
}

// ── App theme extension ───────────────────────────────────────────────

pub trait AppThemeExt: ThemeReader {
    /// Replace the theme. Changes are broadcast immediately to all widgets.
    fn set_theme(&mut self, theme: MechanixTheme);

    /// Mutate the theme in-place.
    fn update_theme<F: FnOnce(&mut MechanixTheme)>(&mut self, f: F);
}

impl ThemeReader for App {
    #[inline]
    fn with_theme<R>(&self, f: impl FnOnce(&MechanixTheme) -> R) -> R {
        f(self.resource::<MechanixTheme>())
    }
}

impl AppThemeExt for App {
    fn set_theme(&mut self, theme: MechanixTheme) {
        *self.resource_mut::<MechanixTheme>() = theme;
        self.signal(ThemeChanged);
    }

    fn update_theme<F: FnOnce(&mut MechanixTheme)>(&mut self, f: F) {
        f(&mut self.resource_mut::<MechanixTheme>());
        self.signal(ThemeChanged);
    }
}

// ── Spawner theme extension ───────────────────────────────────────────

pub trait SpawnerThemeExt<W: Widget>: ThemeReader {
    /// Listen for theme changes on `target`.
    ///
    /// When the theme changes, `handler` is called with a [`Context`]
    /// so the widget can re-read colors and update its styles.
    fn on_theme(
        &mut self,
        target: impl Into<app::NodeId>,
        handler: impl FnMut(&mut Context<'_, W>) + 'static,
    ) -> &mut Self;
}

impl<W: Widget> ThemeReader for Spawner<'_, W> {
    #[inline]
    fn with_theme<R>(&self, f: impl FnOnce(&MechanixTheme) -> R) -> R {
        f(self.resource::<MechanixTheme>())
    }
}

impl<W: Widget> SpawnerThemeExt<W> for Spawner<'_, W> {
    fn on_theme(
        &mut self,
        target: impl Into<app::NodeId>,
        mut handler: impl FnMut(&mut Context<'_, W>) + 'static,
    ) -> &mut Self {
        self.on::<ApplyTheme>(target, move |ctx, _| handler(ctx))
    }
}
