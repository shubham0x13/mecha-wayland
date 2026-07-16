mod dialog;
mod types;

use crate::backend::{BluetoothRequest, BluetoothResponse, BtCallId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use window_manager::{WindowId, WindowKind, WindowManager, WindowSettings};

pub use dialog::BluetoothDialogUi;
pub use types::{DialogArgs, DialogKind};

// --- Thread-local slots ------------------------------------------------------

thread_local! {
    /// Set by the dialog widget when the user acts; drained each Poll tick.
    pub static PENDING_BT_RESPONSE: Cell<Option<BluetoothResponse>> =
        const { Cell::new(None) };
    /// Maps BtCallId → WindowId so we can close the right window.
    pub static ACTIVE_BT_WINDOWS: RefCell<HashMap<BtCallId, WindowId>> =
        RefCell::new(HashMap::new());
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
            // Cancel: close every open bluetooth dialog.
            if let BluetoothRequest::Cancel = req {
                println!("[bt-ui] Cancel — closing all Bluetooth dialogs.");
                ACTIVE_BT_WINDOWS.with(|wins| {
                    for (_, id) in wins.borrow_mut().drain() {
                        wm.destroy(id);
                    }
                });
                return;
            }

            // Map the request to dialog arguments.
            let Some(args) = DialogArgs::from_request(req) else {
                return;
            };

            println!(
                "[bt-ui] Spawning Bluetooth dialog (kind={:?}, device={}).",
                args.kind, args.device
            );

            let call_id = args.call_id;
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
                BluetoothDialogUi::new(call_id, &args.device, &args.kind),
            );

            ACTIVE_BT_WINDOWS.with(|wins| {
                wins.borrow_mut().insert(call_id, id);
            });
            wm.flush_pending();
        })
        .on(
            |wm: &mut WindowManager, _: &app::Poll| -> Option<BluetoothResponse> {
                let done = PENDING_BT_RESPONSE.take();
                if let Some(ref done) = done {
                    // Close the window that produced this response.
                    let id = ACTIVE_BT_WINDOWS.with(|wins| wins.borrow_mut().remove(&done.call_id));
                    if let Some(id) = id {
                        println!(
                            "[bt-ui] Dialog done (call_id={}, outcome={:?}). Closing window.",
                            done.call_id, done.outcome
                        );
                        wm.destroy(id);
                    }
                }
                done
            },
        )
}
