use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use anyhow::{anyhow, Result};
use image::RgbaImage;
use log::{debug, trace, warn};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;

pub struct BackgroundLoader {
    pub backgrounds: Vec<RgbaImage>,
}

impl BackgroundLoader {
    pub fn new<Q: AsRef<Path>>(path: Q) -> Result<BackgroundLoader> {
        let dir = path.as_ref().to_path_buf();
        let v = Mutex::new(vec![]); // mutex for multi-thread access

        let start = Instant::now();

        if !dir.is_dir() {
            return Err(anyhow!("Path provided is not a directory!"));
        }

        debug!("Loading backgrounds from: {:?}", dir);

        // parallelize here because loading images can be slow
        fs::read_dir(dir)?.par_bridge().for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() { // skip if its another folder
                return;
            }

            // deduce file format from name
            if let Ok(img) = image::io::Reader::open(path.clone()) {
                if let Ok(format) = img.with_guessed_format() {
                    if let Ok(img) = format.decode() {
                        trace!("Loaded background: {:?} at {}ms", path, Instant::now().duration_since(start).as_millis());
                        let img = img.to_rgba8();
                        v.lock().unwrap().push(img);
                    } else {
                        warn!("Could not decode background: {:?}", path);
                    }
                } else {
                    warn!("Could not guess background format: {:?}", path);
                }
            } else {
                warn!("Could not open background: {:?}", path);
            }
        });

        let x = Ok(Self { // don't ask why we need to make a variable here, it just works
            backgrounds: v.lock().unwrap().to_vec()
        });

        x
    }

    pub fn random(&self) -> Option<&RgbaImage> {
        self.backgrounds.choose(&mut thread_rng())
    }

    pub fn random_augment(&self, brightness: Option<(i32, i32)>, do_rotate: bool) -> Option<RgbaImage> {
        let mut img = self.random()?.clone();

        // modify the image, rotate / brightness, etc
        if do_rotate {
            let r = thread_rng().gen_range(0..=3); // [0, 3] range

            img = match r {
                1 => image::imageops::rotate90(&img),
                2 => image::imageops::rotate180(&img),
                3 => image::imageops::rotate270(&img),
                _ => img // either when 0 or error
            };
        }

        if let Some(b) = brightness {
            let b = thread_rng().gen_range(b.0..=b.1); // [b.0, b.1] range

            img = image::imageops::brighten(&img, b);
        }

        return Some(img);
    }
}

#[test]
fn test_bg_loader() {
    simple_logger::SimpleLogger::new().init().unwrap();

    let bg_loader = BackgroundLoader::new("backgrounds").unwrap();
    /*let bg = bg_loader.random_augment(Some((-20, 20)), true).unwrap();
    bg.save("output.png").unwrap();*/

    (0..10).par_bridge().for_each(|i| {
        let bg = bg_loader.random_augment(Some((-20, 20)), true).unwrap();
        bg.save(format!("output/output{}.png", i)).unwrap();
    });
}