//! Blinks an LED
//! 
//! Measures the length of a rope run over a quaderature encoder and displays the result on a HD44780 compatible LED display.
#![no_main]
#![no_std]

// Halt on panic
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust#53964
extern crate panic_halt; // panic handler

use core::cell::{Cell, RefCell};

use cortex_m;
use cortex_m_rt::entry;
use cortex_m::interrupt::{free, CriticalSection, Mutex};
use core::ops::DerefMut;
//use cortex_m::asm::delay;
//use ufmt::uwrite;
use core::fmt::Write;
use heapless::String;
use heapless::consts::*;
use embedded_hal::Direction as RotaryDirection;

use stm32f4xx_hal as hal;
use crate::hal::{prelude::*, 
                interrupt,
                gpio::{gpiob::PB8, Edge, ExtiPin, Input, Floating},
                stm32,
                qei::Qei};

mod display;
mod counted_length;

//static QUAD_A: Mutex<RefCell<Option<PA11<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
//static QUAD_B: Mutex<RefCell<Option<PA12<Input<Floating>>>>> = Mutex::new(RefCell::new(None));
//static COUNT: Mutex<Cell<i32>> = Mutex::new(Cell::new(0));
//static OLD_STATE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static RESET_COUNTER: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static BUT1: Mutex<RefCell<Option<PB8<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut dp), Some(cp)) = (
        stm32::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        dp.RCC.apb2enr.write(|w| w.syscfgen().enabled());
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
        let mut a_pin = gpioa.pa15.into_push_pull_output();
        a_pin.set_low().unwrap();
        let mut b_pin = gpiob.pb3.into_push_pull_output();
        b_pin.set_low().unwrap();
        let mut but1 = gpiob.pb8.into_floating_input(); //mid button

        let _but2 = gpiob.pb9.into_floating_input(); //up button
        let _but3 = gpiob.pb7.into_floating_input(); //down button
        let rotary_encoder_pins = (
            a_pin.into_alternate_af1(),
            b_pin.into_alternate_af1()
        );
        

        let rotary_encoder_timer = dp.TIM2;
        let rotary_encoder = Qei::tim2(rotary_encoder_timer, rotary_encoder_pins);

        but1.make_interrupt_source(&mut dp.SYSCFG);
        but1.enable_interrupt(&mut dp.EXTI);
        but1.trigger_on_edge (&mut dp.EXTI, Edge::FALLING);

        free(|cs| {
            BUT1.borrow(cs).replace(Some(but1));
        });

        // Enable interrupts
        stm32::NVIC::unpend(hal::stm32::Interrupt::EXTI9_5);
        unsafe {
            stm32::NVIC::unmask(hal::stm32::Interrupt::EXTI9_5);
        };
        

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
        display.write_str("Length:");
        display.set_cursor_position(1, 0).unwrap();
        display.write_str("0 m");
        
        let mut current_count = rotary_encoder.count();
        let mut length = counted_length::CountedLength::new(0.01, 2048);
        loop {
            
            let reset_count = free(|cs| RESET_COUNTER.borrow(cs).get());
            
            let new_count = rotary_encoder.count();

            if (new_count != current_count) || (reset_count) {

                if reset_count {
                    free(|cs| RESET_COUNTER.borrow(cs).replace(false));
                    length.reset();
                } else {
                    let diff = new_count as i32 - current_count as i32;  
                    current_count = new_count;  
                    length.update_with_difference(diff);

                    // Light up the LED when turning clockwise, turn it off
                    // when turning counter-clockwise.
                    match rotary_encoder.direction() {
                        RotaryDirection::Upcounting => {
                            led.set_low().unwrap();
                        },
                        RotaryDirection::Downcounting => {
                            led.set_high().unwrap();
                        },
                    }
                }

                let mut count_str = String::<U15>::new();
                let _ = write!(count_str, "{:.3} m", length.get_length());
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
fn EXTI9_5(){
    free(|cs| {
        let mut btn_ref = BUT1.borrow(cs).borrow_mut();
        if let Some(ref mut btn) = btn_ref.deref_mut() {
            // We cheat and don't bother checking _which_ exact interrupt line fired - there's only
            // ever going to be one in this example.
            btn.clear_interrupt_pending_bit();
            RESET_COUNTER.borrow(cs).replace(true);
        }
    });
}