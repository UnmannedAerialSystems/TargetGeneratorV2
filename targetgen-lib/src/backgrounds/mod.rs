use rayon::iter::ParallelIterator;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use chrono::{DateTime, Local};
use image::{RgbaImage};
use log::{debug, warn};
use rand::seq::SliceRandom;
use rand::{thread_rng};
use rayon::iter::ParallelBridge;
use crate::generator::error::GenerationError;

pub struct BackgroundLoader {
	pub backgrounds: Arc<Mutex<Vec<BackgroundImage>>>,
}

impl BackgroundLoader {
	pub fn new<Q: AsRef<Path>>(path: Q) -> Result<BackgroundLoader, GenerationError> {
		let dir = path.as_ref().to_path_buf();
		let v = Arc::new(Mutex::new(vec![])); // mutex for multi-thread access

		let start = Instant::now();

		if !dir.is_dir() {
			return Err(GenerationError::NotADirectory);
		}

		debug!("Loading backgrounds from: {:?}", dir);

		fs::read_dir(dir)?.par_bridge().for_each(|entry | {
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
					id: v.lock().unwrap().len() as u32,
				};
				
				v.lock().unwrap().push(back);
			} else {
				warn!("Failed to load image: {}", path_name);
			}
			
			debug!("Loaded image in {}ms: {}", start.elapsed().as_millis(), path_name.to_string().replace("\\", "/"));
		});
		
		let x = Ok(Self { // don't ask why we need to make a variable here, it just works
			backgrounds: v
		});

		x
	}

	pub fn random(&self) -> Option<BackgroundImage> {
		let lock = self.backgrounds.lock().unwrap();
		
		if let Some(bg) = lock.choose(&mut thread_rng()) {
			Some(bg.clone())
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub struct BackgroundImage {
	pub image: RgbaImage,
	pub filename: String,
	pub date_captured: String,
	pub id: u32,
}