use super::gpio::{GPIOController, Pin, OutputLevel, Mode};
use super::timer::Timer;

/*pub struct LCDController<'a> {
    gpio: &'a GPIOController<'a>,
    timer: &'a Timer<'a>,
    register_select: Pin,
    enable: Pin,
    bus: [Pin; 4]
}

impl<'a> LCDController<'a> {
    /// Creates a new LCDController
    pub fn init(gpio: &'a GPIOController, timer: &'a Timer, register_select: Pin, enable: Pin, bus: [Pin; 4]) -> Self {

        gpio.set_mode(register_select, Mode::OUT);
        gpio.set_mode(enable, Mode::OUT);
        for pin in bus {
            gpio.set_mode(pin, Mode::OUT);
        }

        let lcd = Self {
            gpio,
            timer,
            register_select,
            enable,
            bus
        };

        //try to set 4 bit mode three times
        for _ in 0..3 {
            lcd.write4(0b11);
            // > min 4.1 millis
            timer.delay_microseconds(4500);
        }

        //set to 4 bit
        lcd.write4(0b10);


        //init with settings: 4 bit, 2 lines, 5x7
        lcd.send_command(0b0010_1000);

        //turn on without cursor
        lcd.send_command(0b0000_1000);

        lcd.clear();

        //set entry mode to to increment
        lcd.send_command(0b0000_0110);

        lcd
    }

    pub fn clear(&self) {
        self.send_command(1);
        self.timer.delay_microseconds(2000);
    }

    fn send_command(&self, command: u8) {
        self.gpio.set_out(self.register_select, OutputLevel::Low);
        self.write4((command as u8) >> 4);
        self.write4(command as u8);
    }

    pub fn send_data(&self, data: u8) {
        self.gpio.set_out(self.register_select, OutputLevel::High);
        self.write4(data >> 4);
        self.write4(data);
    }

    /// write the lower 4 bits of value to db4-7
    fn write4(&self, data: u8) {
        for i in 0..4 {
            self.gpio.set_out(self.bus[i], if (data >> i) | 0b1 == 0 {OutputLevel::Low} else {OutputLevel::High});
        }
        self.pulse();
    }

    fn pulse(&self) {
        self.gpio.set_out(self.enable, OutputLevel::Low);
        self.timer.delay_microseconds(1);
        self.gpio.set_out(self.enable, OutputLevel::High);
        self.timer.delay_microseconds(1);
        self.gpio.set_out(self.enable, OutputLevel::Low);
        self.timer.delay_microseconds(100);
    }
}
*/