use std::fs;
use std::path::Path;
use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgb, RgbImage};

pub struct BackgroundLoader {
    pub backgrounds: Vec<RgbImage>,
}

impl BackgroundLoader {
    pub fn new(dir: &Path) -> Result<BackgroundLoader> {
        let mut v = vec![];

        if !dir.is_dir() {
            return Err(anyhow!("Path provided is not a directory!"));
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() { // skip if its another folder
                continue;
            }

            let img = image::open(path)?;
            let img = img.to_rgb8();
            v.push(img);
        }

        Ok(Self {
            backgrounds: v
        })
    }
}