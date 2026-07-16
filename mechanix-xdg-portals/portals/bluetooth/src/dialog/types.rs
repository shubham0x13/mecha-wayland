use crate::backend::BluetoothRequest;

/// What the dialog needs to display / what action it expects from the user.
#[derive(Debug, Clone)]
pub enum DialogKind {
    /// Show a PIN code — user just dismisses the dialog.
    DisplayPinCode { pincode: String },
    /// Show a numeric passkey — user just dismisses.
    DisplayPasskey { passkey: u32, entered: u16 },
    /// Ask the user to type a legacy PIN string. Returns PinCode(pin) or Rejected.
    RequestPinCode,
    /// Ask the user to type a numeric passkey (0–999999). Returns Passkey(n) or Rejected.
    RequestPasskey,
    /// Ask the user to confirm a passkey matches. Returns Accepted / Rejected.
    RequestConfirmation { passkey: u32 },
    /// Ask the user to authorize a generic device connection. Returns Accepted / Rejected.
    RequestAuthorization,
    /// Ask the user to authorize a specific Bluetooth service. Returns Accepted / Rejected.
    AuthorizeService { uuid: String },
}

pub struct DialogArgs {
    pub device: String,
    pub kind: DialogKind,
}

impl DialogArgs {
    /// Map a `BluetoothRequest` to dialog arguments.
    /// Returns `None` for `Cancel` (which is handled separately by the coordinator).
    pub fn from_request(req: &BluetoothRequest) -> Option<Self> {
        match req {
            BluetoothRequest::DisplayPinCode { device, pincode } => Some(Self {
                device: device.clone(),
                kind: DialogKind::DisplayPinCode {
                    pincode: pincode.clone(),
                },
            }),
            BluetoothRequest::DisplayPasskey {
                device,
                passkey,
                entered,
            } => Some(Self {
                device: device.clone(),
                kind: DialogKind::DisplayPasskey {
                    passkey: *passkey,
                    entered: *entered,
                },
            }),
            BluetoothRequest::RequestConfirmation { device, passkey } => Some(Self {
                device: device.clone(),
                kind: DialogKind::RequestConfirmation { passkey: *passkey },
            }),
            BluetoothRequest::RequestAuthorization { device } => Some(Self {
                device: device.clone(),
                kind: DialogKind::RequestAuthorization,
            }),
            BluetoothRequest::AuthorizeService { device, uuid } => Some(Self {
                device: device.clone(),
                kind: DialogKind::AuthorizeService { uuid: uuid.clone() },
            }),
            BluetoothRequest::RequestPinCode { device } => Some(Self {
                device: device.clone(),
                kind: DialogKind::RequestPinCode,
            }),
            BluetoothRequest::RequestPasskey { device } => Some(Self {
                device: device.clone(),
                kind: DialogKind::RequestPasskey,
            }),
            BluetoothRequest::Cancel => None,
        }
    }
}
