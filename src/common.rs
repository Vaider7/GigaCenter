use crate::{daemon::client::DaemonClient, EmbeddedController};
use anyhow::{bail, Result};
use enum_dispatch::enum_dispatch;

/// Adapter like type which connect to daemon if available or write to EC by itself
#[enum_dispatch(ECHandler)]
pub enum Handler {
    EmbeddedController,
    DaemonClient,
}

pub const EXIT_MSG: &str =
    "Run GigaCenter as root or install systemd service with `gigacenter daemon install`";

impl Handler {
    pub async fn new() -> Result<Self> {
        let daemon_client = DaemonClient::connect().await;
        if let Ok(dc) = daemon_client {
            log::info!("Connected to daemon");
            return Ok(Handler::from(dc));
        }
        let ec = EmbeddedController::new().await;
        if let Ok(ec) = ec {
            log::warn!("Failed to connect to daemon. Use as standalone");
            return Ok(Handler::from(ec));
        }
        bail!("{EXIT_MSG}")
    }
}
