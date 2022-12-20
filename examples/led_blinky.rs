//! Example
//! Simple LED Blinky implementation which makes use of rust-embedded crates
//! and non_preemptive scheduler to blink a couple of leds on a STM32F429I-DISC1 board
#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use cortex_m::{asm, interrupt::free as critical_section, peripheral::syst::SystClkSource};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    gpio::{
        gpiog::{PG13, PG14},
        Output, PushPull,
    },
    pac,
    prelude::*,
};
use panic_halt as _;
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use scheduler::non_preemptive::{
    resources::{Shared, UnShared},
    *,
};
use scheduler_macros::*;
use stm32f4xx_hal as hal;

// Events
const EVENT_TOGGLE_RED_LED: EventMask = 0x00000001;
// Static and interior mutable entities
static GREEN_LED: UnShared<RefCell<Option<PG13<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static RED_LED: UnShared<RefCell<Option<PG14<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static TIME_COUNTER: Shared<Cell<u32>> = Shared::new(Cell::new(0));

// Instantiate scheduler
const SCHEDULER_TASK_COUNT: usize = 3;
#[scheduler_nonpreeptive((SCHEDULER_TASK_COUNT, get_tick))]
struct Scheduler;

// Tick getter needed by the scheduler
fn get_tick() -> u32 {
    critical_section(|cs| TIME_COUNTER.borrow(cs).get())
}

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
    scheduler_set_event!(("red_led_blinky", EVENT_TOGGLE_RED_LED));
}

// BSP initialization
fn bsp_init() {
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
    systick.set_reload(180_000); // 1ms tick
    systick.enable_counter();
    systick.enable_interrupt();

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

    // Create tasks
    let green_led_blinky_task = Task::new(
        "green_led_blinky",     // Task name
        None,                   // Init runnable
        Some(green_led_blinky), // Process runnable
        Some(250),              // Execution cycle
        None,                   // Execution offset
    );

    let red_led_switcher_task = Task::new(
        "red_led_switcher",
        None,
        Some(red_led_switcher),
        Some(1000),
        Some(10),
    );

    let red_led_blinky_task = Task::new(
        "red_led_blinky",
        Some(red_led_on),
        Some(red_led_blinky),
        None,
        None,
    );

    // Add tasks to scheduler
    scheduler_add_task!(green_led_blinky_task);
    scheduler_add_task!(red_led_switcher_task);
    scheduler_add_task!(red_led_blinky_task);

    // Register idle runnable
    scheduler_register_idle_runnable!(asm::nop);

    // Launch scheduler
    scheduler_launch!();

    loop {
        panic!();
    }
}

#[exception]
fn SysTick() {
    critical_section(|cs| {
        let scheduler_counter = TIME_COUNTER.borrow(cs).get();
        TIME_COUNTER.borrow(cs).set(scheduler_counter + 1);
    })
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    log!("{:#?}", ef);
    loop {
        asm::nop();
    }
}
