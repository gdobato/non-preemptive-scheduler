#[cfg(feature = "arm-cm")]
mod arm_cm;
#[cfg(feature = "arm-cm")]
pub type SysTick = arm_cm::SysTick;
#[cfg(feature = "arm-cm")]
pub use cortex_m::interrupt::free as critical_section;
#[cfg(feature = "arm-cm")]
pub use cortex_m::interrupt::Mutex;
#[cfg(any(feature = "arm-cm", feature = "risc-v"))]
pub use rtt_target::rprintln as log;

use core::panic::PanicInfo;
#[cfg(debug_assertions)]
#[cfg(feature = "arm-cm")]
use cortex_m::asm::bkpt;

#[inline(never)]
#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(debug_assertions)]
    log!("{}", info);
    loop {
        #[cfg(debug_assertions)]
        bkpt();
    }
}
