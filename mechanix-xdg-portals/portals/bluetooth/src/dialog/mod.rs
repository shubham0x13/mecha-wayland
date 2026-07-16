mod dialog;
mod types;

use crate::backend::{BluetoothRequest, BluetoothResponse};
use std::cell::Cell;
use window_manager::{WindowId, WindowKind, WindowManager, WindowSettings};

pub use dialog::BluetoothDialogUi;
pub use types::{DialogArgs, DialogKind};

// --- Thread-local slots ------------------------------------------------------

thread_local! {
    /// Set by the dialog widget when the user acts; drained each Poll tick.
    pub static PENDING_BT_RESPONSE: Cell<Option<BluetoothResponse>> =
        const { Cell::new(None) };
    /// The single active Bluetooth dialog window, if any.
    pub static ACTIVE_BT_WINDOW: Cell<Option<WindowId>> =
        const { Cell::new(None) };
}

// --- Coordinator module ------------------------------------------------------

/// Mount this on the `WindowManager` state slice (exactly like
/// `filechooser_ui_module`).
pub fn bluetooth_ui_module<S>() -> impl app::RegisteredModule<WindowManager, S>
where
    S: app::Lens<WindowManager> + 'static,
{
    app::Module::<WindowManager, _, _>::new()
        .on(|wm: &mut WindowManager, req: &BluetoothRequest| {
            // Cancel: close the open bluetooth dialog (if any).
            if let BluetoothRequest::Cancel = req {
                println!("[bt-ui] Cancel — closing Bluetooth dialog.");
                if let Some(id) = ACTIVE_BT_WINDOW.get() {
                    wm.destroy(id);
                    ACTIVE_BT_WINDOW.set(None);
                }
                return;
            }

            // Map the request to dialog arguments.
            let Some(args) = DialogArgs::from_request(req) else {
                return;
            };

            // Close any existing dialog before opening a new one.
            if let Some(old) = ACTIVE_BT_WINDOW.get() {
                println!("[bt-ui] Replacing existing Bluetooth dialog.");
                wm.destroy(old);
                ACTIVE_BT_WINDOW.set(None);
            }

            println!(
                "[bt-ui] Spawning Bluetooth dialog (kind={:?}, device={}).",
                args.kind, args.device
            );

            let title = format!("Bluetooth — {}", args.device);
            let id = wm.spawn_window(
                WindowSettings {
                    width: 640,
                    height: 480,
                    clear_color: window_manager::Color::rgb(0.06, 0.06, 0.08),
                    kind: WindowKind::Xdg { title },
                    touch_config: None,
                    gesture_config: None,
                },
                BluetoothDialogUi::new(&args.device, &args.kind),
            );

            ACTIVE_BT_WINDOW.set(Some(id));
            wm.flush_pending();
        })
        .on(
            |wm: &mut WindowManager, _: &app::Poll| -> Option<BluetoothResponse> {
                let done = PENDING_BT_RESPONSE.take();
                if done.is_some() {
                    // Close the dialog window that produced this response.
                    if let Some(id) = ACTIVE_BT_WINDOW.get() {
                        println!(
                            "[bt-ui] Dialog done (outcome={:?}). Closing window.",
                            done.as_ref().map(|r| &r.outcome)
                        );
                        wm.destroy(id);
                        ACTIVE_BT_WINDOW.set(None);
                    }
                }
                done
            },
        )
}
