//! This module provides support for the raspberry pi's general purpose input output (gpio) pins

use super::mmio;

const PINS: u32 = 53;

const GPIO_BASE_OFFSET: u32 = 0x00200000;

const GPFSEL_SIZE: u32 = 10;
const GPFSEL_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0;

const GPSET_SIZE: u32 = 32;
const GPSET_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x1c;

const GPCLR_SIZE: u32 = 32;
const GPCLR_BASE_OFFSET: u32 = GPIO_BASE_OFFSET + 0x28;

/// Structure that represents the RGB status light.
/// All methods assume that pin mode has not changed and when turning on a light that the other ones are off
pub struct StatusLight {
    red_pin: Pin,
    green_pin: Pin,
    blue_pin: Pin,
}

impl StatusLight {
    const RED_PIN: u32 = 17;
    const GREEN_PIN: u32 = 27;
    const BLUE_PIN: u32 = 22;

    /// Initializes a status light and sets the pins to output mode
    pub fn init() -> Self {
        let red_pin = Pin::new(StatusLight::RED_PIN).unwrap();
        let green_pin = Pin::new(StatusLight::GREEN_PIN).unwrap();
        let blue_pin = Pin::new(StatusLight::BLUE_PIN).unwrap();

        red_pin.set_mode(Mode::OUT);
        green_pin.set_mode(Mode::OUT);
        blue_pin.set_mode(Mode::OUT);

        StatusLight {
            red_pin,
            green_pin,
            blue_pin
        }
    }

    /// sets the right light
    pub fn set_red(&self, level: OutputLevel) {
        self.red_pin.set_out(level);
    }

    /// sets the green light
    pub fn set_green(&self, level: OutputLevel) {
        self.green_pin.set_out(level);
    }

    /// sets the blue light
    pub fn set_blue(&self, level: OutputLevel) {
        self.blue_pin.set_out(level);
    }
}

/// Pin is a wrapper class for a u32 representing the pin number which ensures that any number inside is a valid pin number
pub struct Pin {
    number: u32,
}

impl Pin {

    /// Constructor that returns an error if an out of range number is supplied
    pub fn new(number: u32) -> Result<Self, ()> {
        if number > PINS {
            Err(())
        } else {
            Ok(Pin { number })
        }
    }

    /// Sets the mode of the pin
    pub fn set_mode(&self, mode: Mode) {
        let mut data = self.get_gpfsel();
        let offset = self.gpfsel_offset();

        data &= !(111 << offset);
        data |= (mode as u32) << offset;
        mmio::write_at_offset(data, (GPFSEL_BASE_OFFSET +  self.gpfsel_block() * 4) as usize);
    }

    /// Gets the mode of the pin or an empty tuple if the mode is not recognized. If Err(()) is returned, something has gone really wrong.
    pub fn get_mode(&self) -> Result<Mode, ()> {
        let offset = self.gpfsel_offset();

        Mode::from_u32(self.get_gpfsel() >> offset & 111)
    }

    /// Sets the output of an output pin to the desired level
    /// Note: this does not check that the pin is set to output
    pub fn set_out(&self, output: OutputLevel) {
        match output {
            OutputLevel::High => {
                mmio::write_at_offset(
                    self.get_gpset() | (1 << self.gpset_offset()),
                    (GPSET_BASE_OFFSET + self.gpset_block() * 4) as usize
                );
            },
            OutputLevel::Low => {
                mmio::write_at_offset(
                    self.get_gpclr() | 1 << self.gpclr_offset(),
                    (GPCLR_BASE_OFFSET + self.gpclr_block() * 4) as usize
                );
            }
        }
    }

    /// Gets the output value of a pin
    pub fn get_out(&self) -> Result<OutputLevel, ()> {
        let offset = self.gpset_offset();

        match self.get_gpset() >> offset & 1 {
            0 => Ok(OutputLevel::Low),
            1 => Ok(OutputLevel::High),
            _ => Err(())
        }
    }

    fn get_gpfsel(&self) -> u32 {
        mmio::read_at_offset((GPFSEL_BASE_OFFSET + self.gpfsel_block() * 4) as usize)
    }

    fn gpfsel_block(&self) -> u32 {
        self.number / GPFSEL_SIZE
    }

    fn gpfsel_offset(&self) -> u32 {
        3 * (self.number % GPFSEL_SIZE)
    }

    fn get_gpset(&self) -> u32 {
        mmio::read_at_offset((GPSET_BASE_OFFSET + self.gpset_block() * 4) as usize)
    }

    fn gpset_block(&self) -> u32 {
        self.number / GPSET_SIZE
    }

    fn gpset_offset(&self) -> u32 {
        self.number % GPSET_SIZE
    }

    fn get_gpclr(&self) -> u32 {
        mmio::read_at_offset((GPCLR_BASE_OFFSET + self.gpclr_block() * 4) as usize)
    }

    fn gpclr_block(&self) -> u32 {
        self.number / GPCLR_SIZE
    }

    fn gpclr_offset(&self) -> u32 {
        self.number % GPCLR_SIZE
    }
}

/// All possible pinmodes for a gpio pin
#[derive(PartialEq)]
#[derive(Debug)]
pub enum Mode {
    IN = 0b000,
    OUT = 0b001,
    AF0 = 0b100,
    AF1 = 0b101,
    AF2 = 0b110,
    AF3 = 0b111,
    AF4 = 0b011,
    AF5 = 0b010
}

/// Represents the possible output values of a pin
#[derive(PartialEq)]
#[derive(Debug)]
pub enum OutputLevel {
    High,
    Low
}

impl Mode {
    /// Takes a u32 an returns a Mode if the value represents a valid pin mode, otherwise Err(())
    pub fn from_u32(value: u32) -> Result<Mode, ()> {
        match value {
            0b000u32 => Ok(Mode::IN),
            0b001u32 => Ok(Mode::OUT),
            0b100u32 => Ok(Mode::AF0),
            0b101u32 => Ok(Mode::AF1),
            0b110u32 => Ok(Mode::AF2),
            0b111u32 => Ok(Mode::AF3),
            0b011u32 => Ok(Mode::AF4),
            0b010u32 => Ok(Mode::AF5),
            _ => Err(())
        }
    }
}

#[cfg(test)]
mod tests {
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
}