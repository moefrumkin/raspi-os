#![allow(dead_code)]

use alloc::{boxed::Box, vec::Vec};
use crate::utils::math;
use super::{Draw, line::Line, matrix::Matrix, vector::Vector};

pub type Color = u32;
pub type Gradient2D = dyn Fn(usize, usize) -> Color;

#[allow(dead_code)]
pub struct Canvas2D<'a, T> where T: Draw {
    gpu: &'a mut T,
    pix_width: usize,
    pix_height: usize,
    background: Box<Gradient2D>,
    points: Vec<(Vector, Color)>,
    lines: Vec<(Line, Color)>
}

impl<'a, T> Canvas2D<'a, T> where T: Draw{
    pub fn new(gpu: &'a mut T, pix_width: usize, pix_height: usize) -> Self {
        Self {
            gpu,
            pix_width,
            pix_height,
            background: Box::new(|x, y| 0xffffff),
            points: Vec::new(),
            lines: Vec::new()
        }
    }

    pub fn draw(&mut self, origin: Vector, width: f64, height: f64) {
        let x_scale = self.pix_width as f64 / width; //1
        let y_scale = self.pix_height as f64 / height; //1

        let gradient = &self.background;

        //draw background
        for y in 0..self.pix_height {
            for x in 0..self.pix_width {
                self.gpu.draw(x, y, gradient(x, y));
            }
        }

        //draw points
        for &(point, color) in &self.points {
            let Vector (x, y) = point - origin;
            if (x >= 0.0 && y >= 0.0) && (x < width && y < height) {
                self.gpu.draw((x * x_scale) as usize, (y * y_scale) as usize, color);
            }
        }
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

