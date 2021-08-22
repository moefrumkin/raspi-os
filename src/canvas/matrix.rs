#![allow(dead_code)]

use core::ops::{Mul, Add, Sub, Neg};
use super::{vector::Vector, line::Line};

/// A 2D matrix. Arbitrarily, each of the elements are rows
#[derive(Copy)]
#[derive(Clone)]
#[derive(PartialEq)]
#[derive(Debug)]
pub struct Matrix (Vector, Vector);

impl Matrix {
    pub fn columns(self) -> (Vector, Vector) {
        ( self.0, self.1 )
    }

    pub fn rows(self) -> (Vector, Vector) {
        (Vector (self.0.0, self.1.0),
         Vector (self.0.1, self.1.1 ))
    }

    pub fn identity() -> Self {
        Matrix (
            Vector (1f64, 0f64),
            Vector (0f64, 1f64)
        )
    }
}

impl Neg for Matrix {
    type Output = Matrix;

    fn neg(self) -> Matrix {
        Matrix (-self.0, -self.1)
    }
}

impl Add for Matrix {
    type Output = Matrix;

    fn add(self, rhs: Matrix) -> Self {
        Matrix ( self.0 + rhs.0, self.1 + rhs.1 )
    }
}

impl Sub for Matrix {
    type Output = Matrix;

    fn sub(self, rhs: Matrix) -> Self {
        Matrix ( self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul<Matrix> for f64 {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Matrix {
        Matrix ( self * rhs.0, self * rhs.1 )
    }
}

impl Mul<Vector> for Matrix {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        let (top, bottom) = self.rows();
        Vector (top * rhs, bottom * rhs)
    }
}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Matrix {
        let (top, bottom) = self.columns();
        let (left, right) = rhs.rows();

        Matrix (
            Vector (top * left, top * right),
            Vector (bottom * left, bottom * right)
        )
    }
}

impl Mul<Line> for Matrix {
    type Output = Line;

    fn mul(self, rhs: Line) -> Line {
        Line (self * rhs.0, self * rhs.1 )
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let matrix = Matrix ( Vector (17f64, 8.93, ), Vector (7.282, 3.1415926535897932384626433832795));

        assert_eq!(matrix * Matrix::identity(), matrix);
        assert_eq!(Matrix::identity() * matrix, matrix);
    }
}