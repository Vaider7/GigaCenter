use crate::{
    daemon::codec::DaemonReq,
    registers::{BAT_THRESHOLD_CUSTOM_MODE, BAT_THRESHOLD_TOGGLE_CUSTOM, CURRENT_BAT_THRESHOLD},
    traits::{ECHandler, InvokeDaemon, ReadEC, WriteEC},
    BitState, RWData,
};
use anyhow::Result;
use std::ops::Deref;

#[derive(Debug, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Eq, PartialEq)]
pub struct BatThreshold {
    value: u8,
    custom_mode_enabled: bool,
}

impl BatThreshold {
    /// Use [`EmbeddedController.write_data()`] to actually set value in EC
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - provided value is not between 60 and 100
    pub fn new(value: u8) -> Self {
        assert!((60..=100).contains(&value));
        Self {
            value,
            custom_mode_enabled: false,
        }
    }

    pub async fn current_state(ec: &mut impl ECHandler) -> Result<Self> {
        let read_data = ec.read_data::<Self>().await?;

        let Some(RWData::U1 {
            reg: _,
            pos: _,
            state,
        }) = read_data.first()
        else {
            unreachable!("Check ReadEC impl");
        };

        let Some(RWData::U8 { reg: _, value }) = read_data.get(1) else {
            unreachable!("Check ReadEC impl");
        };
        Ok(Self {
            value: *value,
            custom_mode_enabled: BitState::Enabled == *state,
        })
    }

    pub fn actual_value(&self) -> u8 {
        if !self.custom_mode_enabled {
            100
        } else {
            self.value
        }
    }
}

impl ReadEC for BatThreshold {
    fn data_to_read() -> Vec<RWData> {
        vec![
            RWData::U1 {
                reg: BAT_THRESHOLD_CUSTOM_MODE.0,
                pos: BAT_THRESHOLD_CUSTOM_MODE.1,
                state: BitState::Disabled,
            },
            RWData::U8 {
                reg: CURRENT_BAT_THRESHOLD,
                value: 0,
            },
        ]
    }
}

impl WriteEC for BatThreshold {
    fn data_to_write(&self) -> Vec<RWData> {
        // Max value, turn off custom mode
        if self.value == 0x64 {
            vec![
                RWData::U1 {
                    reg: BAT_THRESHOLD_TOGGLE_CUSTOM.0,
                    pos: BAT_THRESHOLD_TOGGLE_CUSTOM.1,
                    state: BitState::Disabled,
                },
                RWData::U1 {
                    reg: BAT_THRESHOLD_CUSTOM_MODE.0,
                    pos: BAT_THRESHOLD_CUSTOM_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U8 {
                    reg: CURRENT_BAT_THRESHOLD,
                    value: self.value,
                },
            ]
        } else {
            vec![
                RWData::U1 {
                    reg: BAT_THRESHOLD_TOGGLE_CUSTOM.0,
                    pos: BAT_THRESHOLD_TOGGLE_CUSTOM.1,
                    state: BitState::Enabled,
                },
                RWData::U1 {
                    reg: BAT_THRESHOLD_CUSTOM_MODE.0,
                    pos: BAT_THRESHOLD_CUSTOM_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U8 {
                    reg: CURRENT_BAT_THRESHOLD,
                    value: self.value,
                },
            ]
        }
    }
}

impl Deref for BatThreshold {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        if !self.custom_mode_enabled {
            &100
        } else {
            &self.value
        }
    }
}

impl InvokeDaemon for BatThreshold {
    fn daemon_action(&self) -> DaemonReq {
        DaemonReq::SetBatThreshold(*self)
    }
}
