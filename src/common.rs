use crate::{EmbeddedController, daemon::client::DaemonClient};
use anyhow::{Result, bail};
use enum_dispatch::enum_dispatch;

/// Adapter like type which connect to daemon if available or write to EC by itself
#[enum_dispatch(ECHandler)]
pub enum Handler {
    EmbeddedController,
    DaemonClient,
}

pub const EXIT_MSG: &str =
    "Run Gigabyte Linux as root or install systemd service with `gigabyte-linux daemon install`";

impl Handler {
    pub async fn new() -> Result<Self> {
        let daemon_client = DaemonClient::connect().await;
        if let Ok(dc) = daemon_client {
            return Ok(Handler::from(dc));
        }
        let ec = EmbeddedController::new().await;
        if let Ok(ec) = ec {
            return Ok(Handler::from(ec));
        }
        bail!("{EXIT_MSG}")
    }
}
