use app::{RegisteredModule, prelude::*};
use dbus::{Bus, DbusEvent, DbusMessage, DbusProxy};
use crate::RECONNECT_INTERVAL_FRAMES;

#[derive(State)]
pub struct DbusMonitor<B: Bus> {
    proxy: DbusProxy<B>,
    #[lens(skip)]
    disconnected: bool,
    #[lens(skip)]
    retry_tick: u32,
}

impl<B: Bus> DbusMonitor<B> {
    pub fn new(proxy: DbusProxy<B>) -> Self {
        Self {
            proxy,
            disconnected: false,
            retry_tick: 0,
        }
    }
}

pub fn dbus_monitor_module<B, S>() -> impl RegisteredModule<DbusMonitor<B>, S>
where
    B: Bus,
    S: Lens<DbusMonitor<B>> + 'static,
{
    Module::<DbusMonitor<B>, _, _>::new()
        .on(|s: &mut DbusMonitor<B>, _: &app::PrePoll| {
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
                    println!("[dbus] Reconnected to the {} bus socket.", B::NAME);
                }
                Err(e) => {
                    eprintln!("[dbus] Reconnect attempt failed for {}: {e}", B::NAME);
                }
            }
        })
        .on(|s: &mut DbusMonitor<B>, ev: &DbusEvent<B>| {
            if let DbusMessage::Disconnected = &ev.msg {
                s.disconnected = true;
                s.retry_tick = 0;
                println!("[dbus] {} bus disconnected. Scheduling reconnection...", B::NAME);
            }
        })
}
