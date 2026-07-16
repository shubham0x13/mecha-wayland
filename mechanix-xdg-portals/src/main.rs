use app::RegisteredModule;
use app::prelude::*;
use bluetooth::{BluetoothBackend, bluetooth_module};
use dbus::{DbusConnection, SessionBus, SystemBus, module as dbus_module};
use file_chooser::{FileChooserBackend, filechooser_module};
use portal_core::{DbusMonitor, dbus_monitor_module};

use io_ring::{Ring, RingSettings};
use window_manager::WindowManager;

#[derive(State)]
pub struct AppRoot {
    dbus_monitor: DbusMonitor,
    backend: FileChooserBackend,
    bt_backend: BluetoothBackend,
    window_manager: WindowManager,
    dbus_session: DbusConnection<SessionBus>,
    dbus_system: DbusConnection<SystemBus>,
    ring: Ring,
}

pub fn main_poll_module<S>() -> impl RegisteredModule<AppRoot, S> {
    Module::<AppRoot, _, _>::new().on(|s: &mut AppRoot, _: &app::Start| {
        s.window_manager.upload_atlas(&portal_core::atlas::UI);
        println!("[main] Service started. Waiting for portal D-Bus requests...");
    })
}

fn main() {
    let ring = Ring::new(RingSettings::default());
    let dbus_system = DbusConnection::<SystemBus>::new(ring.proxy());
    let dbus_session = DbusConnection::<SessionBus>::new(ring.proxy());
    let dbus_monitor = DbusMonitor::new(dbus_session.proxy());
    let backend = FileChooserBackend::new(dbus_session.proxy());
    let bt_backend = BluetoothBackend::new(dbus_system.proxy());
    let window_manager = WindowManager::new(ring.proxy());

    let app_root = AppRoot {
        backend,
        bt_backend,
        dbus_monitor,
        window_manager,
        dbus_session,
        dbus_system,
        ring,
    };

    let mut app = App::new(app_root)
        .mount(main_poll_module())
        .mount(dbus_monitor_module())
        .mount(filechooser_module())
        .mount(bluetooth_module())
        .mount(dbus_module::<SessionBus, _>())
        .mount(dbus_module::<SystemBus, _>())
        .mount(window_manager::module())
        .mount(io_ring::module());

    println!("[main] Starting application event loop.");
    app.dispatch(&app::Start);
    loop {
        app.dispatch(&app::PrePoll);
        app.dispatch(&app::Poll);
    }
}
