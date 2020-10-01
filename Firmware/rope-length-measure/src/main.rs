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

use crate::hal::{prelude::*, stm32};

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        //Create a delay abstraction based on SysTick
        let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

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

        let mut display = Display::new(data_bus, enable_pin, read_write_pin, register_select_pin, & mut delay);
        display.initialize_display();

        loop {
            // On for 1s, off for 1s.
            led.set_high().unwrap();
            delay.delay_ms(1000_u32);
            led.set_low().unwrap();
            delay.delay_ms(100_u32);
        }
    }

    loop {}
}

/// Structure than handles controlling a Newhaven 16x2 display
/// Details of the display are found in the NHD-0420 display documentation.
pub struct Display<'a, BUS, CTRL> {
    data_bus: [BUS; 8],
    enable: CTRL,
    read_write: CTRL,
    register_select: CTRL,

    delay: &'a mut stm32f4xx_hal::delay::Delay
}

impl<'a, BUS, CTRL> Display<'a, BUS, CTRL> {

    const WIDTH: u8 = 16;
    const HEIGHT: u8 = 2;

}

impl<'a, BUS, CTRL> Display<'a, BUS, CTRL> 
where BUS: embedded_hal::digital::v2::OutputPin, CTRL: embedded_hal::digital::v2::OutputPin  {
    

    /// Create a new Display with the data bus contained in an array of pins (must be on the same bus for the STM32F4xx_HAL) and control pins on
    /// other pins. For the type system to work, the pins need to be downgraded, and the bus pins and control pins have to be on the same 
    /// GPIO bus, respectively.
    pub fn new (bus: [BUS; 8], enable: CTRL, read_write: CTRL, register_select: CTRL, delay: &'a mut stm32f4xx_hal::delay::Delay) -> Display<BUS, CTRL> {
        Display {
            data_bus: bus, 
            enable: enable,
            read_write: read_write,
            register_select: register_select,
            delay: delay
        }
    }

    ///Initialize the display
    pub fn initialize_display(& mut self) {
        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };
        self.delay.delay_ms(100_u8);
        self.send_command(0x30_u8);
        self.delay.delay_ms(30_u8);
        self.send_command(0x30_u8);
        self.delay.delay_ms(10_u8);
        self.send_command(0x30_u8);
        self.delay.delay_ms(10_u8);
        self.send_command(0x38_u8); // Set to 8 bit, 2 line mode
        //in 2 line mode, DDRAM address in the 1st line is from "00H" to "27H", and DDRAM address in the 2nd line is from "40H" to "67H".
        self.send_command(0x10_u8); //set cursor
        self.send_command(0x03_u8); //display on and cursur on
        self.send_command(0x06_u8); //entry mode set.
    }

    //Set the cursor position on the screen.
    pub fn set_cursor_position(& mut self, row: u8, column: u8) -> Result <(), ()> {
        if row >= Self::HEIGHT {
            return Err(());
        }
        if column >= Self::WIDTH {
            return Err(());
        }

        let mut cursor_position: u8;

        if row == 0 {
            cursor_position = 0x00_u8;
        } else {
            cursor_position = 0x40_u8;
        }

        cursor_position += column;

        cursor_position |= 1<<7;

        self.send_command(cursor_position);

        Ok(())
    }

    /// Write a byte to the bus by iterating over the bus array.
    fn write_to_bus (&mut self, data: u8) {
        for (i, pin) in self.data_bus.iter_mut().enumerate() {
            if (data & (1 << i)) > 0  {
                let _ = match pin.set_high() { //This nastiness is needed to get around a generics/embedded_hal problem I don't understand.
                    Ok(_) => {},
                    _ => {}
                };
            } else {
                let _ = match pin.set_low() {
                    Ok(_) => {},
                    _ => {}
                };
            }
        }
    }

    /// Send a command to the display
    fn send_command (&mut self, command: u8) {
        self.write_to_bus(command);
        let _ = match self.register_select.set_low() {
            Ok(_) => {},
            _ => {}
        };
        let _ = match self.read_write.set_low() {
            Ok(_) => {},
            _ => {}
        };
        let _ = match self.enable.set_high() {
            Ok(_) => {},
            _ => {}
        };

        self.delay.delay_ms(1_u8);

        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };
    }

    /// Write a character to the display at the pre-programmed location.
    pub fn write_char (&mut self, character: u8) {
        self.write_to_bus(character);
        let _ = match self.register_select.set_low() {
            Ok(_) => {},
            _ => {}
        };
        let _ = match self.read_write.set_high() {
            Ok(_) => {},
            _ => {}
        };
        let _ = match self.enable.set_high() {
            Ok(_) => {},
            _ => {}
        };

        self.delay.delay_ms(1_u8);

        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };
    }

    
}
