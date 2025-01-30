use std::{io::ErrorKind, marker::PhantomData};

use rkyv::{rancor::Error as RkyvError, Archive, Deserialize, Serialize};
use tokio::net::UnixStream;
use tokio_util::{
    bytes::Buf as _,
    codec::{Decoder, Encoder, Framed},
};

use crate::{bat::BatThreshold, fan_speed::FanMode, traits::WriteResult, RWData};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum DaemonReq {
    SetFanMode(FanMode),
    SetBatThreshold(BatThreshold),
    ReadValues(Vec<RWData>),
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum DaemonResp {
    ReadValues(Vec<RWData>),
    WriteResult(WriteResult),
    Error(String),
}

pub type FramedClient = Framed<UnixStream, DaemonCodec<RoleClient>>;
pub type FramedServer = Framed<UnixStream, DaemonCodec<RoleServer>>;

pub fn bind_transport_client(
    unix_socket: UnixStream,
) -> Framed<UnixStream, DaemonCodec<RoleClient>> {
    Framed::new(
        unix_socket,
        DaemonCodec {
            _phantom: PhantomData,
        },
    )
}

pub fn bind_transport_server(
    unix_socket: UnixStream,
) -> Framed<UnixStream, DaemonCodec<RoleServer>> {
    Framed::new(
        unix_socket,
        DaemonCodec {
            _phantom: PhantomData,
        },
    )
}

#[derive(Debug)]
pub struct RoleClient;
#[derive(Debug)]
pub struct RoleServer;

#[derive(Debug)]
pub struct DaemonCodec<T> {
    _phantom: PhantomData<T>,
}

impl Decoder for DaemonCodec<RoleClient> {
    type Item = DaemonResp;

    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let mut length_bytes = [0; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let response = rkyv::access::<ArchivedDaemonResp, RkyvError>(&data)
            .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, err))?;
        let response = rkyv::deserialize::<_, RkyvError>(response).unwrap();
        Ok(Some(response))
    }
}

impl Encoder<DaemonReq> for DaemonCodec<RoleClient> {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: DaemonReq,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let bytes = rkyv::to_bytes::<RkyvError>(&item).unwrap();
        let len_slice = u32::to_le_bytes(bytes.len() as u32);
        dst.reserve(4 + bytes.len());
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(&bytes);
        Ok(())
    }
}

impl Decoder for DaemonCodec<RoleServer> {
    type Item = DaemonReq;

    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let mut length_bytes = [0; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        if src.len() < 4 + length {
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let response = rkyv::access::<ArchivedDaemonReq, RkyvError>(&data)
            .map_err(|err| std::io::Error::new(ErrorKind::InvalidData, err))?;
        let response = rkyv::deserialize::<_, RkyvError>(response).unwrap();
        Ok(Some(response))
    }
}

impl Encoder<DaemonResp> for DaemonCodec<RoleServer> {
    type Error = std::io::Error;

    fn encode(
        &mut self,
        item: DaemonResp,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let bytes = rkyv::to_bytes::<RkyvError>(&item).unwrap();
        let len_slice = u32::to_le_bytes(bytes.len() as u32);
        dst.reserve(4 + bytes.len());
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(&bytes);
        Ok(())
    }
}
