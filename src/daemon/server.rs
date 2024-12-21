use std::sync::Arc;

use anyhow::{Context, Result, bail};
use futures::{SinkExt, StreamExt};
use std::{fs, os::unix::fs::PermissionsExt};
use tokio::{net::UnixListener, sync::Mutex};

use crate::{
    EmbeddedController,
    daemon::codec::bind_transport_server,
    traits::{ECHandler, WriteResult},
};

use super::codec::{DaemonReq, DaemonResp, FramedServer};

pub const DAEMON_UDS_PATH: &str = "/tmp/gigabyte-linux";

pub async fn start_daemon() -> Result<()> {
    let ec = Arc::new(Mutex::new(EmbeddedController::new().await?));
    _ = tokio::fs::remove_file(DAEMON_UDS_PATH).await;
    let listener = UnixListener::bind(DAEMON_UDS_PATH).context("Create listener")?;
    let perms = fs::Permissions::from_mode(0o777);

    fs::set_permissions(DAEMON_UDS_PATH, perms)?;
    println!("Daemon ready for incoming connections");
    loop {
        match listener.accept().await.context("Create IPC listener") {
            Ok((stream, _)) => {
                let cloned = ec.clone();
                let transport = bind_transport_server(stream);
                _ = tokio::spawn(async {
                    if let Err(err) = handle_incoming(transport, cloned).await {
                        eprintln!("Error daemon request: {err}");
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
            println!("Connection finished");
            return Ok(());
        };
        let Ok(req) = req else {
            bail!("Unknown daemon request");
        };
        match req {
            DaemonReq::SetFanMode(fan_mode) => {
                _ = ec.lock().await.write_data(&fan_mode).await?;
                stream
                    .send(DaemonResp::WriteResult(WriteResult::Done))
                    .await?;
            }
            DaemonReq::SetBatThreshold(bat_threshold) => {
                if *bat_threshold < 60 || *bat_threshold > 100 {
                    bail!("Unknown daemon request");
                }
                _ = ec.lock().await.write_data(&bat_threshold).await?;
                stream
                    .send(DaemonResp::WriteResult(WriteResult::Done))
                    .await?;
            }
            DaemonReq::ReadValues(mut values) => {
                ec.lock().await.read_data_inner(&mut values).await?;
                stream.send(DaemonResp::ReadValues(values)).await?;
            }
        }
    }
}
