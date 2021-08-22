#![allow(dead_code)]
use crate::utils::math;
use core::ops::{Mul, Add, Sub, Neg};

#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub struct Vector (pub f64, pub f64);

impl Vector {
    pub fn origin() -> Self {
        Vector (0f64, 0f64)
    }

    pub fn magnitude(&self) -> f64 {
        math::sqrt(self.0 * self.0 + self.1 * self.1)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Vector (-self.0, -self.1)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector (self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Vector {
        Vector (self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        Vector (self * rhs.0, self * rhs.1)
    }
}

impl Mul for Vector {
    type Output = f64;

    fn mul(self, rhs: Vector) -> f64 {
        self.0 * rhs.0 + self.1 * rhs.1
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_dot() {
        assert_eq!(Vector (0.0, 0.0) * Vector (3.0, 5.0), 0.0);
        assert_eq!(Vector (4.0, 7.0) * Vector (3.0, 9.0), 75.0);
    }
}