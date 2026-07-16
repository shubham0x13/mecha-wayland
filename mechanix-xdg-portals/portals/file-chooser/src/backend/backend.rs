use std::collections::HashMap;
use std::rc::Rc;

use app::{prelude::*, RegisteredModule};
use dbus::{fdo, variant, DbusEvent, DbusMessage, DbusProxy, IncomingCall, Pending, SessionBus};
use zbus::message::Message;

use super::interface::{
    FileChooser, OpenFile, SaveFile, SaveFiles, FILECHOOSER_IFACE, FILECHOOSER_VERSION,
};
use super::types::{
    FileChooserOutcome, FileChooserRequest, FileChooserResponse, FileChooserResults, RequestHandle,
};
use portal_core::{RequestClose, PORTAL_NAME, PORTAL_PATH, RESPONSE_CANCELLED, RESPONSE_SUCCESS};

// --- Backend state -----------------------------------------------------------
#[derive(State)]
pub struct FileChooserBackend {
    proxy: DbusProxy<SessionBus>,
    request_name: Pending<fdo::RequestName>,
    pending: HashMap<RequestHandle, Rc<Message>>,
    owned: bool,
}

impl FileChooserBackend {
    pub fn new(proxy: DbusProxy<SessionBus>) -> Self {
        Self {
            proxy,
            request_name: Pending::new(),
            pending: HashMap::new(),
            owned: false,
        }
    }

    fn bootstrap(&mut self) {
        self.request_name.call(
            &self.proxy,
            &(PORTAL_NAME.to_string(), fdo::NAME_DO_NOT_QUEUE),
            (),
        );
    }

    fn finish_dialog(&mut self, handle: &str, outcome: FileChooserOutcome) {
        let Some(raw) = self.pending.remove(handle) else {
            return;
        };
        let (response, results) = match outcome {
            FileChooserOutcome::Selected(uris) => {
                (RESPONSE_SUCCESS, FileChooserResults { uris: Some(uris) })
            }
            FileChooserOutcome::Cancelled => {
                (RESPONSE_CANCELLED, FileChooserResults { uris: None })
            }
        };
        self.proxy.reply(&raw, &(response, results));
    }

    fn stash_pending(
        &mut self,
        handle: &zbus::zvariant::OwnedObjectPath,
        raw: &Rc<Message>,
    ) -> RequestHandle {
        let handle_str = handle.as_str().to_string();
        self.pending.insert(handle_str.clone(), Rc::clone(raw));
        handle_str
    }
}

// --- Module registration -----------------------------------------------------
pub fn filechooser_module<S>() -> impl RegisteredModule<FileChooserBackend, S> {
    Module::<FileChooserBackend, _, _>::new()
        .on(|s: &mut FileChooserBackend, _: &app::Start| s.bootstrap())
        .on(|s: &mut FileChooserBackend, done: &FileChooserResponse| {
            s.finish_dialog(&done.handle, done.outcome.clone());
        })
        .on(
            |s: &mut FileChooserBackend,
             ev: &DbusEvent<SessionBus>|
             -> Option<FileChooserRequest> {
                match &ev.msg {
                    DbusMessage::Reconnected => {
                        println!(
                            "FileChooser backend reconnected. Re-bootstrapping name ownership..."
                        );
                        s.bootstrap();
                        return None;
                    }
                    DbusMessage::Disconnected => {
                        s.pending.clear();
                        s.request_name.clear();
                        s.owned = false;
                        println!("FileChooser backend disconnected from D-Bus.");
                        return None;
                    }
                    _ => {}
                }

                // RequestName reply.
                if let Some((_, res)) = s.request_name.resolve(&ev.msg) {
                    match res {
                        Ok(code)
                            if code == fdo::REQUEST_NAME_PRIMARY_OWNER
                                || code == fdo::REQUEST_NAME_ALREADY_OWNER =>
                        {
                            s.owned = true;
                            println!("FileChooser backend serving {PORTAL_NAME}");
                        }
                        Ok(code) => eprintln!("could not own {PORTAL_NAME} (code {code})"),
                        Err(e) => eprintln!("RequestName failed: {e}"),
                    }
                    return None;
                }

                // OpenFile / SaveFile / SaveFiles -> open the dialog.
                if let Some(Ok(call)) = IncomingCall::<OpenFile>::try_from(&ev.msg) {
                    let (handle, _app_id, _parent, title, options) = &call.args;
                    return Some(FileChooserRequest::OpenFile {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: options.clone(),
                    });
                }
                if let Some(Ok(call)) = IncomingCall::<SaveFile>::try_from(&ev.msg) {
                    let (handle, _app_id, _parent, title, options) = &call.args;
                    return Some(FileChooserRequest::SaveFile {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: options.clone(),
                    });
                }
                if let Some(Ok(call)) = IncomingCall::<SaveFiles>::try_from(&ev.msg) {
                    let (handle, _app_id, _parent, title, options) = &call.args;
                    return Some(FileChooserRequest::SaveFiles {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: options.clone(),
                    });
                }

                // Request.Close -> cancel.
                if let Some(Ok(call)) = IncomingCall::<RequestClose>::try_from(&ev.msg) {
                    call.respond(&s.proxy, &());
                    if let Some(handle) = &call.path {
                        let handle = handle.clone();
                        s.finish_dialog(&handle, FileChooserOutcome::Cancelled);
                        return Some(FileChooserRequest::Close { handle });
                    }
                    return None;
                }

                // Properties: read-only `version`.
                if fdo::route_properties(
                    &s.proxy,
                    &ev.msg,
                    FILECHOOSER_IFACE,
                    &["version"],
                    |access| match access {
                        fdo::PropAccess::Get("version") => {
                            fdo::PropReply::Value(variant(FILECHOOSER_VERSION))
                        }
                        fdo::PropAccess::Set("version", _) => fdo::PropReply::ReadOnly,
                        _ => fdo::PropReply::Unknown,
                    },
                ) {
                    return None;
                }

                // Standard interfaces (Peer.Ping / GetMachineId, Introspect).
                if FileChooser::handle_standard(&s.proxy, PORTAL_PATH, &ev.msg) {
                    return None;
                }

                // Fallback: unknown method on our portal object.
                if let DbusMessage::Call(m) = &ev.msg {
                    if m.header().path().is_some_and(|p| p.as_str() == PORTAL_PATH) {
                        s.proxy.reply_unknown_method(m);
                    }
                }
                None
            },
        )
}
