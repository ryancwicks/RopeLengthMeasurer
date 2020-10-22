//! Blinks an LED
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use cortex_m::asm::delay;

use crate::hal::{prelude::*, stm32};

mod display;

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
        let _hal_delay = hal::delay::Delay::new(cp.SYST, clocks);

        // Set up the LED.
        let gpioc = dp.GPIOC.split();
        let mut led = gpioc.pc13.into_push_pull_output();

        //Set up the display pins
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let d0 = gpioa.pa0.into_push_pull_output().downgrade();
        let d1 = gpioa.pa1.into_push_pull_output().downgrade();
        let d2 = gpioa.pa2.into_push_pull_output().downgrade();
        let d3 = gpioa.pa3.into_push_pull_output().downgrade();
        let d4 = gpioa.pa4.into_push_pull_output().downgrade();
        let d5 = gpioa.pa5.into_push_pull_output().downgrade();
        let d6 = gpioa.pa6.into_push_pull_output().downgrade();
        let d7 = gpioa.pa7.into_push_pull_output().downgrade();
        let enable_pin = gpiob.pb2.into_push_pull_output().downgrade();
        let read_write_pin = gpiob.pb1.into_push_pull_output().downgrade();
        let register_select_pin = gpiob.pb0.into_push_pull_output().downgrade();

        let data_bus: [stm32f4xx_hal::gpio::gpioa::PA<stm32f4xx_hal::gpio::Output<stm32f4xx_hal::gpio::PushPull>>; 8] = [d0, d1, d2, d3, d4, d5, d6, d7];   

        let mut display = display::Display::new(data_bus, enable_pin, read_write_pin, register_select_pin, clocks.hclk() );
        display.initialize_display();

        display.set_cursor_position(0, 0).unwrap();
        display.write_str("Hello, World!");
        display.set_cursor_position(1, 1).unwrap();
        display.write_str("newline");
        

        loop {
            // On for 1s, off for 1s.
            led.set_high().unwrap();
            delay( clocks.hclk().0 );
            led.set_low().unwrap();
            delay( clocks.hclk().0 );
        }
    }

    loop {}
}
