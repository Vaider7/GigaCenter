use std::{fmt::Display, str::FromStr};

use crate::{
    daemon::codec::DaemonReq,
    impl_read,
    registers::*,
    traits::{ECHandler, InvokeDaemon, ReadEC, WriteEC},
    BitState, RWData, Reg,
};
use anyhow::Result;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Archive, Serialize, Deserialize)]
/// Different fan speeds. Names as they are presented in official Gigabyte Control Center app
/// TODO: custom fan speed (fixed and curved)
pub enum FanMode {
    Normal,
    Eco,
    Power,
    Turbo,
    Unsupported,
}

impl Display for FanMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            FanMode::Normal => "normal",
            FanMode::Eco => "eco",
            FanMode::Power => "power",
            FanMode::Turbo => "turbo",
            FanMode::Unsupported => "unsupported",
        };
        write!(f, "{to_write}")
    }
}

impl FromStr for FanMode {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "normal" => Ok(FanMode::Normal),
            "eco" => Ok(FanMode::Eco),
            "power" => Ok(FanMode::Power),
            "turbo" => Ok(FanMode::Turbo),
            _ => Err("Failed to convert string to FanMode"),
        }
    }
}

impl WriteEC for FanMode {
    fn data_to_write(&self) -> Vec<RWData> {
        match self {
            FanMode::Normal => vec![
                RWData::U1 {
                    reg: ECO_MODE.0,
                    pos: ECO_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: POWER_MODE.0,
                    pos: POWER_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: CUSTOM_MODE.0,
                    pos: CUSTOM_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: FIXED_MODE.0,
                    pos: FIXED_MODE.1,
                    state: BitState::Disabled,
                },
            ],

            FanMode::Eco => vec![
                RWData::U1 {
                    reg: ECO_MODE.0,
                    pos: ECO_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U1 {
                    reg: POWER_MODE.0,
                    pos: POWER_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: CUSTOM_MODE.0,
                    pos: CUSTOM_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: FIXED_MODE.0,
                    pos: FIXED_MODE.1,
                    state: BitState::Disabled,
                },
            ],

            FanMode::Power => vec![
                RWData::U1 {
                    reg: ECO_MODE.0,
                    pos: ECO_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: POWER_MODE.0,
                    pos: POWER_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U1 {
                    reg: CUSTOM_MODE.0,
                    pos: CUSTOM_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: FIXED_MODE.0,
                    pos: FIXED_MODE.1,
                    state: BitState::Disabled,
                },
            ],

            FanMode::Turbo => vec![
                RWData::U1 {
                    reg: ECO_MODE.0,
                    pos: ECO_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: POWER_MODE.0,
                    pos: POWER_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: CUSTOM_MODE.0,
                    pos: CUSTOM_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U1 {
                    reg: FIXED_MODE.0,
                    pos: FIXED_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U8 {
                    reg: FIXED_SPEED_FAN1,
                    value: FIXED_SPEED_MAX_VALUE,
                },
                RWData::U8 {
                    reg: FIXED_SPEED_FAN2,
                    value: FIXED_SPEED_MAX_VALUE,
                },
            ],
            FanMode::Unsupported => vec![],
        }
    }
}

impl ReadEC for FanMode {
    fn data_to_read() -> Vec<RWData> {
        vec![
            RWData::U1 {
                reg: ECO_MODE.0,
                pos: ECO_MODE.1,
                state: BitState::Disabled,
            },
            RWData::U1 {
                reg: POWER_MODE.0,
                pos: POWER_MODE.1,
                state: BitState::Disabled,
            },
            RWData::U1 {
                reg: CUSTOM_MODE.0,
                pos: CUSTOM_MODE.1,
                state: BitState::Disabled,
            },
            RWData::U1 {
                reg: FIXED_MODE.0,
                pos: FIXED_MODE.1,
                state: BitState::Disabled,
            },
        ]
    }
}

impl InvokeDaemon for FanMode {
    fn daemon_action(&self) -> DaemonReq {
        DaemonReq::SetFanMode(*self)
    }
}

impl FanMode {
    pub async fn current_mode(ec: &mut impl ECHandler) -> Result<Self> {
        let read_data = ec.read_data::<Self>().await?;

        let is_mode = |reg_to_check: Reg| {
            read_data
                .iter()
                .find_map(|d| {
                    if let RWData::U1 { reg, pos: _, state } = d {
                        if *reg == reg_to_check && state == &BitState::Enabled {
                            return Some(());
                        }
                    }
                    None
                })
                .is_some()
        };

        if is_mode(ECO_MODE.0) {
            Ok(Self::Eco)
        } else if is_mode(POWER_MODE.0) {
            Ok(Self::Power)
        } else if is_mode(CUSTOM_MODE.0) && is_mode(FIXED_MODE.0) {
            Ok(Self::Turbo)
        } else if !is_mode(ECO_MODE.0)
            && !is_mode(POWER_MODE.0)
            && !is_mode(CUSTOM_MODE.0)
            && !is_mode(FIXED_MODE.0)
        {
            Ok(Self::Normal)
        } else {
            Ok(Self::Unsupported)
        }
    }
}

impl_read! {U16, CpuFanSpeed, RWData::U16 { reg: CPU_FAN_SPEED, value: 0 }}
impl_read! {U16, GpuFanSpeed, RWData::U16 { reg: GPU_FAN_SPEED, value: 0 }}
