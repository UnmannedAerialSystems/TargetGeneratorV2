use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use image::{DynamicImage};
use log::{warn};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use crate::generator::coco::{CocoCategory, CocoCategoryInfo};
use crate::generator::config::TargetGeneratorConfig;
use crate::generator::error::GenerationError;
use crate::generator::util;

#[derive(Debug)]
pub struct ObjectManager {
	path_buf: PathBuf,
	objects: Vec<Object>,
	object_set: HashSet<(u32, String)>
}

impl ObjectManager {
	pub fn new<P: AsRef<Path>>(path: P) -> ObjectManager {
		ObjectManager {
			path_buf: path.as_ref().to_path_buf(),
			objects: vec![],
			object_set: HashSet::new()
		}
	}
	
	/// Load training objects into the buffer
	pub fn load_objects(&mut self) -> Result<(), GenerationError> {
		let entries = std::fs::read_dir(&self.path_buf)?;
		
		// retrieve objects.json file that holds all info about our training objects
		let out = self.path_buf.join("objects.json");
		let file = std::fs::read_to_string(&out).ok().ok_or(GenerationError::MissingObjectsJSON)?;
		
		let object_details_file: ObjectDetailsFile = serde_json::from_str(&file)?;
		
		let mut id = 1;
		
		for entry in entries {
			let entry = entry?;
			let path = entry.path();
			
			if path.is_dir() || !util::is_image_type(&path.as_os_str().to_str().ok_or(GenerationError::GenericError("Failed to convert path to string".to_string()))?) {
				continue;
			}
			
			let file_name = if let Some(file_name) = path.file_name() {
				file_name.to_str().unwrap()
			} else {
				warn!("Failed to get file name for object: {}", path.display());
				continue;
			};
			
			let object_details = object_details_file.object_images.get(file_name);
			
			if object_details.is_none() {
				warn!("No object details found for object: {}", path.display());
				continue;
			}
			
			let object_details = object_details.unwrap();
			
			self.object_set.insert((object_details.object_type, file_name.to_string()));
			
			let dynamic_image = image::open(path)?;
			self.objects.push(Object {
				object_class: object_details.object_type,
				id,
				dynamic_image,
				object_width_meters: object_details.ground_width,
			});
			
			id += 1;
		}
		
		Ok(())
	}
	
	/// Generate a set of training objects a random that could be used to generate a target
	/// [amount] is the maximum number of objects to return
	/// Returns a set of objects that will contain no duplicates
	pub fn generate_set(&self, amount: u32, config: &TargetGeneratorConfig) -> Result<Vec<&Object>, GenerationError> {
		let mut rng = rand::thread_rng();
		let mut set = Vec::new();
		
		if !config.permit_duplicates {
			if amount > self.objects.len() as u32 {
				return Err(GenerationError::NotEnoughObjectsAvailable);
			}

			self.objects.choose_multiple(&mut rng, amount as usize).for_each(|object| {
				set.push(object);
			});
		} else {
			for _ in 0..amount {
				set.push(self.objects.choose(&mut rng).unwrap());
			}
		}
		
		Ok(set)
	}
}

impl CocoCategoryInfo for ObjectManager {
	fn categories(&self) -> Vec<CocoCategory> {
		let mut categories = vec![];
		
		for (object_type, name) in &self.object_set {
			categories.push(CocoCategory::new(*object_type, name.clone()));
		}
		
		categories
	}
}

#[derive(Debug, Clone)]
pub struct Object {
	pub(crate) object_class: u32,
	id: u16,
	pub(crate) dynamic_image: DynamicImage,
	pub(crate) object_width_meters: f32,
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool {
		self.object_class == other.object_class && self.id == other.id
	}
}

impl Eq for Object {}

impl std::hash::Hash for Object {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.object_class.hash(state);
		self.id.hash(state);
	}
}

/// Represents the object details file that holds all the information about the training objects
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectDetailsFile {
	object_images: HashMap<String, ObjectDetails>,
	object_types: HashMap<u32, ObjectType>
}

/// All details about a training object
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ObjectDetails {
	ground_width: f32,
	object_type: u32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ObjectType {
	name: String,
}

#[test]
fn ensure_sequential_no_duplicate_ids() {
	let mut object_manager = ObjectManager::new("objects");
	object_manager.load_objects().unwrap();
	
	let cats = object_manager.categories();
	
	let mut ids = vec![];
	
	for cat in cats {
		if ids.contains(&cat.id) {
			panic!("Duplicate ID found: {}", cat.id);
		}
		
		ids.push(cat.id);
	}
}

// Used to generate the starting object mapping file
#[ignore]
#[test]
fn generate_objects_json_file() {
	let mut object_images = HashMap::new();
	object_images.insert("bicycle_1.png".to_string(), ObjectDetails {
		ground_width: 1.73,
		object_type: 0
	});
	object_images.insert("bicycle_2.png".to_string(), ObjectDetails {
		ground_width: 1.73,
		object_type: 0
	});
	object_images.insert("tire_1.png".to_string(), ObjectDetails {
		ground_width: 1.0,
		object_type: 1
	});
	object_images.insert("tire_2.png".to_string(), ObjectDetails {
		ground_width: 1.0,
		object_type: 1
	});
	
	let mut object_types = HashMap::new();
	object_types.insert(0, ObjectType {
		name: "bicycle".to_string()
	});
	object_types.insert(1, ObjectType {
		name: "tire".to_string()
	});
	
	let object_details_file = ObjectDetailsFile {
		object_images,
		object_types
	};
	
	let json = serde_json::to_string_pretty(&object_details_file).unwrap();
	
	std::fs::write("objects/objects.json", json).unwrap();
}