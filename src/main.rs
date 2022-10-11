#![no_std]
#![no_main]

use cortex_m::{asm};
use cortex_m_rt::{entry, exception};
use panic_halt as _;
use stm32f4xx_hal as hal;
use hal::{pac, prelude::*};
use rtt_target::{rtt_init_print as log_init, rprintln as log};

#[entry]
fn main() -> ! {
    log_init!();

    log!("Init Target...");

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut systick = cp.SYST;
    let rcc = dp.RCC.constrain();

    let clks = rcc
        .cfgr  // Max values considering HSE source (8 MHz)
        .use_hse(8.MHz())
        .hclk(180.MHz())
        .sysclk(180.MHz())
        .pclk1(45.MHz())
        .pclk2(90.MHz())
        .freeze();

    systick.enable_interrupt();

    let mut delay = systick.delay(&clks);
    
    let gpio_g = dp.GPIOG.split();
    let mut led_red = gpio_g.pg14.into_push_pull_output();
    
    loop {
        #[cfg(debug_assertions)]
        log!("Toggling led...");
        led_red.toggle();
        delay.delay_ms(250_u8);
    }
}

#[exception]
fn SysTick() {
    asm::nop();
}
