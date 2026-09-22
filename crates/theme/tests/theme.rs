use app::prelude::*;
use theme::prelude::*;

struct ThemedWidget {
    applied_count: usize,
}

struct ThemedWidgetBuilder;
impl Build for ThemedWidgetBuilder {
    type Widget = ThemedWidget;
}

impl Widget for ThemedWidget {
    type Builder = ThemedWidgetBuilder;

    fn build(_: ThemedWidgetBuilder, me: Handle<Self>, s: &mut Spawner<'_, Self>) -> Self {
        s.on_theme(me, |ctx| {
            ctx.me().applied_count += 1;
        });

        ThemedWidget { applied_count: 0 }
    }
}

#[test]
fn theme_change_triggers_apply_theme_on_widgets() {
    let mut app = App::new();
    app.add_module(MechanixTheme::dark());

    let handle = app.spawn(app.root(), ThemedWidgetBuilder);
    assert_eq!(app.widget::<ThemedWidget>(handle).unwrap().applied_count, 0);

    // Initial tick: widgets built with initial theme, no ThemeChanged signal has been sent yet.
    app.tick();
    assert_eq!(app.widget::<ThemedWidget>(handle).unwrap().applied_count, 0);

    // Change theme via set_theme: queues ThemeChanged signal
    app.set_theme(MechanixTheme::light());
    assert_eq!(app.theme_mode(), ThemeMode::Light);

    // Tick flushes ThemeChanged, emitting ApplyTheme to all widgets.
    app.tick();
    assert_eq!(app.widget::<ThemedWidget>(handle).unwrap().applied_count, 1);

    // Tick without change does not re-trigger
    app.tick();
    assert_eq!(app.widget::<ThemedWidget>(handle).unwrap().applied_count, 1);

    // Changing via update_theme also signals ThemeChanged!
    app.update_theme(|t| t.mode = ThemeMode::Dark);
    app.tick();
    assert_eq!(app.widget::<ThemedWidget>(handle).unwrap().applied_count, 2);
}
