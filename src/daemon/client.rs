use anyhow::{Context, Result, bail};
use futures::{SinkExt as _, StreamExt};
use tokio::net::UnixStream;

use crate::{
    RWData,
    traits::{ECHandler, ReadEC, WriteResult},
};

use super::{
    codec::{DaemonReq, DaemonResp, FramedClient, bind_transport_client},
    server::DAEMON_UDS_PATH,
};

#[derive(Debug)]
pub struct DaemonClient {
    stream: FramedClient,
}

impl DaemonClient {
    pub async fn connect() -> Result<Self> {
        let stream = UnixStream::connect(DAEMON_UDS_PATH)
            .await
            .context("Connect to daemon")?;
        let stream = bind_transport_client(stream);
        Ok(Self { stream })
    }
}

impl ECHandler for DaemonClient {
    async fn read_data<T: ReadEC>(&mut self) -> Result<Vec<RWData>> {
        let data_to_read = T::data_to_read();
        self.stream
            .send(DaemonReq::ReadValues(data_to_read))
            .await?;
        let Some(Ok(DaemonResp::ReadValues(data))) = self.stream.next().await else {
            bail!("Unknown daemon reply")
        };
        Ok(data)
    }

    async fn write_data(
        &mut self,
        data: &(impl crate::traits::WriteEC + crate::traits::InvokeDaemon),
    ) -> Result<WriteResult> {
        let req = data.daemon_action();
        self.stream.send(req).await?;
        let Some(Ok(DaemonResp::WriteResult(res))) = self.stream.next().await else {
            bail!("Unknown daemon reply")
        };
        Ok(res)
    }
}
