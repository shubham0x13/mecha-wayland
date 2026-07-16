use dbus::{dbus_interface, dbus_method};
use zbus::zvariant::OwnedObjectPath;

pub const AGENT_IFACE: &str = "org.bluez.Agent1";
pub const AGENT_PATH: &str = "/org/mechanix/bluetooth/agent";

pub const AGENT_MANAGER_DEST: &str = "org.bluez";
pub const AGENT_MANAGER_PATH: &str = "/org/bluez";
pub const AGENT_MANAGER_IFACE: &str = "org.bluez.AgentManager1";

// BlueZ Agent API:
// <https://bluez.readthedocs.io/en/latest/agent-api/#agent-hierarchy>
dbus_interface!(
    pub BlueZAgent = AGENT_IFACE;
    method Release() -> ();
    method RequestPinCode(device: OwnedObjectPath) -> (pincode: String);
    method DisplayPinCode(device: OwnedObjectPath, pincode: String) -> ();
    method RequestPasskey(device: OwnedObjectPath) -> (passkey: u32);
    method DisplayPasskey(device: OwnedObjectPath, passkey: u32, entered: u16) -> ();
    method RequestConfirmation(device: OwnedObjectPath, passkey: u32) -> ();
    method RequestAuthorization(device: OwnedObjectPath) -> ();
    method AuthorizeService(device: OwnedObjectPath, uuid: String) -> ();
    method Cancel() -> ();
);

// BlueZ Agent Manager API:
// https://bluez.readthedocs.io/en/latest/agent-api/#agent-manager-hierarchy
dbus_method!(pub RegisterAgent {
    dest: AGENT_MANAGER_DEST,
    path: AGENT_MANAGER_PATH,
    iface: AGENT_MANAGER_IFACE,
    member: "RegisterAgent",
    args: (OwnedObjectPath, String),
    reply: (),
});

dbus_method!(pub RequestDefaultAgent {
    dest: AGENT_MANAGER_DEST,
    path: AGENT_MANAGER_PATH,
    iface: AGENT_MANAGER_IFACE,
    member: "RequestDefaultAgent",
    args: (OwnedObjectPath,),
    reply: (),
});

dbus_method!(pub UnregisterAgent {
    dest: AGENT_MANAGER_DEST,
    path: AGENT_MANAGER_PATH,
    iface: AGENT_MANAGER_IFACE,
    member: "UnregisterAgent",
    args: (OwnedObjectPath,),
    reply: (),
});
