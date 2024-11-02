use std::ops::RangeTo;
use std::path::{Path, PathBuf};
use std::time::Instant;
use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::drawing::draw_text_mut;
use log::{debug, trace};
use rand::{thread_rng, Rng};
use simple_logger::SimpleLogger;
use strum::Display;
use thiserror::Error;
use error::GenerationError;
use util::STANDARD_PPM;
use crate::backgrounds::BackgroundLoader;
use crate::generator::coco::{CocoCategoryInfo, CocoGenerator};
use crate::generator::config::TargetGeneratorConfig;
use crate::objects::{ObjectClass, ObjectManager, PlacedObject};

pub mod coco;
pub mod error;
pub(crate) mod util;
pub mod config;

pub struct TargetGenerator {
	output: PathBuf,
	backgrounds_path: PathBuf,
	pub object_manager: ObjectManager,
	background_loader: BackgroundLoader,
	coco_generator: CocoGenerator,
	config: TargetGeneratorConfig,
}

impl TargetGenerator {
	pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, objects_path: Q, annotations_path: Q) -> Result<Self, GenerationError> {

		let mut object_manager = ObjectManager::new(objects_path);
		object_manager.load_objects()?;
		
		Ok(Self {
			output: output.as_ref().to_path_buf(),
			backgrounds_path: background_path.as_ref().to_path_buf(),
			object_manager,
			background_loader: BackgroundLoader::new(background_path)?,
			coco_generator: CocoGenerator::new(annotations_path, ObjectClass::categories()),
			config: TargetGeneratorConfig::default(),
		})
	}

	pub fn generate_target(&mut self, pixels_per_meter: f32, number_of_objects: u16) -> Result<RgbaImage, GenerationError> {
		trace!("Beginning to generate a target...");
		
		if number_of_objects == 0 {
			return Err(GenerationError::NoObjects);
		}

		let background = self.background_loader.random().unwrap();
		let mut image = background.image.clone();
		let (w, h) = (image.width(), image.height());
		let set = self.object_manager.generate_set(number_of_objects as u32)?;
		let mut placed_objects = vec![];
		
		// add background image to coco here
		let background_id = self.coco_generator.add_image(w, h, background.filename.clone(), background.date_captured.clone());
		
		for obj in set {
			let clone = &obj.dynamic_image.clone();
			let (obj_w, obj_h) = (obj.dynamic_image.width(), obj.dynamic_image.height());
			let (x, y) = (thread_rng().gen_range(0..w - obj_w), thread_rng().gen_range(0..h - obj_h));
			trace!("Placing object at {}, {}", x, y);
			
			let (obj_w, obj_h) = util::new_sizes(obj_w, obj_h, pixels_per_meter, obj.object_width_meters)?;
			debug!("Resizing object to {}x{}", obj_w, obj_h);
			
			// overlay respects transparent pixels unlike copy_from
			image::imageops::overlay(&mut image, &clone.resize(obj_w, obj_h, FilterType::Gaussian), x as i64, y as i64);
			
			if self.config.visualize_bboxes {
				imageproc::drawing::draw_hollow_rect_mut(&mut image, imageproc::rect::Rect::at(x as i32, y as i32).of_size(obj_w, obj_h), Rgba([0, 255, 0, 255]));
			}
			
			if let Some(color) = self.config.maskover_color {
				imageproc::drawing::draw_filled_rect_mut(&mut image, imageproc::rect::Rect::at(x as i32, y as i32).of_size(obj_w, obj_h), color);
			}
			
			let bbox = coco::BoundingBox {
				x,
				y,
				width: obj_w,
				height: obj_h,
			};
			
			// add annotation to coco here
			let object_id = self.coco_generator.add_annotation(background_id, obj.object_class as u32, 0, vec![], (obj_w * obj_h) as f64, bbox);
			
			placed_objects.push(PlacedObject {
				id: object_id,
				bounding_box: bbox,
			});
		}

		Ok(image)
	}

	pub fn generate_targets<A: AsRef<Path>>(&self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<(), GenerationError> {
		let start = Instant::now(); // start timer



		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
	
	pub fn generate_new_location_no_collision(&self) -> (u32, u32) {
		todo!() // TODO: use collision detection to find a new location
	}
	
	pub fn close(&self) {
		self.coco_generator.save();
	}
}

#[test]
#[ignore]
pub fn test_generate_target() {
	SimpleLogger::new().init().unwrap();

	let mut tg = TargetGenerator::new("output", "backgrounds", "objects", "output/annotations.json").unwrap();
	let b = tg.generate_target(STANDARD_PPM, 1).unwrap();

	b.save("output_1.png".to_string()).unwrap();
	debug!("Saved generated target to output_1.png");
	
	tg.close();
}