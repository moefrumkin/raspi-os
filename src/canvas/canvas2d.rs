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

    pub fn clear(&mut self) {
        let gradient = &self.background;
        for y in 0..self.pix_height {
            for x in 0..self.pix_width {
                self.gpu.draw(x, y, gradient(x, y));
            }
        }
    }

    pub fn draw(&mut self, origin: Vector, width: f64, height: f64) {
        let x_scale = self.pix_width as f64 / width; //1
        let y_scale = self.pix_height as f64 / height; //1

        let scale = Matrix ( Vector (x_scale, 0.0), Vector (0.0, y_scale));

        //draw points
        for &(point, color) in &self.points {
            let Vector (x, y) = scale * (point - origin);
            if (x >= 0.0 && y >= 0.0) && (x < width && y < height) {
                self.gpu.draw(x as usize, y as usize, color);
            }
        }

        //draw lines
        for &(Line (v0, v1), color) in &self.lines {
            let Vector (x0, y0) = scale * (v0 - origin);
            let Vector (x1, y1) = scale * (v1 - origin);

            let start_x: usize;
            let start_y: usize;
            let end_x: usize;
            let end_y: usize;

            if x1 < x0 {
                start_x = x1 as usize;
                start_y = y1 as usize;
                end_x = x0 as usize;
                end_y = y0 as usize;
            } else {
                start_x = x0 as usize;
                start_y = y0 as usize;
                end_x = x1 as usize;
                end_y = y1 as usize;
            }

            let dx = end_x - start_x;
            let dy = end_y - start_y;

            if dx > dy && end_y > start_y {
                let mut e = 0;
                let mut y = start_y;
                for x in start_x..end_x {
                    self.gpu.draw(x, y, color);
                    if 2 * (e + dy) < dx {
                        e += dy
                    } else {
                        y += 1;
                        e += dy - dx;
                    }
                }
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

