use app::Event;

/// Supported BlueZ pairing agent capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCapability {
    DisplayOnly,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

impl AgentCapability {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DisplayOnly => "DisplayOnly",
            Self::DisplayYesNo => "DisplayYesNo",
            Self::KeyboardOnly => "KeyboardOnly",
            Self::NoInputNoOutput => "NoInputNoOutput",
            Self::KeyboardDisplay => "KeyboardDisplay",
        }
    }
}

impl std::fmt::Display for AgentCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Internal serial that correlates a BlueZ call with its pending raw D-Bus message.
pub type BtCallId = u64;

/// Emitted by the D-Bus backend; consumed by the Bluetooth UI coordinator.
///
/// Display variants (DisplayPinCode / DisplayPasskey) are fire-and-forget from
/// BlueZ's point of view — the backend replies immediately before emitting the
/// event.  The UI simply shows the information; when the user dismisses the
/// window it emits a `BluetoothResponse` with `outcome: Dismissed` and any
/// `call_id` (the backend ignores unknown ids).
///
/// Request variants (RequestConfirmation / RequestAuthorization /
/// AuthorizeService) stash the raw message; the backend waits for a
/// `BluetoothResponse` before replying to BlueZ.
#[derive(Debug, Clone)]
pub enum BluetoothRequest {
    /// Show the PIN code to the user (reply already sent to BlueZ).
    DisplayPinCode {
        call_id: BtCallId,
        device: String,
        pincode: String,
    },
    /// Show the passkey with a "keys entered so far" counter (reply sent).
    DisplayPasskey {
        call_id: BtCallId,
        device: String,
        passkey: u32,
        entered: u16,
    },
    /// User must confirm that the passkey matches (reply pending).
    RequestConfirmation {
        call_id: BtCallId,
        device: String,
        passkey: u32,
    },
    /// User must authorize the device connection (reply pending).
    RequestAuthorization { call_id: BtCallId, device: String },
    /// User must authorize a specific service UUID (reply pending).
    AuthorizeService {
        call_id: BtCallId,
        device: String,
        uuid: String,
    },
    /// BlueZ cancelled — close any open Bluetooth dialog.
    Cancel,
}

impl Event for BluetoothRequest {}

/// Emitted by the Bluetooth UI when the user acts; consumed by the backend.
#[derive(Debug)]
pub struct BluetoothResponse {
    pub call_id: BtCallId,
    pub outcome: BluetoothOutcome,
}
impl Event for BluetoothResponse {}

#[derive(Debug, Clone)]
pub enum BluetoothOutcome {
    /// User confirmed / allowed.
    Accepted,
    /// User rejected / denied.
    Rejected,
    /// Display-only dialog dismissed — no D-Bus reply needed.
    Dismissed,
}
