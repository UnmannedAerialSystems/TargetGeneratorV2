use std::ops::RangeTo;
use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use log::{debug, trace};
use rand::{Rng, thread_rng};
use simple_logger::SimpleLogger;
use crate::backgrounds::BackgroundLoader;
use crate::objects::ObjectManager;

pub struct TargetGenerator {
	output: PathBuf,
	backgrounds_path: PathBuf,
	pub shape_manager: ObjectManager,
	background_loader: BackgroundLoader,
}

impl TargetGenerator {
	pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, objects_path: Q) -> Result<Self> {

		Ok(Self {
			output: output.as_ref().to_path_buf(),
			backgrounds_path: background_path.as_ref().to_path_buf(),
			shape_manager: ObjectManager::new(objects_path),
			background_loader: BackgroundLoader::new(background_path)?,
		})
	}

	pub fn generate_target(&self, altitude: f32) -> Result<()> {

		debug!("Beginning to generate a target...");

		let mut background = self.background_loader.random().unwrap().clone();
		let (w, h) = (background.width(), background.height());





		background.save("output.png")?;

		Ok(())
	}

	pub fn generate_targets<A: AsRef<Path>>(&self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<()> {
		let start = Instant::now(); // start timer



		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
}

fn calculate_fov(image_width: u32, image_height: u32, focal_length: f32) -> f32 {
	2.0 * (0.5 * image_width as f32 / focal_length).atan()
}

/// Calculate the width of the ground in meters based on camera position and field of view
fn calculate_ground_width(altitude: f32, fov: f32) -> f32 {
	2.0 * altitude * (fov.to_radians() / 2.0).tan()
}

fn meters_per_pixel(image_width: u32, ground_width: f32) -> f32 {
	ground_width / image_width as f32
}

/// This gives you the expected size of an object in pixels, use this to calculate a ratio between
/// the expected and actual values. Then use that to scale up/down the object to the expected size
fn expected_object_size_pixels(real_size: f32, meters_per_pixel: f32) -> f32 {
	real_size / meters_per_pixel
}

#[test]
#[ignore]
pub fn test_generate_target() {
	SimpleLogger::new().init().unwrap();

	let tg = TargetGenerator::new("output", "backgrounds", "shapes").unwrap();
	tg.generate_target( 22.8).unwrap();
}