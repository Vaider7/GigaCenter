use crate::{BitPos, Reg};

// Fan speeds.
pub const ECO_MODE: (Reg, BitPos) = (0x08, BitPos::new(6));
pub const POWER_MODE: (Reg, BitPos) = (0x0C, BitPos::new(4));
pub const CUSTOM_MODE: (Reg, BitPos) = (0x0D, BitPos::new(7));
pub const FIXED_MODE: (Reg, BitPos) = (0x06, BitPos::new(4));

pub const FIXED_SPEED_FAN1: Reg = 0xB0;
pub const FIXED_SPEED_FAN2: Reg = 0xB1;

#[expect(dead_code, reason = "Maybe used in the future")]
pub const FIXED_SPEED_MIN_VALUE: u8 = 0x39;
pub const FIXED_SPEED_MAX_VALUE: u8 = 0xE5;

pub const CPU_FAN_SPEED: Reg = 0xFC;
pub const GPU_FAN_SPEED: Reg = 0xFE;

pub const CPU_TEMP: Reg = 0x60;
pub const GPU_TEMP: Reg = 0x61;

pub const BAT_THRESHOLD_CUSTOM_MODE: (Reg, BitPos) = (0x0F, BitPos::new(2));
pub const BAT_THRESHOLD_TOGGLE_CUSTOM: (Reg, BitPos) = (0xC6, BitPos::new(0));
pub const CURRENT_BAT_THRESHOLD: Reg = 0xA9;
