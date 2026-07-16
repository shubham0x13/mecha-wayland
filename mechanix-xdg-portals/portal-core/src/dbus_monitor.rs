use app::{RegisteredModule, prelude::*};
use dbus::{DbusEvent, DbusMessage, DbusProxy, SessionBus};
use crate::RECONNECT_INTERVAL_FRAMES;

#[derive(State)]
pub struct DbusMonitor {
    proxy: DbusProxy<SessionBus>,
    #[lens(skip)]
    disconnected: bool,
    #[lens(skip)]
    retry_tick: u32,
}

impl DbusMonitor {
    pub fn new(proxy: DbusProxy<SessionBus>) -> Self {
        Self {
            proxy,
            disconnected: false,
            retry_tick: 0,
        }
    }
}

pub fn dbus_monitor_module<S>() -> impl RegisteredModule<DbusMonitor, S>
where
    S: Lens<DbusMonitor> + 'static,
{
    Module::<DbusMonitor, _, _>::new()
        .on(|s: &mut DbusMonitor, _: &app::PrePoll| {
            if !s.disconnected {
                return;
            }
            s.retry_tick += 1;
            if s.retry_tick % RECONNECT_INTERVAL_FRAMES != 0 {
                return;
            }
            match s.proxy.reconnect() {
                Ok(()) => {
                    s.disconnected = false;
                    println!("[dbus] Reconnected to the session bus socket.");
                }
                Err(e) => {
                    eprintln!("[dbus] Reconnect attempt failed: {e}");
                }
            }
        })
        .on(|s: &mut DbusMonitor, ev: &DbusEvent<SessionBus>| {
            if let DbusMessage::Disconnected = &ev.msg {
                s.disconnected = true;
                s.retry_tick = 0;
                println!("[dbus] Session bus disconnected. Scheduling reconnection...");
            }
        })
}
