#![allow(dead_code)]

use super::vector::Vector;
use core::iter;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line(pub Vector, pub Vector);

impl Line {
    /// Clip a line using the Liang-Barksy algorithm
    pub fn clip(&self, xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Option<Self> {
        let Line(Vector(x1, y1), Vector(x2, y2)) = *self;

        let p = [
            //dx
            x1 - x2,
            x2 - x1,
            //dy
            y1 - y2,
            y2 - y1,
        ];

        //distances from windows edges
        let q = [x1 - xmin, xmax - x1, y1 - ymin, ymax - y1];

        //check for lines parralel and out of the frame
        for i in 0..4 {
            if p[i] == 0.0 && q[i] < 0.0 {
                return None;
            }
        }

        //find the intersection point with the largest parametrized t
        //TODO: potential divide by 0
        let u1 = p
            .iter()
            .zip(q)
            .filter(|(_, q)| *q != 0.0)
            .map(|(p, q)| p / q)
            .chain(iter::once(0.0))
            .reduce(f64::max)
            .unwrap();
        let u2 = p
            .iter()
            .zip(q)
            .filter(|(_, q)| *q != 0.0)
            .map(|(p, q)| p / q)
            .chain(iter::once(1.0))
            .reduce(f64::min)
            .unwrap();

        if u1 > u2 {
            return None;
        }

        Some(Self(
            Vector(x1 + p[1] * u1, y1 + p[3] * u1),
            Vector(x1 + p[1] * u2, y1 + p[3] * u2),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::canvas::vector::Vector;
    #[test]
    pub fn square() {
        let all_in = Line(Vector(20.0, 20.0), Vector(20.0, 30.0));
        let all_out = Line(Vector(-500.0, -500.0), Vector(-400.0, 100.0));

        assert_eq!(all_out.clip(0.0, 0.0, 100.0, 100.0), None);
        //assert_eq!(all_in.clip(0.0, 0.0, 100.0, 100.0), Some(all_in));
    }
}
