//! This module provides support for the raspberry pi's general purpose input output (gpio) pins

use core::cell::RefCell;
use alloc::rc::Rc;

use crate::{aarch64::cpu, bitfield, utils::bit_array::BitArray, volatile::Volatile};

const PINS: u32 = 53;

const GPIO_BASE_OFFSET: u32 = 0x00200000;

const GPFSEL_SIZE: u32 = 10;
const GPFSEL_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0;

const GPSET_SIZE: u32 = 32;
const GPSET_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x1c;

const GPCLR_SIZE: u32 = 32;
const GPCLR_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x28;

const GPHEN_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x64;

#[allow(dead_code)]
const GPPPUD: u32 = GPIO_BASE_OFFSET + 0x94;

#[allow(dead_code)]
const GPPUDCLK_SIZE: u32 = 32;
const GPPUDCLK_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x98;

#[repr(C)]
pub struct GPIORegisters {
    function_select_banks: [Volatile<FunctionSelectBlock>; 6],
    res0: u32,
    set_output: [Volatile<BitArray<u32>>; 2],
    res1: u32,
    clear_output: [Volatile<BitArray<u32>>; 2],
    res2: u32,
    gplev: [Volatile<BitArray<u32>>; 2],
    res3: u32,
    gpsed: [Volatile<u32>; 2],
    res4: u32,
    gpren: [Volatile<u32>; 2],
    res5: u32,
    gpfen: [Volatile<u32>; 2],
    res6: u32,
    high_detect_enable: [Volatile<BitArray<u32>>; 2],
    res7: u32,
    gplen: [Volatile<u32>; 2],
    res8: u32,
    gparen: [Volatile<u32>; 2],
    res9: u32,
    gpafen: [Volatile<u32>; 2],
    res10: u32,
    pull_register: Volatile<PullRegister>,
    pull_enable: [Volatile<BitArray<u32>>; 2]
}

pub struct GPIOController<'a> {
    registers: &'a mut GPIORegisters
}

#[allow(dead_code)]
impl<'a> GPIOController<'a> {
    pub const fn with_registers(registers: &'a mut GPIORegisters) -> Self {
        Self {
            registers
        }
    }

    pub fn set_mode(&mut self, pin: Pin, mode: Mode) {
        self.registers.function_select_banks[
            pin.function_select_bank_number()
        ].map_closure(&|bank: FunctionSelectBlock|
            bank.set_pin_mode(pin.number_in_function_select_bank(), mode)
        );
    }

    /// Sets the output of an output pin to the desired level
    /// Note: this does not check that the pin is set to output
    pub fn set_out(&mut self, pin: Pin, output: OutputLevel) {
        match output {
            OutputLevel::High => {
                self.registers.set_output[pin.set_block()].map_closure(&move |output_block: BitArray<u32>|
                    output_block.set_bit(pin.set_offset(), 1)
                );
            }
            OutputLevel::Low => {
                self.registers.clear_output[pin.clear_block()].map_closure(&move |clear_block: BitArray<u32>|
                    clear_block.set_bit(pin.clear_offset(), 1)
                );
            }
        }
    }

    /// Sets the pullup mode of a register
    /// You should remember this, there is no way of reading the mode once set
    /// It takes > 300 clock cycles for this instuction to run because of the wait time after setting the pull mode
    /// TODO: this should have an array slice version because the waits take a while
    pub fn pull(&mut self, pin: Pin, mode: Pull) {
        let pull_enable_block = pin.pull_enable_block();
        let pull_enable_offset = pin.pull_enable_offset();

        self.registers.pull_register.set(PullRegister::mode(mode));

        cpu::wait_for_cycles(150);

        self.registers.pull_enable[pull_enable_block].map_closure(&|pull_enable|
            pull_enable.set_bit(pull_enable_offset, 1)
        );

        cpu::wait_for_cycles(150);

        self.registers.pull_enable[pull_enable_block].map_closure(&|pull_enable|
            pull_enable.set_bit(pull_enable_offset, 0)
        );
    }

    fn get_pin_mode(&self, pin: Pin) -> u32 {
        self.registers.function_select_banks[pin.function_select_bank_number()].get()
            .get_pin_mode(pin.number_in_function_select_bank())
    }

    // TODO: do this better
    pub fn set_high_detect_enable(&mut self, pin: Pin, value: u32) {
        let bank;
        if pin.number < 32  {
            bank = 0;
        } else {
            bank = 1
        }

        let offset_in_bank = pin.number - 32 * bank;

        self.registers.high_detect_enable[bank as usize].map_closure(&|detect_enable|
            detect_enable.set_bit(offset_in_bank as usize, value)
        );
    }
}

bitfield! {
    FunctionSelectBlock(u32) {

    } with {
        const PIN_STATUS_BITS: u32 = 3;
        // TODO: Derive from PIN_STATUS_BITS?
        const PIN_STATUS_MASK: u32 = 0b111;

        fn get_pin_mode(&self, pin_number_in_block: u32) -> u32 {
            (self.value >> (pin_number_in_block * Self::PIN_STATUS_BITS)) & Self::PIN_STATUS_MASK
        }

        fn set_pin_mode(&self, pin_number_in_block: u32, mode: Mode) -> Self {
            let shifted_inverted_mask = !(Self::PIN_STATUS_MASK <<
                (pin_number_in_block * Self::PIN_STATUS_BITS));

            let value = (self.value & shifted_inverted_mask) | 
                ((mode as u32) << (pin_number_in_block * Self::PIN_STATUS_BITS));

            Self { value }
        }
    }
}


bitfield! {
    PullRegister(u32) {
        pull_mode: 0-1
    } with {
        pub fn mode(mode: Pull) -> Self {
            Self {
                value: mode as u32
            }
        }
    }
}

/// Structure that represents the RGB status light.
/// All methods assume that pin mode has not changed and when turning on a light that the other ones are off
pub struct StatusLight<'a> {
    red_pin: Pin,
    green_pin: Pin,
    blue_pin: Pin,
    gpio_controller: &'a mut GPIOController<'a>,
}

impl<'a> StatusLight<'a> {
    const RED_PIN: u32 = 17;
    const GREEN_PIN: u32 = 27;
    const BLUE_PIN: u32 = 22;

    /// Initializes a status light and sets the pins to output mode
    pub const fn new(gpio_controller: &'a mut GPIOController<'a>) -> Self {
        let red_pin = Pin::new_unchecked(StatusLight::RED_PIN);
        let green_pin = Pin::new_unchecked(StatusLight::GREEN_PIN);
        let blue_pin = Pin::new_unchecked(StatusLight::BLUE_PIN);

        StatusLight {
            red_pin,
            green_pin,
            blue_pin,
            gpio_controller,
        }
    }

    pub fn init(&mut self) {
        self.gpio_controller.set_mode(self.red_pin, Mode::OUT);
        self.gpio_controller.set_mode(self.green_pin, Mode::OUT);
        self.gpio_controller.set_mode(self.blue_pin, Mode::OUT);
    }

    /// sets the right light
    pub fn set_red(&mut self, level: OutputLevel) {
        self.gpio_controller.set_out(self.red_pin, level);
    }

    /// sets the green light
    pub fn set_green(&mut self, level: OutputLevel) {
        self.gpio_controller.set_out(self.green_pin, level);
    }

    /// sets the blue light
    pub fn set_blue(&mut self, level: OutputLevel) {
        self.gpio_controller.set_out(self.blue_pin, level);
    }
}

/// Pin is a wrapper class for a u32 representing the pin number which ensures that any number inside is a valid pin number
#[derive(Copy, Clone)]
pub struct Pin {
    number: u32,
}

#[allow(dead_code)]
impl Pin {
    /// Constructor that returns an error if an out of range number is supplied
    pub const fn new(number: u32) -> Result<Self, ()> {
        if number > PINS {
            Err(())
        } else {
            Ok(Pin { number })
        }
    }

    pub const fn new_unchecked(number: u32) -> Self {
        Self { number }
    }

    fn function_select_bank_number(&self) -> usize {
        (self.number / GPFSEL_SIZE) as usize
    }

    fn number_in_function_select_bank(&self) -> u32 {
        self.number % GPFSEL_SIZE
    }

    fn set_block(&self) -> usize {
        (self.number / GPSET_SIZE) as usize
    }

    fn set_offset(&self) -> usize {
        (self.number % GPSET_SIZE) as usize
    }

    fn clear_block(&self) -> usize {
        (self.number / GPCLR_SIZE) as usize
    }

    fn clear_offset(&self) -> usize {
        (self.number % GPCLR_SIZE) as usize
    }

    fn pull_enable_block(&self) -> usize {
        (self.number / GPPUDCLK_SIZE) as usize
    }

    fn pull_enable_offset(&self) -> usize {
        (self.number % GPPUDCLK_SIZE) as usize
    }
}

/// All possible pinmodes for a gpio pin
#[derive(PartialEq, Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Mode {
    IN = 0b000,
    OUT = 0b001,
    AF0 = 0b100,
    AF1 = 0b101,
    AF2 = 0b110,
    AF3 = 0b111,
    AF4 = 0b011,
    AF5 = 0b010,
}

/// Represents the possible output values of a pin
#[derive(PartialEq, Debug)]
pub enum OutputLevel {
    High,
    Low,
}

#[allow(dead_code)]
pub enum Pull {
    Off = 0b00,
    Down = 0b01,
    Up = 0b10,
}

//#[cfg(test)]
/*mod tests {
    use crate::platform::gpio::{Pin, PINS, Mode, OutputLevel};
    const ZERO: Pin = Pin { number : 0 };
    const NINE: Pin = Pin { number : 9 };
    const TWELVE: Pin = Pin { number : 12 };
    const TWENTY: Pin = Pin { number : 20 };
    const TWENTY_FIVE: Pin = Pin { number : 25 };
    const FIFTY: Pin = Pin { number : 50 };
    const FIFTY_THREE: Pin = Pin { number : 53 };

    #[test]
    #[should_panic]
    fn test_bounds() {
        Pin::new(PINS + 1).unwrap();
    }

    #[test]
    fn gpfsel_block() {
        assert_eq!(ZERO.gpfsel_block(), 0);
        assert_eq!(NINE.gpfsel_block(), 0);
        assert_eq!(TWELVE.gpfsel_block(), 1);
        assert_eq!(TWENTY.gpfsel_block(), 2);
        assert_eq!(TWENTY_FIVE.gpfsel_block(), 2);
        assert_eq!(FIFTY.gpfsel_block(), 5);
        assert_eq!(FIFTY_THREE.gpfsel_block(), 5);
    }

    #[test]
    fn gpfsel_offset() {
        assert_eq!(ZERO.gpfsel_offset(), 0);
        assert_eq!(NINE.gpfsel_offset(), 27);
        assert_eq!(TWELVE.gpfsel_offset(), 6);
        assert_eq!(TWENTY.gpfsel_offset(), 0);
        assert_eq!(TWENTY_FIVE.gpfsel_offset(), 15);
        assert_eq!(FIFTY.gpfsel_offset(), 0);
        assert_eq!(FIFTY_THREE.gpfsel_offset(), 9);
    }

    #[test]
    fn gpset_block() {
        assert_eq!(ZERO.gpset_block(), 0);
        assert_eq!(NINE.gpset_block(), 0);
        assert_eq!(TWELVE.gpset_block(), 0);
        assert_eq!(TWENTY.gpset_block(), 0);
        assert_eq!(TWENTY_FIVE.gpset_block(), 0);
        assert_eq!(FIFTY.gpset_block(), 1);
        assert_eq!(FIFTY_THREE.gpset_block(), 1);
    }

    #[test]
    fn gpset_offset() {
        assert_eq!(ZERO.gpset_offset(), 0);
        assert_eq!(NINE.gpset_offset(), 9);
        assert_eq!(TWELVE.gpset_offset(), 12);
        assert_eq!(TWENTY.gpset_offset(), 20);
        assert_eq!(TWENTY_FIVE.gpset_offset(), 25);
        assert_eq!(FIFTY.gpset_offset(), 18);
        assert_eq!(FIFTY_THREE.gpset_offset(), 21);
    }

    #[test]
    fn gpclr_block() {
        assert_eq!(ZERO.gpclr_block(), 0);
        assert_eq!(NINE.gpclr_block(), 0);
        assert_eq!(TWELVE.gpclr_block(), 0);
        assert_eq!(TWENTY_FIVE.gpclr_block(), 0);
        assert_eq!(FIFTY.gpclr_block(), 1);
        assert_eq!(FIFTY_THREE.gpclr_block(), 1);
    }

    #[test]
    fn gpclr_offset() {
        assert_eq!(ZERO.gpclr_offset(), 0);
        assert_eq!(NINE.gpclr_offset(), 9);
        assert_eq!(TWELVE.gpclr_offset(), 12);
        assert_eq!(TWENTY.gpclr_offset(), 20);
        assert_eq!(TWENTY_FIVE.gpclr_offset(), 25);
        assert_eq!(FIFTY.gpclr_offset(), 18);
        assert_eq!(FIFTY_THREE.gpclr_offset(), 21)
    }

    #[test]
    fn gppudclk_block() {
        assert_eq!(ZERO.gppudclk_block(), 0);
        assert_eq!(NINE.gppudclk_block(), 0);
        assert_eq!(TWELVE.gppudclk_block(), 0);
        assert_eq!(TWENTY_FIVE.gppudclk_block(), 0);
        assert_eq!(FIFTY.gppudclk_block(), 1);
        assert_eq!(FIFTY_THREE.gppudclk_block(), 1);
    }

        #[test]
    fn gppudclk_offset() {
        assert_eq!(ZERO.gppudclk_offset(), 0);
        assert_eq!(NINE.gppudclk_offset(), 9);
        assert_eq!(TWELVE.gppudclk_offset(), 12);
        assert_eq!(TWENTY.gppudclk_offset(), 20);
        assert_eq!(TWENTY_FIVE.gppudclk_offset(), 25);
        assert_eq!(FIFTY.gppudclk_offset(), 18);
        assert_eq!(FIFTY_THREE.gppudclk_offset(), 21)
    }


    #[test]
    fn set_mode() {
        ZERO.set_mode(Mode::OUT);
        NINE.set_mode(Mode::AF0);
        TWELVE.set_mode(Mode::AF5);
        TWENTY.set_mode(Mode::AF3);
        TWENTY_FIVE.set_mode(Mode::AF1);
        FIFTY.set_mode(Mode::AF2);
        FIFTY_THREE.set_mode(Mode::AF4);

        assert_eq!(ZERO.get_mode().unwrap(), Mode::OUT);
        assert_eq!(NINE.get_mode().unwrap(), Mode::AF0);
        assert_eq!(TWELVE.get_mode().unwrap(), Mode::AF5);
        assert_eq!(TWENTY.get_mode().unwrap(), Mode::AF3);
        assert_eq!(TWENTY_FIVE.get_mode().unwrap(), Mode::AF1);
        assert_eq!(FIFTY.get_mode().unwrap(), Mode::AF2);
        assert_eq!(FIFTY_THREE.get_mode().unwrap(), Mode::AF4);
    }

    #[test]
    fn set_out() {
        ZERO.set_out(OutputLevel::Low);
        NINE.set_out(OutputLevel::High);
        TWELVE.set_out(OutputLevel::High);
        TWENTY.set_out(OutputLevel::Low);
        TWENTY_FIVE.set_out(OutputLevel::High);
        FIFTY.set_out(OutputLevel::High);
        FIFTY_THREE.set_out(OutputLevel::High);

        assert_eq!(ZERO.get_out().unwrap(), OutputLevel::Low);
        assert_eq!(NINE.get_out().unwrap(), OutputLevel::High);
        assert_eq!(TWELVE.get_out().unwrap(), OutputLevel::High);
        assert_eq!(TWENTY.get_out().unwrap(), OutputLevel::Low);
        assert_eq!(TWENTY_FIVE.get_out().unwrap(), OutputLevel::High);
        assert_eq!(FIFTY.get_out().unwrap(), OutputLevel::High);
        assert_eq!(FIFTY_THREE.get_out().unwrap(), OutputLevel::High);
    }
}*/
