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
use crate::generator::coco::CocoGenerator;
use crate::objects::ObjectManager;

pub mod coco;

pub struct TargetGenerator {
	output: PathBuf,
	backgrounds_path: PathBuf,
	pub object_manager: ObjectManager,
	background_loader: BackgroundLoader,
	coco_generator: CocoGenerator
}

impl TargetGenerator {
	pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, objects_path: Q, annotations_path: Q) -> Result<Self> {

		let mut object_manager = ObjectManager::new(objects_path);
		object_manager.load_objects()?;
		
		Ok(Self {
			output: output.as_ref().to_path_buf(),
			backgrounds_path: background_path.as_ref().to_path_buf(),
			object_manager,
			background_loader: BackgroundLoader::new(background_path)?,
			coco_generator: CocoGenerator::new(annotations_path)
		})
	}

	pub fn generate_target(&self, altitude: f32, fov: f32, iteration: u32, pixels_per_meter: f32) -> Result<RgbaImage> {
		trace!("Beginning to generate a target...");

		let mut background = self.background_loader.random().unwrap().clone();
		let (w, h) = (background.width(), background.height());
		let set = self.object_manager.generate_set(1)?;
		
		for obj in set {
			let clone = &obj.dynamic_image.clone();
			let (obj_w, obj_h) = (obj.dynamic_image.width(), obj.dynamic_image.height());
			let (x, y) = (thread_rng().gen_range(0..w - obj_w), thread_rng().gen_range(0..h - obj_h));
			trace!("Placing object at {}, {}", x, y);
			
			let (obj_w, obj_h) = new_sizes(obj_w, obj_h, pixels_per_meter, obj.object_width_meters);
			debug!("Resizing object to {}x{}", obj_w, obj_h);
			
			// overlay respects transparent pixels unlike copy_from
			image::imageops::overlay(&mut background, &clone.resize(obj_w, obj_h, FilterType::Gaussian), x as i64, y as i64);
		}

		Ok(background)
	}

	pub fn generate_targets<A: AsRef<Path>>(&self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<()> {
		let start = Instant::now(); // start timer



		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
}

const STANDARD_PPM: f32 = 35.0;

/// Use the real size of an object and the Pixels Per Meter value to calculate the size in 
/// pixels that it should be in order to be at scale
fn resize_ratio(object_real_size: f32, pixels_per_meter: f32) -> f32 {
	debug!("Real size: {}, Pixels per meter: {}", object_real_size, pixels_per_meter);
	object_real_size * pixels_per_meter
}

/// Calculate the new sizes of an object in pixels based on the requested Pixel Per Meter value
/// 1. Calculate the aspect ratio
/// 2. Calculate the width of the object in pixels that we expect based on the real width and the Pixels Per Meter value
/// 3. Calculate the height from this new width using the previously calculated aspect ratio
fn new_sizes(object_width: u32, object_height: u32, pixels_per_meter: f32, real_width: f32) -> (u32, u32) {
	let (w, h) = (object_width as f32, object_height as f32);
	let aspect_ratio = w / h;
	let new_width = resize_ratio(real_width, pixels_per_meter) as u32;
	let new_height = (new_width as f32 / aspect_ratio) as u32;
	
	(new_width, new_height)
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

	let tg = TargetGenerator::new("output", "backgrounds", "objects", "output").unwrap();
	let b = tg.generate_target(22.8, degrees_to_radians(35.0), 1, STANDARD_PPM).unwrap();

	b.save("output_1.png".to_string()).unwrap();
	debug!("Saved generated target to output_1.png");
}