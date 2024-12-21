use crate::{
    RWData,
    common::Handler,
    daemon::{client::DaemonClient, codec::DaemonReq},
    ec::EmbeddedController,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use rkyv::{Archive, Deserialize, Serialize};

pub trait WriteEC: Sync + Send {
    /// A series of data to be written to EC
    fn data_to_write(&self) -> Vec<RWData>;
}

pub trait ReadEC {
    /// A series of data to be read from EC
    fn data_to_read() -> Vec<RWData>;
}

pub trait InvokeDaemon {
    /// Action needed to be done by daemon instance
    fn daemon_action(&self) -> DaemonReq;
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum WriteResult {
    Done,
    Busy,
}

/// Trait to manipulate EC data
#[enum_dispatch]
#[expect(async_fn_in_trait)]
pub trait ECHandler: Sized {
    async fn read_data<T: ReadEC>(&mut self) -> Result<Vec<RWData>>;
    async fn write_data(&mut self, data: &(impl WriteEC + InvokeDaemon)) -> Result<WriteResult>;
}
