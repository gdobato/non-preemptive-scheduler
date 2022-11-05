//! Example
//! Simple LED blinky which makes uses of existing rust-embedded crates

#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::{
    asm,
    interrupt::{self, Mutex},
    peripheral::syst::SystClkSource,
};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    gpio::{
        gpiog::{PG13, PG14},
        Output, PushPull,
    },
    pac,
    prelude::*,
};
use hello_rust::scheduler::{Scheduler, Task};
use panic_halt as _;
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use stm32f4xx_hal as hal;

const SCHEDULER_TASK_COUNT: usize = 3;
// Static and interior mutable entities
static LED_GREEN: Mutex<RefCell<Option<PG13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static LED_RED: Mutex<RefCell<Option<PG14<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
// Unsafe static mutable counter, shared between systick isr handler and normal execution level
static mut SCHEDULER_COUNTER: u32 = 0;

fn time_monitor() -> u32 {
    interrupt::free(|_| unsafe { SCHEDULER_COUNTER })
}

fn red_led_blinky_process() {
    interrupt::free(|cs| {
        if let Some(led_red) = LED_RED.borrow(cs).borrow_mut().as_mut() {
            led_red.toggle();
        }
    })
}

fn green_led_blinky_process() {
    interrupt::free(|cs| {
        if let Some(led_green) = LED_GREEN.borrow(cs).borrow_mut().as_mut() {
            led_green.toggle();
        }
    })
}

#[entry]
fn main() -> ! {
    log_init!();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    dp.RCC
        .constrain()
        .cfgr
        .use_hse(8.MHz())
        .sysclk(180.MHz())
        .freeze();

    let mut systick = cp.SYST;
    systick.set_clock_source(SystClkSource::Core);
    systick.set_reload(180_000); // 1ms tick. TODO: Use time based literals
    systick.enable_counter();
    systick.enable_interrupt();

    let gpio_g = dp.GPIOG.split();

    interrupt::free(|cs| {
        LED_GREEN
            .borrow(cs)
            .replace(Some(gpio_g.pg13.into_push_pull_output()));
        LED_RED
            .borrow(cs)
            .replace(Some(gpio_g.pg14.into_push_pull_output()));
    });

    let mut scheduler = Scheduler::<SCHEDULER_TASK_COUNT, &dyn Fn() -> u32>::new(&time_monitor);

    scheduler.add_task(Task::new(
        "green_led_blinky",
        Some(|| log!("green_led_blinky init")),
        Some(green_led_blinky_process),
        250,
        0,
    ));

    scheduler.add_task(Task::new(
        "red_led_blinky",
        Some(|| log!("red_led_blinky init")),
        Some(red_led_blinky_process),
        250,
        250,
    ));

    scheduler.add_task(Task::new(
        "alive_counter",
        None,
        Some(|| {
            static mut ALIVE_COUNTER: u32 = 0;
            log!("Alive counter: {:?}", unsafe { ALIVE_COUNTER });
            unsafe { ALIVE_COUNTER += 1 };
        }),
        1000,
        0,
    ));

    scheduler.register_idle_task(|| asm::nop()); // Bkpt placeholder

    scheduler.launch();

    loop {}
}

#[exception]
fn SysTick() {
    unsafe {
        // Temporary to bring up the stuff
        SCHEDULER_COUNTER += 1;
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    log!("{:#?}", ef);
    loop {
        asm::nop();
    }
}
