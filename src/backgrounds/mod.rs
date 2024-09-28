use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use anyhow::{anyhow, Result};
use image::{DynamicImage, RgbaImage};
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
		let mut v = vec![]; // mutex for multi-thread access

		let start = Instant::now();

		if !dir.is_dir() {
			return Err(anyhow!("Path provided is not a directory!"));
		}

		debug!("Loading backgrounds from: {:?}", dir);

		// parallelize here because loading images can be slow
		fs::read_dir(dir)?.for_each(|entry| {
			let entry = entry.unwrap();
			let path = entry.path();

			if path.is_dir() { // skip if its another folder
				return;
			}
			
			let path_name = path.display().to_string();
			
			if let Ok(img) = image::open(path) {
				v.push(img);
			} else {
				warn!("Failed to load image: {}", path_name);
			}
			
			debug!("Loaded image in {}ms: {}", start.elapsed().as_millis(), path_name);
		});

		let mut r = vec![];
		
		for img in v.iter() {
			r.push(img.to_rgba8());
		}
		
		let x = Ok(Self { // don't ask why we need to make a variable here, it just works
			backgrounds: r
		});

		x
	}

	pub fn random(&self) -> Option<&RgbaImage> {
		self.backgrounds.choose(&mut thread_rng())
	}
}

#[test]
#[ignore]
fn test_bg_loader() {
	simple_logger::SimpleLogger::new().init().unwrap();

	let bg_loader = BackgroundLoader::new("backgrounds").unwrap();

	(0..10).par_bridge().for_each(|i| {
		/*let bg = bg_loader.random_augment(Some((-20, 20)), true).unwrap();
		bg.save(format!("output/output{}.png", i)).unwrap();*/
	});
}