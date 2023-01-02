#[cfg(feature = "armvx-m")]
pub mod armvx_m;
#[cfg(feature = "armvx-m")]
pub use cortex_m::interrupt::free as critical_section;
#[cfg(feature = "armvx-m")]
pub type Mutex<T> = cortex_m::interrupt::Mutex<T>;
#[cfg(feature = "armvx-m")]
pub type SysTick = armvx_m::SysTick;
#[cfg(feature = "armvx-m")]
pub use rtt_target::rprintln as log;
#[cfg(feature = "x86-64")]
mod x86_64;
#[cfg(feature = "x86-64")]
pub use x86_64::critical_section;
#[cfg(feature = "x86-64")]
pub type Mutex<T> = x86_64::Mutex<T>;
#[cfg(feature = "x86-64")]
pub type SysTick = x86_64::SysTick;
#[cfg(feature = "x86-64")]
pub(crate) use println as log;
