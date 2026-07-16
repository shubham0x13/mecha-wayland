use crate::backend::{BluetoothRequest, BtCallId};

/// What the dialog needs to display / what action it expects from the user.
#[derive(Debug, Clone)]
pub enum DialogKind {
    /// Show a PIN code — user just dismisses the dialog.
    DisplayPinCode { pincode: String },
    /// Show a numeric passkey — user just dismisses.
    DisplayPasskey { passkey: u32, entered: u16 },
    /// Ask the user to confirm a passkey matches. Returns Accepted / Rejected.
    RequestConfirmation { passkey: u32 },
    /// Ask the user to authorize a generic device connection. Returns Accepted / Rejected.
    RequestAuthorization,
    /// Ask the user to authorize a specific Bluetooth service. Returns Accepted / Rejected.
    AuthorizeService { uuid: String },
}

pub struct DialogArgs {
    pub call_id: BtCallId,
    pub device: String,
    pub kind: DialogKind,
}

impl DialogArgs {
    /// Map a `BluetoothRequest` to dialog arguments.
    /// Returns `None` for `Cancel` (which is handled separately by the coordinator).
    pub fn from_request(req: &BluetoothRequest) -> Option<Self> {
        match req {
            BluetoothRequest::DisplayPinCode {
                call_id,
                device,
                pincode,
            } => Some(Self {
                call_id: *call_id,
                device: device.clone(),
                kind: DialogKind::DisplayPinCode {
                    pincode: pincode.clone(),
                },
            }),
            BluetoothRequest::DisplayPasskey {
                call_id,
                device,
                passkey,
                entered,
            } => Some(Self {
                call_id: *call_id,
                device: device.clone(),
                kind: DialogKind::DisplayPasskey {
                    passkey: *passkey,
                    entered: *entered,
                },
            }),
            BluetoothRequest::RequestConfirmation {
                call_id,
                device,
                passkey,
            } => Some(Self {
                call_id: *call_id,
                device: device.clone(),
                kind: DialogKind::RequestConfirmation { passkey: *passkey },
            }),
            BluetoothRequest::RequestAuthorization { call_id, device } => Some(Self {
                call_id: *call_id,
                device: device.clone(),
                kind: DialogKind::RequestAuthorization,
            }),
            BluetoothRequest::AuthorizeService {
                call_id,
                device,
                uuid,
            } => Some(Self {
                call_id: *call_id,
                device: device.clone(),
                kind: DialogKind::AuthorizeService { uuid: uuid.clone() },
            }),
            BluetoothRequest::Cancel => None,
        }
    }
}
