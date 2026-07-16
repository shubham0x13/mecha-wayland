use std::collections::HashMap;
use std::rc::Rc;

use app::{RegisteredModule, prelude::*};
use dbus::{DbusEvent, DbusMessage, DbusProxy, IncomingCall, Pending, SystemBus};
use zbus::message::Message;
use zbus::zvariant::OwnedObjectPath;

use super::interface::{
    AGENT_PATH, AuthorizeService, BlueZAgent, Cancel, DisplayPasskey, DisplayPinCode,
    RegisterAgent, Release, RequestAuthorization, RequestConfirmation, RequestDefaultAgent,
    RequestPasskey, RequestPinCode,
};
use super::types::{
    AgentCapability, BluetoothOutcome, BluetoothRequest, BluetoothResponse, BtCallId,
};

pub const AGENT_CAPABILITY: AgentCapability = AgentCapability::DisplayYesNo;

#[derive(State)]
pub struct BluetoothBackend {
    proxy: DbusProxy<SystemBus>,
    register_agent: Pending<RegisterAgent>,
    request_default_agent: Pending<RequestDefaultAgent>,
    pending: HashMap<BtCallId, Rc<Message>>,
    next_id: BtCallId,
}

impl BluetoothBackend {
    pub fn new(proxy: DbusProxy<SystemBus>) -> Self {
        Self {
            proxy,
            register_agent: Pending::new(),
            request_default_agent: Pending::new(),
            pending: HashMap::new(),
            next_id: 0,
        }
    }

    fn bootstrap(&mut self) {
        let path = OwnedObjectPath::try_from(AGENT_PATH).expect("valid agent path");
        self.register_agent
            .call(&self.proxy, &(path, AGENT_CAPABILITY.to_string()), ());
    }

    /// Allocate the next call-id.
    fn next_call_id(&mut self) -> BtCallId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Stash a raw message (for request variants) and return its call-id.
    fn stash(&mut self, raw: &Rc<Message>) -> BtCallId {
        let id = self.next_call_id();
        self.pending.insert(id, Rc::clone(raw));
        id
    }

    /// Complete a pending request. `Dismissed` is a no-op (display variants).
    fn finish(&mut self, call_id: BtCallId, outcome: BluetoothOutcome) {
        let Some(raw) = self.pending.remove(&call_id) else {
            // Display-only dialogs or unknown ids — nothing to reply.
            return;
        };
        match outcome {
            BluetoothOutcome::Accepted => {
                self.proxy.reply(&raw, &());
            }
            BluetoothOutcome::Rejected | BluetoothOutcome::Dismissed => {
                self.proxy
                    .reply_error(&raw, "org.bluez.Error.Rejected", "rejected by user");
            }
        }
    }

    /// BlueZ sent Cancel — reject every stashed call and return their ids so
    /// the UI module can close the corresponding windows.
    fn cancel_all(&mut self) {
        for (_, raw) in self.pending.drain() {
            self.proxy
                .reply_error(&raw, "org.bluez.Error.Cancelled", "cancelled by BlueZ");
        }
    }
}

// --- Module ------------------------------------------------------------------

pub fn bluetooth_module<S>() -> impl RegisteredModule<BluetoothBackend, S> {
    Module::<BluetoothBackend, _, _>::new()
        // Bootstrap: register our agent with BlueZ on startup.
        .on(|s: &mut BluetoothBackend, _: &app::Start| s.bootstrap())
        // UI reply → finish the stashed D-Bus call.
        .on(|s: &mut BluetoothBackend, resp: &BluetoothResponse| {
            s.finish(resp.call_id, resp.outcome.clone());
        })
        // D-Bus events from the system bus.
        .on(
            |s: &mut BluetoothBackend, ev: &DbusEvent<SystemBus>| -> Option<BluetoothRequest> {
                // --- Connection lifecycle ------------------------------------
                match &ev.msg {
                    DbusMessage::Reconnected => {
                        println!("[bt] System bus reconnected. Re-registering agent...");
                        s.bootstrap();
                        return None;
                    }
                    DbusMessage::Disconnected => {
                        s.pending.clear();
                        s.register_agent.clear();
                        s.request_default_agent.clear();
                        println!("[bt] System bus disconnected.");
                        return None;
                    }
                    _ => {}
                }

                // --- Outgoing call replies ------------------------------------

                // RegisterAgent reply
                if let Some((_, res)) = s.register_agent.resolve(&ev.msg) {
                    match res {
                        Ok(()) => {
                            println!(
                                "[bt] Agent registered at {AGENT_PATH} (cap={AGENT_CAPABILITY})"
                            );
                            // Also request to be the default agent.
                            let path =
                                OwnedObjectPath::try_from(AGENT_PATH).expect("valid agent path");
                            s.request_default_agent.call(&s.proxy, &(path,), ());
                        }
                        Err(e) => eprintln!("[bt] RegisterAgent failed: {e}"),
                    }
                    return None;
                }

                // RequestDefaultAgent reply
                if let Some((_, res)) = s.request_default_agent.resolve(&ev.msg) {
                    match res {
                        Ok(()) => println!("[bt] Set as default agent."),
                        Err(e) => eprintln!("[bt] RequestDefaultAgent: {e}"),
                    }
                    return None;
                }

                // --- Incoming Agent1 method calls from BlueZ -----------------

                // Release — BlueZ is unregistering us.
                if let Some(Ok(call)) = IncomingCall::<Release>::try_from(&ev.msg) {
                    call.respond(&s.proxy, &());
                    println!("[bt] Agent released by BlueZ.");
                    return None;
                }

                // DisplayPinCode — fire-and-forget; reply immediately.
                if let Some(Ok(call)) = IncomingCall::<DisplayPinCode>::try_from(&ev.msg) {
                    let (device, pincode) = &call.args;
                    let call_id = s.next_call_id(); // NOT stashed; reply sent now
                    call.respond(&s.proxy, &());
                    return Some(BluetoothRequest::DisplayPinCode {
                        call_id,
                        device: device.as_str().to_string(),
                        pincode: pincode.clone(),
                    });
                }

                // DisplayPasskey — fire-and-forget; reply immediately.
                if let Some(Ok(call)) = IncomingCall::<DisplayPasskey>::try_from(&ev.msg) {
                    let (device, passkey, entered) = &call.args;
                    let call_id = s.next_call_id();
                    call.respond(&s.proxy, &());
                    return Some(BluetoothRequest::DisplayPasskey {
                        call_id,
                        device: device.as_str().to_string(),
                        passkey: *passkey,
                        entered: *entered,
                    });
                }

                // RequestPinCode — reject (no text-input widget yet).
                if let Some(Ok(call)) = IncomingCall::<RequestPinCode>::try_from(&ev.msg) {
                    eprintln!("[bt] RequestPinCode: rejecting (PIN input not supported).");
                    call.error(
                        &s.proxy,
                        "org.bluez.Error.Rejected",
                        "PIN input not supported",
                    );
                    return None;
                }

                // RequestPasskey — reject (no text-input widget yet).
                if let Some(Ok(call)) = IncomingCall::<RequestPasskey>::try_from(&ev.msg) {
                    eprintln!("[bt] RequestPasskey: rejecting (passkey input not supported).");
                    call.error(
                        &s.proxy,
                        "org.bluez.Error.Rejected",
                        "passkey input not supported",
                    );
                    return None;
                }

                // RequestConfirmation — stash and ask the UI.
                if let Some(Ok(call)) = IncomingCall::<RequestConfirmation>::try_from(&ev.msg) {
                    let (device, passkey) = &call.args;
                    let call_id = s.stash(call.raw());
                    println!(
                        "[bt] RequestConfirmation: device={} passkey={}",
                        device.as_str(),
                        passkey
                    );
                    return Some(BluetoothRequest::RequestConfirmation {
                        call_id,
                        device: device.as_str().to_string(),
                        passkey: *passkey,
                    });
                }

                // RequestAuthorization — stash and ask the UI.
                if let Some(Ok(call)) = IncomingCall::<RequestAuthorization>::try_from(&ev.msg) {
                    let (device,) = &call.args;
                    let call_id = s.stash(call.raw());
                    println!("[bt] RequestAuthorization: device={}", device.as_str());
                    return Some(BluetoothRequest::RequestAuthorization {
                        call_id,
                        device: device.as_str().to_string(),
                    });
                }

                // AuthorizeService — stash and ask the UI.
                if let Some(Ok(call)) = IncomingCall::<AuthorizeService>::try_from(&ev.msg) {
                    let (device, uuid) = &call.args;
                    let call_id = s.stash(call.raw());
                    println!(
                        "[bt] AuthorizeService: device={} uuid={}",
                        device.as_str(),
                        uuid
                    );
                    return Some(BluetoothRequest::AuthorizeService {
                        call_id,
                        device: device.as_str().to_string(),
                        uuid: uuid.clone(),
                    });
                }

                // Cancel — BlueZ is dismissing everything.
                if let Some(Ok(call)) = IncomingCall::<Cancel>::try_from(&ev.msg) {
                    call.respond(&s.proxy, &());
                    s.cancel_all();
                    return Some(BluetoothRequest::Cancel);
                }

                // Standard interfaces (Peer, Introspectable) for our agent path.
                if BlueZAgent::handle_standard(&s.proxy, AGENT_PATH, &ev.msg) {
                    return None;
                }

                // Unknown call on our agent path.
                if let DbusMessage::Call(m) = &ev.msg {
                    if m.header().path().is_some_and(|p| p.as_str() == AGENT_PATH) {
                        s.proxy.reply_unknown_method(m);
                    }
                }

                None
            },
        )
}
