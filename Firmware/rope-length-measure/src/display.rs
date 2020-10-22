
use cortex_m::asm::delay;
use stm32f4xx_hal::time::Hertz;

// Structure than handles controlling a Newhaven 16x2 display
/// Details of the display are found in the NHD-0420 display documentation.
pub struct Display<BUS, CTRL> {
    data_bus: [BUS; 8],
    enable: CTRL,
    read_write: CTRL,
    register_select: CTRL,

    cpu_clk: Hertz
}

impl<'a, BUS, CTRL> Display<BUS, CTRL> {

    const WIDTH: u8 = 16;
    const HEIGHT: u8 = 2;

}

impl<'a, BUS, CTRL> Display<BUS, CTRL> 
where BUS: embedded_hal::digital::v2::OutputPin, CTRL: embedded_hal::digital::v2::OutputPin  {
    

    /// Create a new Display with the data bus contained in an array of pins (must be on the same bus for the STM32F4xx_HAL) and control pins on
    /// other pins. For the type system to work, the pins need to be downgraded, and the bus pins and control pins have to be on the same 
    /// GPIO bus, respectively.
    pub fn new (bus: [BUS; 8], enable: CTRL, read_write: CTRL, register_select: CTRL, cpu_clk: Hertz) -> Display<BUS, CTRL> {
        Display {
            data_bus: bus, 
            enable: enable,
            read_write: read_write,
            register_select: register_select,
            cpu_clk: cpu_clk
        }
    }

    pub fn write_str(&mut self, string: &str) {
        self.write_bytes(string.as_bytes());
    }

    fn write_bytes (&mut self, string: &[u8]) {
        for &b in string {
            self.write_to_address(b);
        }
    }

    pub fn delay_ms(&self, ms: u32) {
        delay( (self.cpu_clk.0) / 1000_u32 * ms);
    }

    /// Initialize the display
    pub fn initialize_display(& mut self) {
        self.delay_ms(15);
        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };
        self.delay_ms(100);
        self.send_command(0x30_u8); //set 8 bit mode
        self.delay_ms(1);
        self.send_command(0b0011_1000); //8 bit, enable 5x7 mode
        self.delay_ms(1);
        self.send_command(0b0011_1000);
        self.delay_ms(1);
        self.send_command(0b0000_1110);
        self.delay_ms(1);
        self.send_command(0b0000_0001);
        self.delay_ms(1);
        self.send_command(0b0000_0111);
        self.delay_ms(1);

        //self.send_command(0x38_u8); // Set to 8 bit, 2 line mode
        //in 2 line mode, DDRAM address in the 1st line is from "00H" to "27H", and DDRAM address in the 2nd line is from "40H" to "67H".
        //self.send_command(0x10_u8); //set cursor
        //self.send_command(0x03_u8); //display on and cursur on
        self.send_command(0x06_u8); //entry mode set.
    }

    /// Set the cursor position on the screen.
    pub fn set_cursor_position(& mut self, row: u8, column: u8) -> Result <(), ()> {
        if row >= Self::HEIGHT {
            return Err(());
        }
        if column >= Self::WIDTH +1{ //allow for parking of the cursor
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

        self.delay_ms(2);

        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };

        self.delay_ms(2);
    }

    /// Write a character to the display at the pre-programmed location.
    fn write_to_address (&mut self, character: u8) {
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

        self.delay_ms(2);

        let _ = match self.enable.set_low() {
            Ok(_) => {},
            _ => {}
        };

        self.delay_ms(2);
    }

    
}
