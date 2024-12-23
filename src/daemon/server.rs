use std::sync::Arc;

use anyhow::{Context, Result, bail};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use std::{fs, os::unix::fs::PermissionsExt};
use tokio::{net::UnixListener, sync::Mutex};

use crate::{
    EmbeddedController,
    daemon::codec::bind_transport_server,
    traits::{ECHandler, WriteResult},
};

use super::codec::{DaemonReq, DaemonResp, FramedServer};

#[cfg(feature = "self-packed")]
pub const SYSTEMD_SERVICE: &[u8] = include_bytes!("../../assets/gigacenter-daemon.service");

pub const DAEMON_UDS_PATH: &str = "/tmp/gigacenter";

pub async fn start_daemon() -> Result<()> {
    let ec = Arc::new(Mutex::new(EmbeddedController::new().await?));
    _ = tokio::fs::remove_file(DAEMON_UDS_PATH).await;
    let listener = UnixListener::bind(DAEMON_UDS_PATH).context("Create listener")?;
    let perms = fs::Permissions::from_mode(0o777);

    fs::set_permissions(DAEMON_UDS_PATH, perms)?;
    info!("Daemon ready for incoming connections");
    loop {
        match listener.accept().await.context("Create IPC listener") {
            Ok((stream, _)) => {
                let cloned = ec.clone();
                let transport = bind_transport_server(stream);
                _ = tokio::spawn(async {
                    if let Err(err) = handle_incoming(transport, cloned).await {
                        error!("Error daemon request: {err}");
                    };
                });
            }
            Err(err) => bail!(err),
        }
    }
}

pub async fn handle_incoming(
    mut stream: FramedServer,
    ec: Arc<Mutex<EmbeddedController>>,
) -> Result<()> {
    loop {
        let Some(req) = stream.next().await else {
            info!("Connection finished");
            return Ok(());
        };
        let Ok(req) = req else {
            bail!("Unknown daemon request");
        };
        match req {
            DaemonReq::SetFanMode(fan_mode) => {
                _ = ec.lock().await.write_data(&fan_mode).await?;
                info!("Fan mode set to {fan_mode}");
                stream
                    .send(DaemonResp::WriteResult(WriteResult::Done))
                    .await?;
            }
            DaemonReq::SetBatThreshold(bat_threshold) => {
                if *bat_threshold < 60 || *bat_threshold > 100 {
                    bail!("Unknown daemon request");
                }
                _ = ec.lock().await.write_data(&bat_threshold).await?;
                info!("Battery threshold set to {}", *bat_threshold);
                stream
                    .send(DaemonResp::WriteResult(WriteResult::Done))
                    .await?;
            }
            DaemonReq::ReadValues(mut values) => {
                ec.lock().await.read_data_inner(&mut values).await?;
                debug!("Read data: {values:#?}");
                stream.send(DaemonResp::ReadValues(values)).await?;
            }
        }
    }
}

#[cfg(feature = "self-packed")]
pub fn install_daemon() -> Result<()> {
    use std::{fs::File, io::Write, process::Command};

    let this_exe = std::env::current_exe()?;
    if this_exe.to_string_lossy() != "/usr/local/bin/gigacenter" {
        info!("Installing binary to /usr/local/bin...");
        let res = Command::new("cp")
            .arg(this_exe)
            .arg("/usr/local/bin/gigacenter")
            .spawn()?
            .wait()?;
        if !res.success() {
            bail!("Failed to install binary to /usr/local/bin")
        }
        info!("Binary successfully installed to /usr/local/bin");
    }
    info!("Installing systemd service");
    let mut file = File::create("/etc/systemd/system/gigacenter-daemon.service")?;
    file.write_all(SYSTEMD_SERVICE)?;
    let res = Command::new("systemctl")
        .args(["enable", "gigacenter-daemon.service", "--now"])
        .spawn()?
        .wait()?;
    if !res.success() {
        bail!("Failed to install binary to /usr/local/bin")
    }
    info!("Systemd service successfully installed");

    Ok(())
}

#[cfg(feature = "self-packed")]
pub fn remove_daemon() -> Result<()> {
    use std::process::Command;

    _ = Command::new("rm")
        .args(["-f", "/usr/local/bin/gigacenter"])
        .spawn()?
        .wait()?;
    _ = Command::new("systemctl")
        .args(["disable", "gigacenter-daemon.service", "--now"])
        .spawn()?
        .wait()?;
    _ = Command::new("rm")
        .args(["-f", "/etc/systemd/system/gigacenter-daemon.service"])
        .spawn()?
        .wait()?;

    info!("GigaCenter successfully removed");

    Ok(())
}
