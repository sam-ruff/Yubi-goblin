use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Dependencies {
    #[schema(example = true)]
    /// Set to true if apt is installed
    pub(crate) apt: bool,
    #[schema(example = true)]
    /// Set to true if libpam_u2f is installed
    pub(crate) libpam_u2f: bool,
    #[schema(example = true)]
    /// Set to true if pamu2fg is installed
    pub(crate) pamu2fcfg: bool,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct YubiKey {
    #[schema(example = true)]
    /// A friendly name of the USB device
    pub(crate) name: String,
    /// The port the USB device is plugged into
    pub(crate) usb_port: i32,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ErrorMessage {
    #[schema(example = true)]
    /// A friendly name of the USB device
    pub message: String,
    pub error: bool,
}

impl Default for ErrorMessage {
    fn default() -> Self {
        Self {
            message: String::new(),
            error: true,
        }
    }
}