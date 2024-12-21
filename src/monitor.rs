use std::fmt::Display;

use crate::{
    bat::BatThreshold,
    fan_speed::{CpuFanSpeed, FanMode, GpuFanSpeed},
    temp::{CpuTemp, GpuTemp},
    traits::ECHandler,
};
use anyhow::Result;

/// Type describing current machine state
#[derive(Debug, Clone, Copy)]
pub struct Monitor {
    pub fan_mode: FanMode,
    pub cpu_fan_speed: CpuFanSpeed,
    pub gpu_fan_speed: GpuFanSpeed,
    pub cpu_temp: CpuTemp,
    pub gpu_temp: GpuTemp,
    pub bat_threshold: BatThreshold,
}

impl Monitor {
    pub async fn try_new(ec: &mut impl ECHandler) -> Result<Self> {
        Ok(Self {
            fan_mode: FanMode::current_mode(ec).await?,
            cpu_temp: CpuTemp::current_state(ec).await?,
            gpu_temp: GpuTemp::current_state(ec).await?,
            cpu_fan_speed: CpuFanSpeed::current_state(ec).await?,
            gpu_fan_speed: GpuFanSpeed::current_state(ec).await?,
            bat_threshold: BatThreshold::current_state(ec).await?,
        })
    }
}

impl Display for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r"Current machine state:
- Fan mode: {}
- Battery threshold: {}
- GPU Temp: {}
- CPU Temp: {}
- GPU fan speed: {}
- CPU fan speed: {}
",
            self.fan_mode,
            *self.bat_threshold,
            *self.gpu_temp,
            *self.cpu_temp,
            *self.gpu_fan_speed,
            *self.cpu_fan_speed
        )
    }
}
