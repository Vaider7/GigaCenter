use rkyv::{Archive, Deserialize, Serialize};
use std::{
    fmt::Display,
    io::SeekFrom,
    num::{IntErrorKind, ParseIntError},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt as _, AsyncWriteExt},
};

use anyhow::{Context, Error, Result, bail};
use libc::geteuid;

use crate::{
    deref,
    traits::{ECHandler, ReadEC, WriteEC, WriteResult},
};

/// Register (addr) of embedded controller
pub type Reg = u8;

/// Represent state of bit. (Disabled - bit is 0, enabled - bit is 1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub enum BitState {
    Disabled = 0,
    Enabled = 1,
}

impl Display for BitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// Represent the position of bit in u8
#[derive(Clone, Copy, Debug, Archive, Serialize, Deserialize)]
pub struct BitPos(u8);

impl BitPos {
    /// May panic if bit_num > 7
    pub const fn new(bit_num: u8) -> Self {
        if bit_num > 7 {
            panic!("Bit number must be from 0 to 7");
        }
        Self(bit_num)
    }

    pub fn try_new(bit_num: u8) -> Result<Self, Error> {
        if bit_num > 7 {
            bail!("Bit number must be from 0 to 7")
        }
        Ok(Self(bit_num))
    }
}

deref!(BitPos(u8));

/// Represent bit or byte to be written/read to/from specified register
#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum RWData {
    U1 {
        reg: Reg,
        pos: BitPos,
        state: BitState,
    },
    U8 {
        reg: u8,
        value: u8,
    },
    U16 {
        reg: u8,
        value: u16,
    },
}

pub const WRITE_TIMEOUT_MS: u16 = 2000;
const WRITE_TIMEOUT_PATH: &str = "/tmp/last-write-ec";

/// Main struct to read/write data to/from EC
#[derive(Debug)]
pub struct EmbeddedController {
    ec_file: File,
}

impl EmbeddedController {
    pub async fn new() -> Result<Self> {
        let euid = unsafe { geteuid() };
        if euid != 0 {
            bail!("You must run this program as root")
        }
        _ = Command::new("modprobe")
            .args(["ec_sys", "write_support=1"])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?
            .wait()?;
        let ec_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/sys/kernel/debug/ec/ec0/io")
            .await?;
        Ok(Self { ec_file })
    }

    async fn read_u1(&mut self, reg: Reg, bit_num: BitPos) -> Result<BitState> {
        _ = self.ec_file.seek(SeekFrom::Start(reg as u64)).await?;
        let mut current_value = [0];
        _ = self.ec_file.read_exact(&mut current_value).await?;
        let byte_musk: u8 = 1 << bit_num.0;
        let res = current_value[0] & byte_musk;
        if res == 0 {
            Ok(BitState::Disabled)
        } else {
            Ok(BitState::Enabled)
        }
    }

    async fn read_u8(&mut self, reg: Reg) -> Result<u8> {
        _ = self.ec_file.seek(SeekFrom::Start(reg as u64)).await?;
        let mut current_value = [0];
        _ = self.ec_file.read_exact(&mut current_value).await?;
        Ok(current_value[0])
    }

    pub async fn read_u16(&mut self, reg: Reg) -> Result<u16> {
        _ = self.ec_file.seek(SeekFrom::Start(reg as u64)).await?;
        let mut current_value = [0, 0];
        _ = self.ec_file.read_exact(&mut current_value).await?;
        Ok(u16::from_be_bytes(current_value))
    }

    async fn write_u1(&mut self, reg: Reg, bit_num: BitPos, value: BitState) -> Result<()> {
        let mut current_value = self.read_u8(reg).await?;
        match value {
            BitState::Disabled => {
                current_value &= !(1 << bit_num.0);
            }
            BitState::Enabled => {
                current_value |= 1 << bit_num.0;
            }
        }
        self.write_u8(reg, current_value).await?;
        Ok(())
    }

    async fn write_u8(&mut self, reg: Reg, value: u8) -> Result<()> {
        _ = self.ec_file.seek(SeekFrom::Start(reg as u64)).await?;
        self.ec_file.write_all(&[value]).await?;
        Ok(())
    }

    #[inline]
    pub async fn read_data_inner(&mut self, data: &mut [RWData]) -> Result<()> {
        for op in data {
            match op {
                RWData::U1 { reg, pos, state } => {
                    *state = self.read_u1(*reg, *pos).await?;
                }
                RWData::U8 { reg, value } => {
                    *value = self.read_u8(*reg).await?;
                }
                RWData::U16 { reg, value } => {
                    *value = self.read_u16(*reg).await?;
                }
            }
        }
        Ok(())
    }

    async fn duration_since_last_write_ms(&self) -> Result<u64> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(WRITE_TIMEOUT_PATH)
            .await
            .context("Open timestamp file")?;
        let mut str_timestamp = String::new();
        _ = file.read_to_string(&mut str_timestamp).await?;
        let timestamp_ms = str_timestamp.parse::<u64>().context("Parse timestamp")?;
        let duration = Duration::from_millis(timestamp_ms);
        let now = SystemTime::now();

        let diff = now - duration;
        let elapsed = diff
            .duration_since(UNIX_EPOCH)
            .context("Duration since last write")?;

        Ok(elapsed.as_millis() as u64)
    }

    async fn set_last_write_time(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WRITE_TIMEOUT_PATH)
            .await?;
        let now = SystemTime::now();
        let str_unix = now.duration_since(UNIX_EPOCH)?.as_millis().to_string();
        file.write_all(str_unix.as_bytes()).await?;
        Ok(())
    }
}

impl ECHandler for EmbeddedController {
    async fn write_data(&mut self, data: &impl WriteEC) -> Result<WriteResult> {
        let last_write_time = self.duration_since_last_write_ms().await;
        let timeout = match last_write_time {
            Ok(duration) => {
                if duration as u16 > WRITE_TIMEOUT_MS {
                    Duration::from_millis(0)
                } else {
                    Duration::from_millis((WRITE_TIMEOUT_MS as u64).saturating_sub(duration))
                }
            }
            Err(err) => {
                let root = err.root_cause();
                let timeout = root
                    .downcast_ref::<ParseIntError>()
                    .and_then(|err| {
                        // If IntErrorKind::Empty, then no write was done yet
                        if *err.kind() == IntErrorKind::Empty {
                            return Some(Duration::from_millis(0));
                        }
                        None
                    })
                    .unwrap_or_else(|| Duration::from_millis(WRITE_TIMEOUT_MS as u64));
                timeout
            }
        };
        tokio::time::sleep(timeout).await;
        let ops = data.data_to_write();
        for op in ops {
            match op {
                RWData::U1 { reg, pos, state } => self.write_u1(reg, pos, state).await?,
                RWData::U8 { reg, value } => self.write_u8(reg, value).await?,
                // No need to write u16 data for now
                RWData::U16 { .. } => {}
            }
        }
        if let Err(err) = self.set_last_write_time().await {
            eprintln!("Failed tp set_last_write_time {err:?}");
        };
        Ok(WriteResult::Done)
    }

    async fn read_data<T: ReadEC>(&mut self) -> Result<Vec<RWData>> {
        let mut ops = T::data_to_read();
        self.read_data_inner(&mut ops).await?;
        Ok(ops)
    }
}
