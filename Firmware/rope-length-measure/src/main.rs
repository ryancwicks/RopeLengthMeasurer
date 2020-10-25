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

use core::cell::{Cell, RefCell};

use cortex_m;
use cortex_m_rt::entry;
use cortex_m::interrupt::{free, CriticalSection, Mutex};
//use cortex_m::asm::delay;
//use ufmt::uwrite;
use core::fmt::Write;
use heapless::String;
use heapless::consts::*;
use embedded_hal::Direction as RotaryDirection;

use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, 
                interrupt,
                gpio::{gpioa::PA11, gpioa::PA12, Edge, ExtiPin, Input, Floating},
                stm32,
                qei::Qei};

mod display;

//static QUAD_A: Mutex<RefCell<Option<PA11<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
//static QUAD_B: Mutex<RefCell<Option<PA12<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
//static COUNT: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
//static OLD_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up the system clock. We want to run at 48MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
        let mut hal_delay = hal::delay::Delay::new(cp.SYST, clocks);

        // Set up the ports
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();

        // Set up the LED.
        let mut led = gpioc.pc13.into_push_pull_output();
        led.set_high().unwrap();

        // Set up the GPIO pins
        //let _x = gpioa.pa15.into_floating_input();
        let _but1 = gpiob.pb8.into_floating_input();
        let _but2 = gpiob.pb9.into_floating_input();
        let rotary_encoder_pins = (
            gpioa.pa15.into_alternate_af1(),
            gpiob.pb3.into_alternate_af1()
        );
        

        let rotary_encoder_timer = dp.TIM2;
        let rotary_encoder = Qei::tim2(rotary_encoder_timer, rotary_encoder_pins);


        //but1.make_interrupt_source(&mut dp.SYSCFG);
        //but1.enable_interrupt(&mut dp.EXTI);
        //but1.trigger_on_edge (&mut dp.EXTI, Edge::RISING);
        

        //Set up the display pins
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
        display.write_str("Rope Measurer");
        display.set_cursor_position(1, 0).unwrap();
        display.write_str("0");
        
        let mut current_count = rotary_encoder.count();
        loop {
            
            let new_count = rotary_encoder.count();

            if new_count != current_count {
    
                // Light up the LED when turning clockwise, turn it off
                // when turning counter-clockwise.
                match rotary_encoder.direction() {
                    RotaryDirection::Upcounting => led.set_low().unwrap(),
                    RotaryDirection::Downcounting => led.set_high().unwrap(),
                }
    
                current_count = new_count;

                let mut count_str = String::<U8>::new();
                let _ = write!(count_str, "{}", current_count);
                display.set_cursor_position(1, 0).unwrap();
                for _ in 0..count_str.len()+1 {
                    display.write_str(" ");
                }
                display.set_cursor_position(1, 0).unwrap();
                display.write_str(count_str.as_str());
            }
    
            hal_delay.delay_ms(10_u32);         
        }
    }

    loop {}
}

#[interrupt]
fn EXTI15_10(){
    free(|cs| {
        
    });
}