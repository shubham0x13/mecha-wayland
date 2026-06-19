use lock_screen::{AppState, LockMode, atlas, handlers, time};
use lock_screen::widgets::clock::{ClockChanged, ClockUpdate};

fn main() {
    let state = AppState::new();

    let mut app = app::App::new(state)
        .mount(io_ring::module())
        .mount(wayland::module())
        .mount(timer::module())
        .mount(interactivity::module())
        .mount(app::Module::new().on(|s: &mut AppState, _: &app::Start| {
            s.renderer.upload_atlas(&atlas::UI).expect("atlas upload");
        }))
        .mount(app::Module::new().on(|s: &mut AppState, ev: &timer::TimerEvent| {
            let (h, m, ..) = time::try_clock_tick(s.clock_timer_id, ev)?;
            time::arm_clock(
                &mut s.timer,
                &mut s.clock_timer_id,
                time::Precision::Minutes,
            );
            Some(ClockUpdate(h, m))
        }))
        .mount(app::Module::new().on(|s: &mut AppState, ev: &ClockUpdate| {
            let changed = s
                .lock_uis
                .values_mut()
                .fold(false, |acc, ui| ui.update_clock(ev.0, ev.1) || acc);
            if changed { Some(ClockChanged) } else { None }
        }))
        .mount(app::Module::new().on(|s: &mut AppState, _: &ClockChanged| {
            s.redraw_all_lock_surfaces();
        }))
        .mount(
            app::Module::new().on(|s: &mut AppState, _: &wayland::Initilised| {
                s.setup_layer_surface();
                time::arm_clock(
                    &mut s.timer,
                    &mut s.clock_timer_id,
                    time::Precision::Minutes,
                );
            }),
        )
        .mount(app::Module::new().on(handlers::on_layer_surface_configured))
        .mount(app::Module::new().on(handlers::on_lock_surface_configured))
        .mount(app::Module::new().on(
            |s: &mut AppState, ev: &wayland::ExtSessionLockEvent| match ev {
                wayland::ExtSessionLockEvent::Locked => {
                    s.mode = LockMode::Locked;
                }
                wayland::ExtSessionLockEvent::Finished => {
                    s.cleanup_lock();
                }
            },
        ))
        .mount(app::Module::new().on(handlers::on_frame_done))
        .mount(app::Module::new().on(handlers::on_buffer_release))
        .mount(app::Module::new().on(|s: &mut AppState, ev: &interactivity::KeyEvent| {
            if let interactivity::KeyEvent::Press { key, modifiers, .. } = ev {
                // Alt + L -> lock
                if modifiers.alt && *key == 38 && s.mode == LockMode::Unlocked {
                    s.trigger_lock();
                }
            }
        }))
        .mount(app::Module::new().on(handlers::on_touch))
        .mount(app::Module::new().on(handlers::on_pointer));

    app.dispatch(&app::Start);
    loop {
        app.dispatch(&app::PrePoll);
        app.dispatch(&app::Poll);
    }
}
