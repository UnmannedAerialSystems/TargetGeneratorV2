#![allow(dead_code)] // allows unused struct members

mod backgrounds;
mod objects;
mod generator;

use simple_logger::SimpleLogger;
use log::debug;

fn main() {
    SimpleLogger::new().init().unwrap();
    debug!("Starting...");
    
}