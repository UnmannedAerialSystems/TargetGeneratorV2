use rayon::iter::ParallelIterator;
use std::ops::RangeTo;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgba, RgbaImage};
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
use crate::generator::coco::{BoundingBox, CocoCategoryInfo, CocoGenerator};
use crate::generator::config::TargetGeneratorConfig;
use crate::objects::{ObjectManager};
use moka::sync::{Cache, CacheBuilder};
use rayon::iter::{IntoParallelIterator, ParallelBridge};

pub mod coco;
pub mod error;
pub(crate) mod util;
pub mod config;

const COLLISION_ATTEMPTS: u32 = 15;

pub struct TargetGenerator {
	output: PathBuf,
	backgrounds_path: PathBuf,
	pub object_manager: ObjectManager,
	background_loader: BackgroundLoader,
	coco_generator: Arc<Mutex<CocoGenerator>>,
	config: TargetGeneratorConfig,
	resized_cache: Cache<String, DynamicImage>,
}

impl TargetGenerator {
	pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, objects_path: Q, annotations_path: Q) -> Result<Self, GenerationError> {

		let mut object_manager = ObjectManager::new(objects_path);
		object_manager.load_objects()?;
		
		let categories = object_manager.categories();
		let config = TargetGeneratorConfig::default();
		
		let resized_cache: Cache<String, DynamicImage> = CacheBuilder::new(config.cache_size as u64 * 1024 * 1024)
			.weigher(|_key, value: &DynamicImage| -> u32 { // evict based on size in MBs
				value.as_bytes().len() as u32
			})
			.build();
		
		Ok(Self {
			output: output.as_ref().to_path_buf(),
			backgrounds_path: background_path.as_ref().to_path_buf(),
			object_manager,
			background_loader: BackgroundLoader::new(background_path)?,
			coco_generator: Arc::new(Mutex::new(CocoGenerator::new(annotations_path, categories))),
			config,
			resized_cache,
		})
	}

	pub fn generate_target(&self, pixels_per_meter: f32, number_of_objects: u16) -> Result<RgbaImage, GenerationError> {
		trace!("Beginning to generate a target...");
		
		if number_of_objects == 0 {
			return Err(GenerationError::NoObjects);
		}

		let background = self.background_loader.random().unwrap();
		let mut image = background.image.clone();
		let (w, h) = (image.width(), image.height());
		let set = self.object_manager.generate_set(number_of_objects as u32, &self.config)?;
		let mut placed_objects = vec![];
		
		// add background image to coco here
		let background_id = self.coco_generator.lock().unwrap().add_image(w, h, background.filename.clone(), background.date_captured.clone());
		
		for obj in set {
			let clone = &obj.dynamic_image.clone();
			let (obj_w, obj_h) = (obj.dynamic_image.width(), obj.dynamic_image.height());
			let (x, y) = self.generate_new_location_no_collision((w, h), (obj_w, obj_h), &placed_objects)?;
			trace!("Placing object at {}, {}", x, y);
			
			let (obj_w, obj_h) = util::new_sizes(obj_w, obj_h, pixels_per_meter, obj.object_width_meters)?;
			debug!("Resizing object to {}x{}", obj_w, obj_h);
			
			// overlay respects transparent pixels unlike copy_from
			let resized = if let Some(resized) = self.resized_cache.get(&format!("{}x{}_{}", obj_w, obj_h, obj.object_class)) {
				resized.clone()
			} else {
				let resized = clone.resize(obj_w, obj_h, FilterType::Gaussian);
				self.resized_cache.insert(format!("{}x{}_{}", obj_w, obj_h, obj.object_class), resized.clone());
				resized
			};
			
			image::imageops::overlay(&mut image, &resized, x as i64, y as i64);
			
			if self.config.visualize_bboxes {
				imageproc::drawing::draw_hollow_rect_mut(&mut image, imageproc::rect::Rect::at(x as i32, y as i32).of_size(obj_w, obj_h), Rgba([0, 255, 0, 255]));
			}
			
			if let Some(color) = self.config.maskover_color {
				imageproc::drawing::draw_filled_rect_mut(&mut image, imageproc::rect::Rect::at(x as i32, y as i32).of_size(obj_w, obj_h), color);
			}
			
			let bbox = BoundingBox {
				x,
				y,
				width: obj_w,
				height: obj_h,
			};
			
			// add annotation to coco here
			self.coco_generator.lock().unwrap().add_annotation(background_id, obj.object_class, 0, vec![], (obj_w * obj_h) as f64, bbox);
			
			placed_objects.push(bbox);
		}

		Ok(image)
	}

	pub fn generate_targets<A: AsRef<Path> + Sync>(&mut self, amount: u32, range_to: RangeTo<u32>, path: A) -> Result<(), GenerationError> {
		let start = Instant::now(); // start timer
		debug!("Generating {} targets...", amount);

		let threadpool = rayon::ThreadPoolBuilder::new().num_threads(self.config.worker_threads as usize).build().unwrap();
		
		threadpool.install(|| {
			(0..amount).into_par_iter().for_each(|i| {
				let b = self.generate_target(STANDARD_PPM, thread_rng().gen_range(1..range_to.end) as u16).unwrap();
				let path = path.as_ref().join(format!("{}.png", i));
				b.save(path.clone()).unwrap();
				debug!("Saved generated target to {}", path.display());
			});
		});


		debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

		Ok(())
	}
	
	pub fn generate_new_location_no_collision(&self, bg_dimensions: (u32, u32), obj_dimensions: (u32, u32), placed_objects: &Vec<BoundingBox>) -> Result<(u32, u32), GenerationError> {
		let mut i = 0;
		
		loop {
			if i >= COLLISION_ATTEMPTS {
				return Err(GenerationError::TooManyCollisions)
			}
			
			let x = thread_rng().gen_range(0..bg_dimensions.0);
			let y = thread_rng().gen_range(0..bg_dimensions.1);

			if self.config.permit_collisions {
				return Ok((x, y));
			}

			let bbox = BoundingBox {
				x,
				y,
				width: obj_dimensions.0,
				height: obj_dimensions.1,
			};
			
			if placed_objects.iter().all(|placed| !placed.collides_with(&bbox)) {
				return Ok((x, y));
			}

			i += 1;
		}
	}
	
	pub fn close(&self) {
		self.coco_generator.lock().unwrap().save();
	}
}

#[test]
#[ignore]
pub fn test_generate_target() {
	SimpleLogger::new().init().unwrap();

	let mut tg = TargetGenerator::new("output", "backgrounds", "objects", "output/annotations.json").unwrap();
	tg.config.permit_duplicates = true;
	tg.config.permit_collisions = true;
	let b = tg.generate_target(STANDARD_PPM, 200).unwrap();

	b.save("output_1.png".to_string()).unwrap();
	debug!("Saved generated target to output_1.png");
	
	tg.close();
}

#[test]
#[ignore]
pub fn test_generate_targets() {
	SimpleLogger::new().init().unwrap();

	let mut tg = TargetGenerator::new("output", "backgrounds", "objects", "output/annotations.json").unwrap();
	tg.config.permit_duplicates = true;
	tg.config.permit_collisions = false;
	tg.config.visualize_bboxes = true;
	tg.generate_targets(500, ..6u32, "output").unwrap();
	
	tg.close();
}