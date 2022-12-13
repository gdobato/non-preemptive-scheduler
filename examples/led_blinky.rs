//! Example
//! Simple LED Blinky implementation which makes use of rust-embedded crates
//! and non_preemptive scheduler to blink a couple of leds on a STM32F429I-DISC1 board
#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use cortex_m::{
    asm,
    interrupt::{free as critical_section, Mutex},
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
use panic_halt as _;
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use scheduler::non_preemptive::{EventMask, Scheduler, Task};
use stm32f4xx_hal as hal;

const SCHEDULER_TASK_COUNT: usize = 3;
const EVENT_TOGGLE_RED_LED: EventMask = 0x00000001;
// Static and interior mutable entities
static GREEN_LED: Mutex<RefCell<Option<PG13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static RED_LED: Mutex<RefCell<Option<PG14<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static TIME_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
// Static mutable entities
static mut RED_LED_BLINKY_TASK: Option<Task> = None;

// Tick getter needed by the scheduler
fn get_tick() -> u32 {
    critical_section(|cs| TIME_COUNTER.borrow(cs).get())
}

// Functions which are bound to task runnables
fn green_led_blinky(_: EventMask) {
    critical_section(|cs| {
        if let Some(led_green) = GREEN_LED.borrow(cs).borrow_mut().as_mut() {
            led_green.toggle();
        }
    })
}

fn red_led_on() {
    critical_section(|cs| {
        if let Some(led_red) = RED_LED.borrow(cs).borrow_mut().as_mut() {
            led_red.set_high();
        }
    });
}

fn red_led_blinky(event_mask: EventMask) {
    if event_mask & EVENT_TOGGLE_RED_LED != 0 {
        critical_section(|cs| {
            if let Some(led_red) = RED_LED.borrow(cs).borrow_mut().as_mut() {
                led_red.toggle();
            }
        });
    }
}

fn red_led_switcher(_: EventMask) {
    unsafe {
        if let Some(red_led_blinky_task) = RED_LED_BLINKY_TASK.as_mut() {
            red_led_blinky_task.set_event(EVENT_TOGGLE_RED_LED);
        }
    }
}

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

    let gpio_g = dp.GPIOG.split();

    critical_section(|cs| {
        GREEN_LED
            .borrow(cs)
            .replace(Some(gpio_g.pg13.into_push_pull_output()));
        RED_LED
            .borrow(cs)
            .replace(Some(gpio_g.pg14.into_push_pull_output()));
    });
}

#[entry]
fn main() -> ! {
    log_init!();

    bsp_init();

    // Create scheduler, passing a tick getter
    let mut scheduler = Scheduler::<SCHEDULER_TASK_COUNT>::new(get_tick);

    // Create tasks
    let mut green_led_blinky_task = Task::new(
        "green_led_blinky",     // Task name
        None,                   // Init runnable
        Some(green_led_blinky), // Process runnable
        Some(250),              // Execution cycle
        None,                   // Execution offset
    );

    let mut red_led_switcher_task = Task::new(
        "red_led_switcher",
        None,
        Some(red_led_switcher),
        Some(1000),
        Some(10),
    );

    // Use of unsafe blocks to handle static mutable tasks due to scheduler limitations
    unsafe {
        RED_LED_BLINKY_TASK.replace(Task::new(
            "red_led_blinky",
            Some(red_led_on),
            Some(red_led_blinky),
            None,
            None,
        ));
    }

    // Add tasks to scheduler
    scheduler.add_task(&mut green_led_blinky_task);

    scheduler.add_task(&mut red_led_switcher_task);

    unsafe {
        if let Some(red_led_blinky_task) = RED_LED_BLINKY_TASK.as_mut() {
            scheduler.add_task(red_led_blinky_task);
        }
    }

    // Register idle runnable
    scheduler.register_idle_runnable(asm::nop);

    // Launch scheduler
    scheduler.launch();

    loop {}
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
