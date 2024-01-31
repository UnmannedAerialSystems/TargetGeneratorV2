use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use anyhow::{anyhow, Result};
use image::RgbaImage;
use log::{debug, trace, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge};
use rayon::iter::ParallelIterator;
use simple_logger::SimpleLogger;

pub struct ShapeManager {
    shapes: Vec<Shape>,
    color_variants: Vec<Shape>
}

impl ShapeManager {
    pub fn new<Q: AsRef<Path>>(path: Q) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
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

        if let Ok(v) = v.lock() {
            if v.len() == 0 {
                warn!("No shapes were parsed! Are there any shape templates in the input?");
            }
        } else {
            return Err(anyhow!("Could not unlock parsed shape list."));
        }

        let x = Ok(Self {
            shapes: v.lock().unwrap().to_vec(),
            color_variants: vec![]
        });

        x
    }

    pub fn generate_color_variants(mut self, colors: Vec<ShapeColor>) -> Result<Self> {
        let vec = Mutex::new(vec![]);

        for s in &self.shapes {
            colors.par_iter().for_each(|x| {
                let s = s.get_color(x).unwrap();
                vec.lock().unwrap().push(s);
            });
        }

        self.color_variants = vec.lock().unwrap().to_vec();

        Ok(self)
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

impl Shape {
    pub fn get_color(&self, color: &ShapeColor) -> Result<Shape> {
        let image = self.get_inner_image();

        let mut image = image.clone();

        for i in 0..image.width() {
            for j in 0..image.height() {
                let p = image.get_pixel(i, j);

                // if alpha is greater than 10, set to whatever color
                if p.0[3] > 10 { // test auto-generating varying shape colors
                    image.put_pixel(i, j, image::Rgba(color.get_rgba()));
                }
            }
        }

        match self {
            Shape::CIRCLE(_) => {Ok(Shape::CIRCLE(image))}
            Shape::SEMICIRCLE(_) => {Ok(Shape::SEMICIRCLE(image))}
            Shape::QUARTERCIRCLE(_) => {Ok(Shape::QUARTERCIRCLE(image))}
            Shape::TRIANGLE(_) => {Ok(Shape::TRIANGLE(image))}
        }
    }

    pub fn get_inner_image(&self) -> &RgbaImage {
        match self {
            Shape::CIRCLE(c) => {c}
            Shape::SEMICIRCLE(c) => {c}
            Shape::QUARTERCIRCLE(c) => {c}
            Shape::TRIANGLE(c) => {c}
        }
    }

    pub fn get_center(&self) -> (u32, u32) {
        let x = self.get_inner_image().width() / 2;
        let y = self.get_inner_image().height() / 2;

        return (x, y);
    }
}

// white, black, red, blue, green, purple, brown, and orange
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ShapeColor {
    WHITE,
    RED,
    BLACK,
    BLUE,
    GREEN,
    PURPLE,
    BROWN,
    ORANGE,
}

impl ShapeColor {
    // TODO: test colors
    pub fn get_rgba(&self) -> [u8; 4] {
        match self {
            ShapeColor::WHITE => [255, 255, 255, 100],
            ShapeColor::RED => [255, 0, 0, 100],
            ShapeColor::BLACK => [0, 0, 0, 100],
            ShapeColor::BLUE => [0, 0, 255, 100],
            ShapeColor::GREEN => [0, 255, 0, 100],
            ShapeColor::PURPLE => [255, 0, 255, 100],
            ShapeColor::BROWN => [165, 42, 42, 100],
            ShapeColor::ORANGE => [255, 165, 0, 100],
        }
    }
}

#[test]
#[ignore]
pub fn load_shapes() {
    let shapes = ShapeManager::new("shapes").unwrap();

    if let Some(shape) = shapes.random() {
        match shape {
            Shape::CIRCLE(c) => {
                let mut c = c.clone();

                println!("Ran");

                for i in 0..c.width() {
                    for j in 0..c.height() {
                        let p = c.get_pixel(i, j);

                        // if alpha is greater than 10, set to whatever color
                        if p.0[3] > 10 { // test auto-generating varying shape colors
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

#[test]
#[ignore]
fn generate_colors() {
    SimpleLogger::new().init().unwrap();

    let shapes = ShapeManager::new("shapes").unwrap();
    let colors = vec![
        ShapeColor::WHITE,
        ShapeColor::RED,
        ShapeColor::BLACK,
        ShapeColor::BLUE,
        ShapeColor::GREEN,
        ShapeColor::PURPLE,
        ShapeColor::BROWN,
        ShapeColor::ORANGE,
    ];

    let shapes = shapes.generate_color_variants(colors).unwrap();

    if shapes.color_variants.len() > 0 {
        let shape = shapes.color_variants.choose(&mut thread_rng()).unwrap();

        shape.get_inner_image().save("output.png").unwrap();
    }
}