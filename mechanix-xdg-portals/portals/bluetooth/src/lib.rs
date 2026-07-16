pub mod backend;
pub mod dialog;

use app::RegisteredModule;
pub use backend::BluetoothBackend;

pub fn bluetooth_module<S>() -> impl app::RegisteredModule<S, S>
where
    S: app::Lens<BluetoothBackend> + app::Lens<window_manager::WindowManager> + 'static,
{
    app::Module::<S, _, _, _>::new()
        .mount(backend::bluetooth_module::<S>().into_module())
        .mount(dialog::bluetooth_ui_module::<S>().into_module())
}
