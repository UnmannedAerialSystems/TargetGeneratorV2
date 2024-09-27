mod backgrounds;
mod objects;
mod generator;

use image::{ImageBuffer, Rgb, RgbImage};
use simple_logger::SimpleLogger;
use log::debug;

fn main() {
    SimpleLogger::new().init().unwrap();
    debug!("Starting...");
    
}