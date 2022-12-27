pub use cortex_m::{
    interrupt::{free as critical_section, Mutex},
    peripheral,
};
pub use rtt_target::rprintln as log;

use cortex_m_rt::exception;
use volatile_register::RW;

static mut TICK: u32 = 0;

pub struct SysTick;

impl SysTick {
    const SYST_CSR: *mut RW<u32> = 0xE000E010 as *mut _;
    const SYST_RVR: *mut RW<u32> = 0xE000E014 as *mut _;
    const SYST_CSR_COUNTER_ENABLE: u32 = 1 << 0;
    const SYST_CSR_TICK_INT_ENABLE: u32 = 1 << 1;
    const SYST_CSR_TICK_PROCESSOR_AS_CLCK_SOURCE: u32 = 1 << 2;

    pub fn take() -> Option<SysTick> {
        static mut TAKEN: bool = false;
        critical_section(|_| {
            if unsafe { !TAKEN } {
                unsafe {
                    TAKEN = true;
                }
                Some(SysTick)
            } else {
                None
            }
        })
    }

    pub fn launch(&self, core_freq: u32) {
        unsafe {
            (*Self::SYST_CSR).modify(|v| v & !Self::SYST_CSR_COUNTER_ENABLE);
            (*Self::SYST_RVR).write((core_freq / 1000) - 1); // 1ms
        }
        unsafe {
            (*Self::SYST_CSR).modify(|v| {
                v | Self::SYST_CSR_COUNTER_ENABLE
                    | Self::SYST_CSR_TICK_INT_ENABLE
                    | Self::SYST_CSR_TICK_PROCESSOR_AS_CLCK_SOURCE
            });
        }
    }

    pub fn get(&self) -> u32 {
        critical_section(|_| unsafe { TICK })
    }
}

#[exception]
fn SysTick() {
    unsafe {
        TICK += 1;
    }
}
