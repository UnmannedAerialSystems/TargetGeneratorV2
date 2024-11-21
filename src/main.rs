#![allow(dead_code)] // allows unused struct members

mod backgrounds;
mod objects;
mod generator;

use simple_logger::SimpleLogger;
use log::{debug, LevelFilter};
use crate::generator::TargetGenerator;

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
    debug!("Starting...");
    
    let mut tg = TargetGenerator::new("output", "backgrounds", "objects", "output/annotations.json").unwrap();
    tg.config.permit_duplicates = true;
    tg.config.permit_collisions = false;
    //tg.config.visualize_bboxes = true;
    tg.generate_targets(500, ..6u32, "output").unwrap();

    tg.close();
}