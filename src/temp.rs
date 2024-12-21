use crate::registers::{CPU_TEMP, GPU_TEMP};
use crate::{RWData, impl_read};

impl_read! {U8, CpuTemp, RWData::U8 {reg: CPU_TEMP, value: 0}}
impl_read! {U8, GpuTemp, RWData::U8 {reg: GPU_TEMP, value: 0}}
