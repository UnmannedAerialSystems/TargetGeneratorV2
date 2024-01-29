use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use anyhow::{anyhow, Result};
use image::{Pixel, RgbaImage, RgbImage};
use log::{debug, trace, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;

pub struct ShapeManager {
    shapes: Vec<Shape>,
}

impl ShapeManager {
    pub fn new(path: &Path) -> Result<Self> {
        let v = Mutex::new(vec![]);
        let start = Instant::now();

        if !path.is_dir() {
            return Err(anyhow!("Path provided is not a directory!"));
        }

        debug!("Loading shapes from: {:?}", path);

        fs::read_dir(path)?.par_bridge().for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() { // skip if its another folder
                return;
            }

            if let Some(s) = entry.file_name().to_str() {
                if let Some((s1, s2)) = s.split_once("_") {
                    if let Ok(img) = image::io::Reader::open(path.clone()) {
                        if let Ok(format) = img.with_guessed_format() {
                            if let Ok(img) = format.decode() {
                                trace!("Loaded shape: {:?} at {}ms", path, Instant::now().duration_since(start).as_millis());
                                let img = img.to_rgba8();

                                match s1 {
                                    "CIRCLE" => {v.lock().unwrap().push(Shape::CIRCLE(img))},
                                    "SEMICIRCLE" => {v.lock().unwrap().push(Shape::SEMICIRCLE(img))},
                                    "QUARTERCIRCLE" => {v.lock().unwrap().push(Shape::QUARTERCIRCLE(img))},
                                    "TRIANGLE" => {v.lock().unwrap().push(Shape::TRIANGLE(img))},
                                    _ => {warn!("Unknown shape type: \"{}\" for file \"{}\"", s1, s)}
                                }
                            } else {
                                warn!("Could not decode shape: {:?}", path);
                            }
                        } else {
                            warn!("Could not guess shape format: {:?}", path);
                        }
                    } else {
                        warn!("Could not open shape: {:?}", path);
                    }
                } else {
                    warn!("Shape file name is not marked with an identifier: \"{}\"", s);
                }
            } else {
                warn!("Could not parse Shape file name. Skipping: \"{:?}\"", entry.file_name());
            }
        });

        let x = Ok(Self {
            shapes: v.lock().unwrap().to_vec()
        });

        x
    }

    pub fn random(&self) -> Option<&Shape> {
        self.shapes.choose(&mut thread_rng())
    }
}

/// Valid shapes for the standard object include: circle, semicircle, quarter circle, triangle
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Shape {
    CIRCLE(RgbaImage),
    SEMICIRCLE(RgbaImage),
    QUARTERCIRCLE(RgbaImage),
    TRIANGLE(RgbaImage),
}

#[test]
pub fn load_shapes() {
    let shapes = ShapeManager::new(Path::new("shapes")).unwrap();

    if let Some(shape) = shapes.random() {
        match shape {
            Shape::CIRCLE(c) => {
                let mut c = c.clone();

                println!("Ran");

                for i in 0..c.width() {
                    for j in 0..c.height() {
                        let p = c.get_pixel(i, j);

                        if p.0[0] > 100 || p.0[1] > 100 || p.0[2] > 100 { // test auto-generating varying shape colors
                            c.put_pixel(i, j, image::Rgba([255, 255, 255, 255]));
                        }
                    }
                }

                c.save("output.png").unwrap()
            }
            Shape::SEMICIRCLE(_) => {}
            Shape::QUARTERCIRCLE(_) => {}
            Shape::TRIANGLE(_) => {}
        }
    } else {
        println!("No shapes found!");
    }
}