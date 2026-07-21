mod dialog;
mod types;

use crate::backend::{FileChooserRequest, FileChooserResponse, RequestHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use window_manager::{WindowId, WindowKind, WindowManager, WindowSettings};

pub use dialog::FileChooserUi;
pub use types::ChooserOptions;

// Coordinator module to dispatch UI commands and monitor dialog actions.
pub fn filechooser_ui_module<S>() -> impl app::RegisteredModule<WindowManager, S>
where
    S: app::Lens<WindowManager> + 'static,
{
    let active: Rc<RefCell<HashMap<RequestHandle, WindowId>>> =
        Rc::new(RefCell::new(HashMap::new()));

    app::Module::<WindowManager, _, _>::new()
        .mount(ui::register_events!(FileChooserResponse))
        .on({
            let active = active.clone();
            move |wm: &mut WindowManager, cmd: &FileChooserRequest| {
                let spawn_args = match cmd {
                    FileChooserRequest::OpenFile { handle, options, .. } => {
                        Some((handle.clone(), ChooserOptions::OpenFile(options.clone())))
                    }
                    FileChooserRequest::SaveFile { handle, options, .. } => {
                        Some((handle.clone(), ChooserOptions::SaveFile(options.clone())))
                    }
                    FileChooserRequest::SaveFiles { handle, options, .. } => {
                        Some((handle.clone(), ChooserOptions::SaveFiles(options.clone())))
                    }
                    FileChooserRequest::Close { handle } => {
                        if let Some(id) = active.borrow_mut().remove(handle) {
                            wm.destroy(id);
                        }
                        None
                    }
                };

                if let Some((handle, chooser_options)) = spawn_args {
                    let id = wm.spawn_window(
                        WindowSettings {
                            width: 800,
                            height: 600,
                            clear_color: window_manager::Color::rgb(0.08, 0.08, 0.1),
                            kind: WindowKind::Xdg { title: "File Picker".to_string() },
                            touch_config: None,
                            gesture_config: None,
                        },
                        FileChooserUi::new(handle.clone(), chooser_options),
                    );
                    active.borrow_mut().insert(handle, id);
                    wm.flush_pending();
                }
            }
        })
        .on(move |wm: &mut WindowManager, resp: &FileChooserResponse| {
            if let Some(id) = active.borrow_mut().remove(&resp.handle) {
                wm.destroy(id);
            }
        })
}
