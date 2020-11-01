pub struct CountedLength {
    position: i64,
    radius: f32,
    pulses_per_revolution: u32
}

impl CountedLength {
    const REV_RAD: f32 = 2.0*3.14159265;
}

impl CountedLength {
    pub fn new(radius: f32, pulses_per_revolution: u32) -> CountedLength {
        CountedLength {
            position: 0i64,
            radius: radius,
            pulses_per_revolution: pulses_per_revolution
        }
    }

    pub fn reset (&mut self) {
        self.position=0;
    }

    pub fn update_with_difference (&mut self, difference: i32) {
        self.position += difference as i64;
    }

    fn pulses_to_m (&self, pulses: i64) -> f32 {
        let revolutions = pulses as f32 / self.pulses_per_revolution as f32;
        CountedLength::REV_RAD * revolutions*self.radius
    }

    pub fn get_length (&self) -> f32 {
        self.pulses_to_m (self.position)
    }
}