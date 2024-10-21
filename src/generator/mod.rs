use std::ops::RangeTo;
use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;
use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::drawing::draw_text_mut;
use log::{debug, trace};
use rand::{Rng, thread_rng};
use simple_logger::SimpleLogger;
use crate::backgrounds::BackgroundLoader;
use crate::objects::ObjectManager;

pub mod coco;

pub struct TargetGenerator {
	output: PathBuf,
	backgrounds_path: PathBuf,
	pub object_manager: ObjectManager,
	background_loader: BackgroundLoader,
}

impl TargetGenerator {
	pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, objects_path: Q) -> Result<Self> {

		let mut object_manager = ObjectManager::new(objects_path);
		object_manager.load_objects()?;
		
		Ok(Self {
			output: output.as_ref().to_path_buf(),
			backgrounds_path: background_path.as_ref().to_path_buf(),
			object_manager,
			background_loader: BackgroundLoader::new(background_path)?,
		})
	}

	pub fn generate_target(&self, altitude: f32, fov: f32, iteration: u32) -> Result<()> {
		trace!("Beginning to generate a target...");

		let mut background = self.background_loader.random().unwrap().clone();
		let (w, h) = (background.width(), background.height());
		let set = self.object_manager.generate_set(1)?;
		
		for obj in set {
			let clone = &obj.dynamic_image.clone();
			let (obj_w, obj_h) = (obj.dynamic_image.width(), obj.dynamic_image.height());
			let (x, y) = (thread_rng().gen_range(0..w - obj_w), thread_rng().gen_range(0..h - obj_h));
			trace!("Placing object at {}, {}", x, y);
			
			let adjusted = resize_ratio(obj.object_width_meters, METER_CONST);
			debug!("Width: {}, Height: {}", obj_w, obj_h);
			let aspect_ratio = obj_w as f32 / obj_h as f32;
			let (obj_w, obj_h) = (adjusted as u32, (adjusted / aspect_ratio) as u32);

			debug!("Resizing object to {}x{}", obj_w, obj_h); //TODO: too big
			
			// overlay respects transparent pixels unlike copy_from
			image::imageops::overlay(&mut background, &clone.resize(obj_w, obj_h, FilterType::Gaussian), x as i64, y as i64);
		}
		
		background.save(format!("output_{iteration}.png"))?;
		debug!("Saved generated target to output_{iteration}.png");

		Ok(())
	}

	pub fn generate_targets<A: AsRef<Path>>(&self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<()> {
		let start = Instant::now(); // start timer



		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundingBox {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}

// TODO: make this adjustable
const METER_CONST: f32 = 35.0;

fn resize_ratio(object_real_size: f32, pixels_per_meter: f32) -> f32 {
	debug!("Real size: {}, Pixels per meter: {}", object_real_size, pixels_per_meter);
	object_real_size * pixels_per_meter
}

fn degrees_to_radians(degrees: f32) -> f32 {
	degrees * std::f32::consts::PI / 180.0
}

// Calculate the fov in radians based on the image width, height and focal length
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

	let tg = TargetGenerator::new("output", "backgrounds", "objects").unwrap();
	tg.generate_target( 22.8, degrees_to_radians(35.0), 1).unwrap();
}