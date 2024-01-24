use std::fs;
use std::path::Path;
use anyhow::{anyhow, Result};
use image::{ImageBuffer, Rgb, RgbImage};

pub struct BackgroundLoader {
    pub backgrounds: Option<Vec<RgbImage>>,
}

impl BackgroundLoader {
    pub fn load_backgrounds(&mut self, dir: &Path) -> Result<Vec<RgbImage>> {
        if self.backgrounds.is_some() {
            return Ok(self.backgrounds.clone().unwrap());
        }

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

        self.backgrounds = Some(v.clone()); // cached

        return Ok(v);
    }
}