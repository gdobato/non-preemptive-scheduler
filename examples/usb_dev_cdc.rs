//! Example
//! Usb device CDC class which makes uses of existing rust-embedded crates
//! Basic echo functionality, polling approach

#![no_std]
#![no_main]

use core::{cell::RefCell, str::from_utf8};
use cortex_m::{asm, singleton};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use hal::{
    otg_hs::{UsbBus, USB},
    pac::{self},
    prelude::*,
};
use panic_halt as _;
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use stm32f4xx_hal as hal;
use usb_device::prelude::*;

mod app {
    use super::*;
    pub struct UsbDevHandler<F, U> {
        state: UsbDeviceState,
        previous_state: UsbDeviceState,
        on_enumeration_complete_cb: F,
        on_enumeration_lost_cb: U,
    }

    impl<F, U> UsbDevHandler<F, U>
    where
        F: FnMut(),
        U: FnMut(),
    {
        pub fn new(on_enumeration_complete_cb: F, on_enumeration_lost_cb: U) -> Self {
            Self {
                on_enumeration_complete_cb,
                on_enumeration_lost_cb,
                state: UsbDeviceState::Default,
                previous_state: UsbDeviceState::Default,
            }
        }

        pub fn update(&mut self, state: UsbDeviceState) {
            if self.state != state {
                self.previous_state = self.state;
                self.state = state;
                self.on_transition();
            }
        }

        pub fn is_enumerated(&self) -> bool {
            self.state == UsbDeviceState::Configured
        }

        fn on_transition(&mut self) {
            match self.state {
                // Going to configured state means that device gets enumerated
                UsbDeviceState::Configured => {
                    log!("Enumeration completed");
                    (self.on_enumeration_complete_cb)();
                }
                // Coming from configured state means that device lost enumeration
                _ if self.previous_state == UsbDeviceState::Configured => {
                    log!("Enumeration lost");
                    (self.on_enumeration_lost_cb)();
                }
                // Ignore any other transition
                _ => (),
            }
        }
    }
}

#[entry]
fn main() -> ! {
    log_init!();
    log!("Init Target...");

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

    // Get Systick as delay source
    let systick = cp.SYST;
    let mut delay = systick.delay(&clks);

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
    const USB_BUS_BUFFER_SIZE: usize = 512;
    let usb_bus_buffer: &'static mut [u32; USB_BUS_BUFFER_SIZE] =
        singleton!(USB_BUFFER: [u32; USB_BUS_BUFFER_SIZE] = [0u32; USB_BUS_BUFFER_SIZE]).unwrap();
    let usb_bus = UsbBus::new(usb, usb_bus_buffer);
    let mut usb_serial_port = usbd_serial::SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1234, 0x1234))
        .manufacturer("Hello rust")
        .product("Usb device CDC example")
        .serial_number("01-23456")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    // Initialize app buffer to process data
    const USB_APP_BUFFER_SIZE: usize = 64;
    let usb_app_buffer: &'static mut [u8; USB_APP_BUFFER_SIZE] =
        singleton!(USB_APP_BUFFER: [u8; USB_APP_BUFFER_SIZE] = [0u8; USB_APP_BUFFER_SIZE]).unwrap();

    // Initialize usb device handler
    let led_enumeration = RefCell::new(dp.GPIOG.split().pg13.into_push_pull_output()); // TODO: Apply better solution than interior mutability
    let mut usb_app_dev_handler = app::UsbDevHandler::new(
        || led_enumeration.borrow_mut().set_high(), // Switch LED on upon enumeration completion
        || led_enumeration.borrow_mut().set_low(),  // Switch LED off upon enumeration lost
    );

    // Super loop for polling usb events
    loop {
        usb_app_dev_handler.update(usb_dev.state());

        if usb_dev.poll(&mut [&mut usb_serial_port]) && usb_app_dev_handler.is_enumerated() {
            // Read from reception fifo. TODO: Needs additional handling
            match usb_serial_port.read(&mut usb_app_buffer[..]) {
                Ok(cnt) if cnt > 0 => {
                    log!(
                        "Received {} bytes: {}",
                        cnt,
                        from_utf8(&usb_app_buffer[..cnt]).unwrap_or("not valid")
                    );
                    // Send back received data
                    match usb_serial_port.write(&usb_app_buffer[..cnt]) {
                        Ok(_) => (),
                        Err(err) => log!("Error in transmission: {:?}", err),
                    }
                }
                _ => (),
            }
        }
        delay.delay_ms(5u8);
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    log!("{:#?}", ef);
    loop {
        asm::nop();
    }
}
