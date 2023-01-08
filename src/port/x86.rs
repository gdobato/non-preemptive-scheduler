//! Abstractions for x86

use std::marker::PhantomData;

pub struct SysTick {
    _core_freq: u32,
}

impl SysTick {
    pub fn bind_with_core_and_take(core_freq: u32) -> Option<SysTick> {
        static mut TAKEN: bool = false;
        if unsafe { !TAKEN } {
            unsafe {
                TAKEN = true;
            }
            Some(SysTick {
                _core_freq: core_freq,
            })
        } else {
            None
        }
    }

    pub fn launch(&self) {
        todo!();
    }

    pub fn get(&self) -> u32 {
        todo!();
    }
}

pub struct Mutex<T> {
    _marker: PhantomData<T>,
}

pub struct CriticalSection;

pub fn critical_section<F, R>(f: F) -> R
where
    F: FnOnce(CriticalSection) -> R,
{
    f(CriticalSection {})
}

#[cfg(feature = "panic")]
pub mod panic {
    use println as log;
    #[inline(never)]
    #[panic_handler]
    #[allow(unused_variables)]
    fn panic(info: &PanicInfo) -> ! {
        #[cfg(debug_assertions)]
        log!("{}", info);
        loop {}
    }
}
