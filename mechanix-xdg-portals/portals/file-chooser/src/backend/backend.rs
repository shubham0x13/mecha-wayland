use std::collections::HashMap;
use std::rc::Rc;

use app::{RegisteredModule, prelude::*};
use dbus::{DbusEvent, DbusMessage, DbusProxy, IncomingCall, Pending, SessionBus, fdo, variant};
use zbus::message::Message;

use super::interface::{
    FILECHOOSER_IFACE, FILECHOOSER_VERSION, FileChooser, OpenFile, SaveFile, SaveFiles,
};
use super::types::{
    FileChooserOutcome, FileChooserRequest, FileChooserResponse, FileChooserResults, RequestHandle,
};
use portal_core::{PORTAL_NAME, PORTAL_PATH, RESPONSE_CANCELLED, RESPONSE_SUCCESS, RequestClose};

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

    fn stash_pending(
        &mut self,
        handle: &zbus::zvariant::OwnedObjectPath,
        raw: &Rc<Message>,
    ) -> RequestHandle {
        let handle_str = handle.as_str().to_string();
        self.pending.insert(handle_str.clone(), Rc::clone(raw));
        handle_str
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
}

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
                        println!("[file-chooser] session bus reconnected, re-bootstrapping");
                        s.bootstrap();
                        return None;
                    }
                    DbusMessage::Disconnected => {
                        s.pending.clear();
                        s.request_name.clear();
                        s.owned = false;
                        println!("[file-chooser] session bus disconnected");
                        return None;
                    }
                    _ => {}
                }

                // RequestName reply
                if let Some((_, res)) = s.request_name.resolve(&ev.msg) {
                    match res {
                        Ok(code)
                            if code == fdo::REQUEST_NAME_PRIMARY_OWNER
                                || code == fdo::REQUEST_NAME_ALREADY_OWNER =>
                        {
                            s.owned = true;
                            println!("[file-chooser] serving {PORTAL_NAME}");
                        }
                        Ok(code) => eprintln!("[file-chooser] could not own {PORTAL_NAME} (code {code})"),
                        Err(e) => eprintln!("[file-chooser] RequestName failed: {e}"),
                    }
                    return None;
                }

                // OpenFile
                if let Some(Ok(call)) = IncomingCall::<OpenFile>::try_from(&ev.msg) {
                    let (handle, app_id, _parent, title, _options) = &call.args;
                    println!("[file-chooser] OpenFile: app_id={app_id} title='{title}' handle={}", handle.as_str());
                    return Some(FileChooserRequest::OpenFile {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: _options.clone(),
                    });
                }

                // SaveFile
                if let Some(Ok(call)) = IncomingCall::<SaveFile>::try_from(&ev.msg) {
                    let (handle, app_id, _parent, title, _options) = &call.args;
                    println!("[file-chooser] SaveFile: app_id={app_id} title='{title}' handle={}", handle.as_str());
                    return Some(FileChooserRequest::SaveFile {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: _options.clone(),
                    });
                }

                // SaveFiles
                if let Some(Ok(call)) = IncomingCall::<SaveFiles>::try_from(&ev.msg) {
                    let (handle, app_id, _parent, title, _options) = &call.args;
                    println!("[file-chooser] SaveFiles: app_id={app_id} title='{title}' handle={}", handle.as_str());
                    return Some(FileChooserRequest::SaveFiles {
                        handle: s.stash_pending(handle, call.raw()),
                        title: title.clone(),
                        options: _options.clone(),
                    });
                }

                // Request.Close -> cancel
                if let Some(Ok(call)) = IncomingCall::<RequestClose>::try_from(&ev.msg) {
                    call.respond(&s.proxy, &());
                    if let Some(handle) = &call.path {
                        let handle = handle.clone();
                        s.finish_dialog(&handle, FileChooserOutcome::Cancelled);
                        return Some(FileChooserRequest::Close { handle });
                    }
                    return None;
                }

                // Properties: read-only `version`
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

                // Standard interfaces
                if FileChooser::handle_standard(&s.proxy, PORTAL_PATH, &ev.msg) {
                    return None;
                }

                // Unknown method call
                if let DbusMessage::Call(m) = &ev.msg {
                    if m.header().path().is_some_and(|p| p.as_str() == PORTAL_PATH) {
                        eprintln!(
                            "[file-chooser] unknown method: interface={:?} member={:?}",
                            m.header().interface(),
                            m.header().member()
                        );
                        s.proxy.reply_unknown_method(m);
                    }
                }

                None
            },
        )
}
