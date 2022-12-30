//! Example
//! Simple LED Blinky implementation which makes use of rust-embedded crates
//! and non_preemptive scheduler to blink a couple of leds on a STM32F429I-DISC1 board

#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    gpio::{
        gpiog::{PG13, PG14},
        Output, PushPull,
    },
    pac,
    prelude::*,
};
use non_preemptive_scheduler::{resources::UnShared, EventMask, Scheduler, Task};
use non_preemptive_scheduler_macros as scheduler;
use panic_halt as _;
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use stm32f4xx_hal as hal;

// Events
const EVENT_TOGGLE_RED_LED: EventMask = 0x00000001;
// Static and interior mutable entities
static GREEN_LED: UnShared<RefCell<Option<PG13<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static RED_LED: UnShared<RefCell<Option<PG14<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));

// Create scheduler
#[scheduler::new(task_count = 3, core_freq = 180_000_000)]
struct NonPreemptiveScheduler;

// Functions which are bound to task runnables
fn green_led_blinky(_: EventMask) {
    if let Some(led_green) = GREEN_LED.borrow().borrow_mut().as_mut() {
        led_green.toggle();
    }
}

fn red_led_on() {
    if let Some(led_red) = RED_LED.borrow().borrow_mut().as_mut() {
        led_red.set_high();
    }
}

fn red_led_blinky(event_mask: EventMask) {
    if event_mask & EVENT_TOGGLE_RED_LED != 0 {
        if let Some(led_red) = RED_LED.borrow().borrow_mut().as_mut() {
            led_red.toggle();
        }
    }
}

fn red_led_switcher(_: EventMask) {
    // Set event on red_led_blinky task
    scheduler::set_task_event!("red_led_blinky", EVENT_TOGGLE_RED_LED);
}

// BSP initialization
fn bsp_init() {
    let dp = pac::Peripherals::take().unwrap();

    dp.RCC
        .constrain()
        .cfgr
        .use_hse(8.MHz())
        .sysclk(180.MHz())
        .freeze();

    // Initialize LEDs
    let gpio_g = dp.GPIOG.split();
    GREEN_LED
        .borrow()
        .replace(Some(gpio_g.pg13.into_push_pull_output()));
    RED_LED
        .borrow()
        .replace(Some(gpio_g.pg14.into_push_pull_output()));
}

#[entry]
fn main() -> ! {
    log_init!();

    bsp_init();

    // Create and add tasks
    scheduler::add_task!(
        "green_led_blinky",     // Task name
        None,                   // Init runnable
        Some(green_led_blinky), // Process runnable
        Some(1_000),            // Execution cycle
        Some(3)                 // Execution offset
    );

    scheduler::add_task!(
        "red_led_switcher",
        None,
        Some(red_led_switcher),
        Some(1_000),
        Some(5)
    );

    scheduler::add_task!(
        "red_led_blinky",
        Some(red_led_on),
        Some(red_led_blinky),
        None,
        None
    );

    // Register idle runnable (optional)
    scheduler::register_idle_runnable!(asm::nop);

    // Launch scheduler
    scheduler::launch!();

    loop {
        panic!("Not expected execution");
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    log!("{:#?}", ef);
    loop {
        asm::nop();
    }
}
