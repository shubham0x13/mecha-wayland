//! A themed counter with a Dark / Light mode toggle button.
//!
//! Demonstrates:
//! - Initializing and using `MechanixTheme` with `mecha-wayland`
//! - Reading theme colors via `s.color(ColorRole::...)`
//! - Listening for theme changes using `s.on_theme(...)`
//! - Switching themes at runtime with `ctx.set_theme(...)`
//!
//! Run under a Wayland session: `cargo run --example themed_counter`.

use std::cell::Cell;
use std::rc::Rc;

use mecha_wayland::prelude::*;

/// A theme-aware clickable button accepting either exact colors or theme roles.
struct Button;

fn button(
    font: FontId,
    label: impl Into<String>,
    bg: impl Into<ColorSource>,
    fg: impl Into<ColorSource>,
) -> ButtonBuilder {
    ButtonBuilder {
        font,
        label: label.into(),
        bg: bg.into(),
        fg: fg.into(),
        width: px(44.0),
        height: px(44.0),
    }
}

struct ButtonBuilder {
    font: FontId,
    label: String,
    bg: ColorSource,
    fg: ColorSource,
    width: Val,
    height: Val,
}

impl ButtonBuilder {
    pub fn size(mut self, width: Val, height: Val) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl Build for ButtonBuilder {
    type Widget = Button;
}

impl Widget for Button {
    type Builder = ButtonBuilder;

    fn build(b: ButtonBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        *s.component_mut::<LayoutStyle>(me).unwrap() =
            LayoutStyle::default().center().size(b.width, b.height);

        let initial_bg = s.color(b.bg);
        let initial_fg = s.color(b.fg);

        *s.component_mut::<Paint>(me).unwrap() = Paint::Quad(Quad::new(initial_bg).radius(8.0));

        let label = s.spawn(me, text(b.font, b.label).color(initial_fg).size(16));

        // Re-paint when the theme changes
        let bg = b.bg;
        let fg = b.fg;
        s.on_theme(me, move |ctx| {
            let new_bg = ctx.color(bg);
            let new_fg = ctx.color(fg);
            ctx.set_paint(Paint::Quad(Quad::new(new_bg).radius(8.0)));
            ctx.at(label).unwrap().set_color(new_fg);
        });

        Button
    }
}

/// A counter widget that responds to theme changes and includes a theme switcher.
struct ThemedCounter;

fn themed_counter(font: FontId) -> ThemedCounterBuilder {
    ThemedCounterBuilder { font }
}

struct ThemedCounterBuilder {
    font: FontId,
}

impl Build for ThemedCounterBuilder {
    type Widget = ThemedCounter;
}

impl Widget for ThemedCounter {
    type Builder = ThemedCounterBuilder;

    fn build(b: ThemedCounterBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        // Container column: fills 100% of the window
        *s.component_mut::<LayoutStyle>(me).unwrap() = LayoutStyle::default()
            .column()
            .center()
            .fill()
            .padding_all(px(20.0))
            .gap(px(14.0));

        // Background quad for the whole window interior
        *s.component_mut::<Paint>(me).unwrap() =
            Paint::Quad(Quad::new(s.color(ColorRole::Surface)));

        // Mode display label
        let mode_text = match s.theme_mode() {
            ThemeMode::Dark => "Mode: Dark",
            ThemeMode::Light => "Mode: Light",
        };
        let mode_label = s.spawn(
            me,
            text(b.font, mode_text)
                .color(s.color(ColorRole::OnSurfaceVariant))
                .size(14),
        );

        // Counter value display
        let count_label = s.spawn(
            me,
            text(b.font, "0")
                .color(s.color(ColorRole::OnSurface))
                .size(36),
        );

        // Row with [-] and [+] buttons
        let row = s.spawn(
            me,
            div().style(LayoutStyle::default().row().center().gap(px(10.0))),
        );
        let minus = s.spawn(
            row,
            button(b.font, " - ", ColorRole::Primary, ColorRole::OnPrimary),
        );
        let plus = s.spawn(
            row,
            button(b.font, " + ", ColorRole::Primary, ColorRole::OnPrimary),
        );

        // Theme toggle button
        let toggle_btn = s.spawn(
            me,
            button(
                b.font,
                "Toggle Theme",
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            )
            .size(px(140.0), px(40.0)),
        );

        // ── State handling ───────────────────────────────────────────────

        // Counter logic
        let count = Rc::new(Cell::new(0i32));

        let dec = count.clone();
        s.on::<Clicked>(minus, move |ctx, _| {
            dec.set(dec.get() - 1);
            ctx.at(count_label).unwrap().set_text(dec.get().to_string());
        });

        let inc = count.clone();
        s.on::<Clicked>(plus, move |ctx, _| {
            inc.set(inc.get() + 1);
            ctx.at(count_label).unwrap().set_text(inc.get().to_string());
        });

        // Toggle theme logic
        s.on::<Clicked>(toggle_btn, move |ctx, _| {
            let next_theme = match ctx.theme_mode() {
                ThemeMode::Dark => MechanixTheme::light(),
                ThemeMode::Light => MechanixTheme::dark(),
            };
            ctx.set_theme(next_theme);
        });

        // ── Theme change handler for this container ──────────────────────
        s.on_theme(me, move |ctx| {
            // Update container background
            let bg = ctx.color(ColorRole::Surface);
            ctx.set_paint(Paint::Quad(Quad::new(bg)));

            // Update mode text & color
            let text = match ctx.theme_mode() {
                ThemeMode::Dark => "Mode: Dark",
                ThemeMode::Light => "Mode: Light",
            };
            let on_surface_var = ctx.color(ColorRole::OnSurfaceVariant);
            let mut mode_ctx = ctx.at(mode_label).unwrap();
            mode_ctx.set_text(text);
            mode_ctx.set_color(on_surface_var);

            // Update counter text color
            let on_surface = ctx.color(ColorRole::OnSurface);
            ctx.at(count_label).unwrap().set_color(on_surface);
        });

        ThemedCounter
    }
}

/// Spawns the OS window and houses the themed counter.
struct Shell;

struct ShellBuilder {
    root: NodeId,
    font: FontId,
}

impl Build for ShellBuilder {
    type Widget = Shell;
}

impl Widget for Shell {
    type Builder = ShellBuilder;

    fn build(b: ShellBuilder, _me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        let win = s.spawn(
            b.root,
            window()
                .title("Themed Counter")
                .clear(Color::rgb(0.08, 0.08, 0.10))
                .layout(LayoutStyle::default().center().size(px(280.0), px(240.0))),
        );

        s.on::<CloseRequested>(win, |ctx, _| ctx.signal(Stop));
        s.spawn(win, themed_counter(b.font));

        Shell
    }
}

fn main() {
    let mut app = App::new();

    // Engine modules
    app.add_module(LayoutModule)
        .add_module(PaintModule)
        .add_module(WindowModule)
        .add_module(InteractivityModule)
        .add_module(RenderModule::default())
        .insert_resource(Atlas::new());

    // Install theme module (defaults to dark mode)
    app.add_module(MechanixTheme::dark());

    // Wayland & Presentation
    app.add_module(RingModule::default())
        .add_module(
            WaylandModule::new()
                .bind::<WlCompositor>()
                .bind::<ZwpLinuxDmabufV1>()
                .bind::<XdgWmBase>()
                .bind::<WlSeat>(),
        )
        .add_module(PresentationModule {
            app_id: "mecha.themed_counter".into(),
            budget: Budget::default(),
        });

    let font = app
        .resource_mut::<Atlas>()
        .add_font(include_bytes!(
            "../crates/atlas/tests/fixtures/Inter-Regular.ttf"
        ))
        .expect("Inter loads");

    let root = app.root();
    app.spawn(root, ShellBuilder { root, font });
    app.run();
}
