#![allow(dead_code)]

use alloc::{boxed::Box, vec::Vec};
use crate::utils::math;
use super::{Draw, line::Line, matrix::Matrix, vector::Vector};

pub type Color = u32;

#[allow(dead_code)]
pub struct Canvas2D<'a, T> where T: Draw {
    gpu: &'a mut T,
    pix_width: usize,
    pix_height: usize,
    points: Vec<(Vector, Color)>,
    lines: Vec<(Line, Color)>
}

impl<'a, T> Canvas2D<'a, T> where T: Draw{
    pub fn new(gpu: &'a mut T, pix_width: usize, pix_height: usize) -> Self {
        Self {
            gpu,
            pix_width,
            pix_height,
            points: Vec::new(),
            lines: Vec::new()
        }
    }

    pub fn draw(&mut self, width: f64, height: f64) {
        let origin = Vector (0.0, 0.0);

        for x in 0..1080 {
            self.gpu.draw(x, x, 0xffffff);
        }

        //draw points
        for &(point, color) in &self.points {
            unimplemented!();
        }
    }

    //TODO: repeatedly using has unnecessary computation
    fn map(min_in: f64, max_in: f64, min_out: f64, max_out: f64, val: f64) -> f64 {
        (val - min_in) * (max_out - min_out) / (max_in - min_in) + min_out
    }

    pub fn transform(&mut self, transformation: Matrix) {
        for i in 0..self.points.len() {
            self.points[i] = (transformation * self.points[i].0, self.points[i].1);
        }

        for i in 0..self.lines.len() {
            self.lines[i] = (transformation * self.lines[i].0, self.lines[i].1);
        }
    }

    pub fn add_point(&mut self, point: Vector, color: Color) {
        self.points.push((point, color));
    }

    pub fn add_line(&mut self, line: Line, color: Color) {
        self.lines.push((line, color));
    }
}

