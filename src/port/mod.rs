#[cfg(feature = "armvx-m")]
mod armvx_m;
#[cfg(feature = "armvx-m")]
pub use cortex_m::interrupt::free as critical_section;
#[cfg(feature = "armvx-m")]
pub type Mutex<T> = cortex_m::interrupt::Mutex<T>;
#[cfg(feature = "armvx-m")]
pub type SysTick = armvx_m::SysTick;
#[cfg(feature = "armvx-m")]
pub use rtt_target::rprintln as log;
#[cfg(feature = "x86")]
mod x86;
#[cfg(feature = "x86")]
pub use x86::critical_section;
#[cfg(feature = "x86")]
pub type Mutex<T> = x86::Mutex<T>;
#[cfg(feature = "x86")]
pub type SysTick = x86::SysTick;
#[cfg(feature = "x86")]
pub(crate) use println as log;

#[cfg(feature = "panic")]
mod panic {
    #[cfg(feature = "armvx-m")]
    pub use super::armvx_m::panic;
    #[cfg(feature = "x86")]
    pub use super::x86::panic;
}
