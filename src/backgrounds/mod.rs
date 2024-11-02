use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use chrono::{DateTime, Local};
use image::{DynamicImage, RgbaImage};
use log::{debug, trace, warn};
use rand::seq::SliceRandom;
use rand::{Rng, thread_rng};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use crate::generator::error::GenerationError;

pub struct BackgroundLoader {
	pub backgrounds: Vec<BackgroundImage>,
}

impl BackgroundLoader {
	pub fn new<Q: AsRef<Path>>(path: Q) -> Result<BackgroundLoader, GenerationError> {
		let dir = path.as_ref().to_path_buf();
		let mut v = vec![]; // mutex for multi-thread access

		let start = Instant::now();

		if !dir.is_dir() {
			return Err(GenerationError::NotADirectory);
		}

		debug!("Loading backgrounds from: {:?}", dir);

		// TODO: parallelize here because loading images can be slow
		fs::read_dir(dir)?.for_each(|entry | {
			let entry = entry.unwrap();
			let path = entry.path();

			if path.is_dir() { // skip if its another folder
				return;
			}
			
			let path_name = path.display().to_string();
			
			if let Ok(img) = image::open(path) {
				let datetime: DateTime<Local> = entry.metadata().unwrap().created().unwrap().into();
				
				let back = BackgroundImage {
					image: img.to_rgba8(),
					filename: path_name.clone(),
					date_captured: datetime.to_string(),
					id: v.len() as u32,
				};
				
				v.push(back);
			} else {
				warn!("Failed to load image: {}", path_name);
			}
			
			debug!("Loaded image in {}ms: {}", start.elapsed().as_millis(), path_name);
		});
		
		let x = Ok(Self { // don't ask why we need to make a variable here, it just works
			backgrounds: v
		});

		x
	}

	pub fn random(&self) -> Option<&BackgroundImage> {
		self.backgrounds.choose(&mut thread_rng())
	}
}

pub struct BackgroundImage {
	pub image: RgbaImage,
	pub filename: String,
	pub date_captured: String,
	pub id: u32,
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