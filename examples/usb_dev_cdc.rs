//! Example
//! Usb device CDC class which makes uses of existing rust-embedded crates
//! and non_preemptive scheduler running on a STM32F429I-DISC1 board
#![no_std]
#![no_main]

use core::{
    cell::{Cell, RefCell},
    str::from_utf8,
};
use cortex_m::{
    asm,
    interrupt::{free as critical_section, Mutex},
    peripheral::syst::SystClkSource,
    singleton,
};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    gpio::{gpiog::PG13, Output, PushPull, PG14},
    otg_hs::{UsbBus, USB},
    pac::{self},
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
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

// Events
const EVENT_USB_ENUMERATION: EventMask = 0x00000001;
const EVENT_USB_ENUMERATION_LOST: EventMask = 0x00000002;
// Static and interior mutable entities
static GREEN_LED: UnShared<RefCell<Option<PG13<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static RED_LED: UnShared<RefCell<Option<PG14<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static USB_SERIAL_PORT: UnShared<RefCell<Option<SerialPort<UsbBus<USB>>>>> =
    UnShared::new(RefCell::new(None));
static USB_DEV: UnShared<RefCell<Option<UsbDevice<UsbBus<USB>>>>> =
    UnShared::new(RefCell::new(None));
static TIME_COUNTER: Shared<Cell<u32>> = Mutex::new(Cell::new(0));
// Static mutable entities
const USB_BUS_BUFFER_SIZE: usize = 512;
static mut USB_BUS_BUFFER: [u32; USB_BUS_BUFFER_SIZE] = [0u32; USB_BUS_BUFFER_SIZE];
const USB_APP_BUFFER_SIZE: usize = 64;
static mut USB_APP_BUFFER: [u8; USB_APP_BUFFER_SIZE] = [0u8; USB_APP_BUFFER_SIZE];

// Instantiate scheduler
const SCHEDULER_TASK_COUNT: usize = 2;
#[scheduler_nonpreeptive((SCHEDULER_TASK_COUNT, get_tick))]
struct Scheduler;

// Tick getter needed by the scheduler
fn get_tick() -> u32 {
    critical_section(|cs| TIME_COUNTER.borrow(cs).get())
}

// Functions which are bound to task runnables
fn usb_process(_: EventMask) {
    if let (Some(usb_dev), Some(usb_serial_port)) = (
        USB_DEV.borrow().borrow_mut().as_mut(),
        USB_SERIAL_PORT.borrow().borrow_mut().as_mut(),
    ) {
        // Previous state before polling
        let previous_state = usb_dev.state();
        if usb_dev.poll(&mut [usb_serial_port]) {
            // Read from reception fifo.
            match usb_serial_port.read(unsafe { &mut USB_APP_BUFFER[..] }) {
                Ok(cnt) if cnt > 0 => {
                    log!(
                        "Received {} bytes: {}",
                        cnt,
                        from_utf8(unsafe { &USB_APP_BUFFER[..cnt] }).unwrap_or("not valid")
                    );
                    // Send back received data
                    match usb_serial_port.write(unsafe { &USB_APP_BUFFER[..cnt] }) {
                        Ok(_) => (),
                        Err(err) => log!("Error in transmission: {:?}", err),
                    }
                }
                _ => (),
            }
        }

        // Current state after polling
        match usb_dev.state() {
            // Transition to enumeration
            UsbDeviceState::Configured if previous_state == UsbDeviceState::Addressed => {
                scheduler_set_event!(("led_handler", EVENT_USB_ENUMERATION));
            }
            // Already enumerated
            UsbDeviceState::Configured => {}
            // Enumeration lost
            _ if previous_state == UsbDeviceState::Configured => {
                scheduler_set_event!(("led_handler", EVENT_USB_ENUMERATION_LOST));
            }
            _ => (),
        }
    }
}

fn led_handler(event_mask: EventMask) {
    // Execution due to an event
    if event_mask != 0 {
        match event_mask & (EVENT_USB_ENUMERATION | EVENT_USB_ENUMERATION_LOST) {
            EVENT_USB_ENUMERATION => {
                if let Some(green_led) = GREEN_LED.borrow().borrow_mut().as_mut() {
                    log!("Enumeration completed");
                    green_led.set_high();
                }
            }

            EVENT_USB_ENUMERATION_LOST => {
                if let Some(green_led) = GREEN_LED.borrow().borrow_mut().as_mut() {
                    log!("Enumeration lost");
                    green_led.set_low();
                }
            }
            _ => (),
        }
    // Cyclic execution
    } else if let Some(red_led) = RED_LED.borrow().borrow_mut().as_mut() {
        red_led.toggle();
    }
}

// BSP initialization
fn bsp_init() {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clks = rcc
        .cfgr
        .use_hse(8.MHz())
        .require_pll48clk()
        .hclk(180.MHz())
        .sysclk(180.MHz())
        .pclk1(45.MHz())
        .pclk2(90.MHz())
        .freeze();

    // Throw panic if USB source clock is not correctly set
    if !clks.is_pll48clk_valid() {
        panic!("USB clock invalid!");
    }

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

    // Initialize USB peripheral
    let gpio_b = dp.GPIOB.split();
    let usb = USB {
        usb_global: dp.OTG_HS_GLOBAL,
        usb_device: dp.OTG_HS_DEVICE,
        usb_pwrclk: dp.OTG_HS_PWRCLK,
        pin_dm: gpio_b.pb14.into_alternate(),
        pin_dp: gpio_b.pb15.into_alternate(),
        hclk: clks.hclk(),
    };

    // Initialize USB stack
    let usb_bus: &'static UsbBusAllocator<UsbBus<USB>> = singleton!(
        USB_BUS: UsbBusAllocator<UsbBus<USB>> = UsbBus::new(usb, unsafe { &mut USB_BUS_BUFFER })
    )
    .unwrap();

    USB_SERIAL_PORT
        .borrow()
        .replace(Some(usbd_serial::SerialPort::new(usb_bus)));
    USB_DEV.borrow().replace(Some(
        UsbDeviceBuilder::new(usb_bus, UsbVidPid(0xABCD, 0xABCD))
            .manufacturer("Hello rust")
            .product("Usb device CDC example")
            .serial_number("01-23456")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build(),
    ));
}

#[entry]
fn main() -> ! {
    log_init!();

    bsp_init();

    // Create tasks
    let usb_echo_task = Task::new(
        "usb_echo",        // Task name
        None,              // Init runnable
        Some(usb_process), // Process runnable
        Some(10),          // Execution cycle
        None,              // Execution offset
    );

    let led_handler_task = Task::new("led_handler", None, Some(led_handler), Some(500), None);

    // Add tasks to scheduler
    scheduler_add_task!(usb_echo_task);
    scheduler_add_task!(led_handler_task);

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
