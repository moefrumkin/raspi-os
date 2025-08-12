use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub trait BitArrayBacking: 
    Copy
    + Shr<usize, Output = Self>
    + Shl<usize, Output = Self>
    + Not<Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self> {

    const BIT_MASK: Self;
}

impl BitArrayBacking for u32 {
    const BIT_MASK: Self = 0b1;
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BitArray<T> {
    array: T
}

impl<T> BitArray<T> where
    T: BitArrayBacking
{
    pub fn get_bit(&self, bit: usize) -> T {
        (self.array >> bit) & T::BIT_MASK
    }

    pub fn set_bit(&self, bit: usize, value: T) -> Self {
        Self {
            array: (self.array & !(T::BIT_MASK << bit)) | (value << bit)
        }
    }
}