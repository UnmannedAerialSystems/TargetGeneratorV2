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
use strum::Display;
use thiserror::Error;
use crate::backgrounds::BackgroundLoader;
use crate::generator::coco::{CocoCategoryInfo, CocoGenerator};
use crate::objects::{ObjectClass, ObjectManager};

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
			coco_generator: CocoGenerator::new(annotations_path, ObjectClass::categories())
		})
	}

	pub fn generate_target(&mut self, pixels_per_meter: f32) -> Result<RgbaImage> {
		trace!("Beginning to generate a target...");

		let background = self.background_loader.random().unwrap();
		let mut image = background.image.clone();
		let (w, h) = (image.width(), image.height());
		let set = self.object_manager.generate_set(1)?;
		
		// add background image to coco here
		self.coco_generator.add_image(w, h, background.filename.clone(), background.date_captured.clone());
		
		for obj in set {
			let clone = &obj.dynamic_image.clone();
			let (obj_w, obj_h) = (obj.dynamic_image.width(), obj.dynamic_image.height());
			let (x, y) = (thread_rng().gen_range(0..w - obj_w), thread_rng().gen_range(0..h - obj_h));
			trace!("Placing object at {}, {}", x, y);
			
			let (obj_w, obj_h) = new_sizes(obj_w, obj_h, pixels_per_meter, obj.object_width_meters)?;
			debug!("Resizing object to {}x{}", obj_w, obj_h);
			
			// overlay respects transparent pixels unlike copy_from
			image::imageops::overlay(&mut image, &clone.resize(obj_w, obj_h, FilterType::Gaussian), x as i64, y as i64);
			
			// TODO: remove, both are top left
			//imageproc::drawing::draw_filled_circle_mut(&mut image, (x as i32, y as i32), 4, Rgba([255, 0, 0, 255]));
			//imageproc::drawing::draw_filled_circle_mut(&mut image, (0i32, 0i32), 8, Rgba([255, 0, 255, 255]));
			imageproc::drawing::draw_hollow_rect_mut(&mut image, imageproc::rect::Rect::at(x as i32, y as i32).of_size(obj_w, obj_h), Rgba([0, 255, 0, 255]));
			
			// add annotation to coco here
			self.coco_generator.add_annotation(0, obj.object_class as u32, 0, vec![], (obj_w * obj_h) as f64, coco::BoundingBox {
				x,
				y,
				width: obj_w,
				height: obj_h,
			});
		}

		Ok(image)
	}

	pub fn generate_targets<A: AsRef<Path>>(&self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<()> {
		let start = Instant::now(); // start timer



		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
	
	pub fn close(&self) {
		self.coco_generator.save();
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
fn new_sizes(object_width: u32, object_height: u32, pixels_per_meter: f32, real_width: f32) -> Result<(u32, u32), GenerationError> {
	let (w, h) = (object_width as f32, object_height as f32);
	let aspect_ratio = w / h;
	let new_width = resize_ratio(real_width, pixels_per_meter) as u32;
	let new_height = (new_width as f32 / aspect_ratio) as u32;
	
	if new_height == 0 || new_width == 0 {
		return Err(GenerationError::SizeError);
	}
	
	Ok((new_width, new_height))
}

#[derive(Debug, Error)]
pub enum GenerationError {
	#[error("Serde decoding or encoding error")]
	SerdeError(#[from] serde_json::Error),
	#[error("IO error occurred while generating target")]
	IOError(#[from] std::io::Error),
	#[error("Calculated new sizes provided an invalid size")]
	SizeError
}

#[test]
#[ignore]
pub fn test_generate_target() {
	SimpleLogger::new().init().unwrap();

	let mut tg = TargetGenerator::new("output", "backgrounds", "objects", "output/annotations.json").unwrap();
	let b = tg.generate_target(STANDARD_PPM).unwrap();

	b.save("output_1.png".to_string()).unwrap();
	debug!("Saved generated target to output_1.png");
	
	tg.close();
}