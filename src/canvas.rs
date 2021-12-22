pub mod canvas2d;
pub mod vector;
pub mod line;
pub mod matrix;

pub trait Draw {
    fn draw(&mut self, x: usize, y: usize, color: u32);
}