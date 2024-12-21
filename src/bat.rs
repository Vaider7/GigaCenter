use crate::{
    BitState, RWData,
    daemon::codec::DaemonReq,
    impl_read,
    registers::{BAT_THRESHOLD_CUSTOM_MODE, CURRENT_BAT_THRESHOLD},
    traits::{InvokeDaemon, WriteEC},
};

impl_read! {U8, BatThreshold, RWData::U8 { reg: CURRENT_BAT_THRESHOLD, value: 0 } }

impl BatThreshold {
    /// Use [`EmbeddedController.write_data()`] to actually set value in EC
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - provided value is not between 60 and 100
    pub fn new(value: u8) -> Self {
        assert!((60..=100).contains(&value));
        Self(value)
    }
}

impl WriteEC for BatThreshold {
    fn data_to_write(&self) -> Vec<RWData> {
        // Max value, turn off custom mode
        if self.0 == 0x64 {
            vec![
                RWData::U1 {
                    reg: BAT_THRESHOLD_CUSTOM_MODE.0,
                    pos: BAT_THRESHOLD_CUSTOM_MODE.1,
                    state: BitState::Disabled,
                },
                RWData::U8 {
                    reg: CURRENT_BAT_THRESHOLD,
                    value: self.0,
                },
            ]
        } else {
            vec![
                RWData::U1 {
                    reg: BAT_THRESHOLD_CUSTOM_MODE.0,
                    pos: BAT_THRESHOLD_CUSTOM_MODE.1,
                    state: BitState::Enabled,
                },
                RWData::U8 {
                    reg: CURRENT_BAT_THRESHOLD,
                    value: self.0,
                },
            ]
        }
    }
}

impl InvokeDaemon for BatThreshold {
    fn daemon_action(&self) -> DaemonReq {
        DaemonReq::SetBatThreshold(*self)
    }
}
