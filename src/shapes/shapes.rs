use std::fs;
use std::path::Path;
use anyhow::{anyhow, Result};
use image::RgbImage;
use log::warn;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct ShapeManager {
    shapes: Vec<Shape>,
}

impl ShapeManager {
    pub fn new(path: &Path) -> Result<Self> {
        let mut v: Vec<Shape> = vec![];

        if !path.is_dir() {
            return Err(anyhow!("Path provided is not a directory!"));
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() { // skip if its another folder
                continue;
            }

            if let Some(s) = entry.file_name().to_str() {
                if let Some((s1, s2)) = s.split_once("_") {
                    let img = image::open(path)?;
                    let img = img.to_rgb8();

                    match s1 {
                        "CIRCLE" => {v.push(Shape::CIRCLE(img))},
                        "SEMICIRCLE" => {v.push(Shape::SEMICIRCLE(img))},
                        "QUARTERCIRCLE" => {v.push(Shape::QUARTERCIRCLE(img))},
                        "TRIANGLE" => {v.push(Shape::TRIANGLE(img))},
                        _ => {warn!("Unknown shape type: \"{}\" for file \"{}\"", s1, s)}
                    }
                } else {
                    warn!("Shape file name is not marked with an identifier: \"{}\"", s);
                }
            } else {
                return Err(anyhow!("Could not parse Shape file name."));
            }
        }

        Ok(Self {
            shapes: v
        })
    }

    pub fn random(&self) -> Option<&Shape> {
        self.shapes.choose(&mut thread_rng())
    }
}

/// Valid shapes for the standard object include: circle, semicircle, quarter circle, triangle
pub enum Shape {
    CIRCLE(RgbImage),
    SEMICIRCLE(RgbImage),
    QUARTERCIRCLE(RgbImage),
    TRIANGLE(RgbImage),
}