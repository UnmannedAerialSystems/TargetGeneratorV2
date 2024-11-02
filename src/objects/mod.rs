use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use image::{DynamicImage, RgbaImage};
use log::warn;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use crate::generator::coco::{BoundingBox, CocoCategory, CocoCategoryInfo};
use crate::generator::error::GenerationError;

#[derive(Debug)]
pub struct ObjectManager {
	path_buf: PathBuf,
	objects: Vec<Object>,
}

impl ObjectManager {
	pub fn new<P: AsRef<Path>>(path: P) -> ObjectManager {
		ObjectManager {
			path_buf: path.as_ref().to_path_buf(),
			objects: vec![],
		}
	}
	
	/// Load training objects into the buffer
	pub fn load_objects(&mut self) -> Result<(), GenerationError> {
		let mut entries = std::fs::read_dir(&self.path_buf)?;
		
		// retrieve objects.json file that holds all info about our training objects
		let out = entries.find(|entry| {
			let entry = entry.as_ref().unwrap().path();
			
			entry.file_name().unwrap().to_str().unwrap() == "objects.json"
		}).ok_or(GenerationError::MissingObjectsJSON)??;
		
		let object_details_file: ObjectDetailsFile = serde_json::from_str(&std::fs::read_to_string(out.path())?)?;
		
		for entry in entries {
			let entry = entry?;
			let path = entry.path();
			
			let mut name_parts = path.file_stem()
				.ok_or(GenerationError::GenericError("Couldn't extract filename!".to_string()))?
				.to_str().unwrap()
				.split('_');
			
			let prefix = name_parts.next().unwrap();
			let id = name_parts.next().unwrap().parse::<u16>()?;
			
			let object_details = object_details_file.map.get(&format!("{}_{}", prefix, id));
			
			if object_details.is_none() {
				warn!("No object details found for object: {}", path.display());
				continue;
			}
			
			let object_details = object_details.unwrap();
			
			let object_class = ObjectClass::from_prefix(prefix);
			let dynamic_image = image::open(path)?;
			self.objects.push(Object {
				object_class,
				id,
				dynamic_image,
				object_width_meters: object_details.ground_width,
			});
		}
		
		Ok(())
	}
	
	/// Generate a set of training objects a random that could be used to generate a target
	/// [amount] is the maximum number of objects to return
	/// Returns a set of objects that will contain no duplicates
	pub fn generate_set(&self, amount: u32) -> Result<HashSet<&Object>, GenerationError> {
		let mut rng = rand::thread_rng();
		let mut set = HashSet::new();
		
		if amount > self.objects.len() as u32 {
			return Err(GenerationError::NotEnoughObjectsAvailable);
		}
		
		self.objects.choose_multiple(&mut rng, amount as usize).for_each(|object| {
			set.insert(object);
		});
		
		Ok(set)
	}
}

#[derive(Debug, Clone)]
pub struct Object {
	pub(crate) object_class: ObjectClass,
	id: u16,
	pub(crate) dynamic_image: DynamicImage,
	pub(crate) object_width_meters: f32,
}

impl Object {
	pub fn to_rgba_image(&self) -> RgbaImage {
		self.dynamic_image.to_rgba8()
	}
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum ObjectClass {
	BICYCLE,
	TIRE
}

impl ObjectClass {
	pub fn from_prefix(prefix: &str) -> ObjectClass {
		match prefix {
			"bicycle" => ObjectClass::BICYCLE,
			"tire" => ObjectClass::TIRE,
			_ => panic!("Unknown object class: {}", prefix),
		}
	}
	
	pub fn prefix(&self) -> &str {
		match self {
			ObjectClass::BICYCLE => "bicycle",
			ObjectClass::TIRE => "tire",
		}
	}
	
	/// Gets the numerical id of the object class for COCO format category id.
	/// NOTE: object ids should be sequential and non-duplicative
	pub fn id(&self) -> u32 {
		match self {
			ObjectClass::BICYCLE => 0,
			ObjectClass::TIRE => 1,
		}
	}
}

impl CocoCategoryInfo for ObjectClass {
	fn categories() -> Vec<CocoCategory> {
		let mut v = vec![];
		
		for object_class in ObjectClass::iter() {
			v.push(CocoCategory::new(object_class.id(), object_class.prefix().to_string()));
		}
		
		v
	}
}

/// Represents the object details file that holds all the information about the training objects
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectDetailsFile {
	map: HashMap<String, ObjectDetails>
}

/// All details about a training object
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ObjectDetails {
	ground_width: f32
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct PlacedObject {
	pub(crate) id: u32,
	pub(crate) bounding_box: BoundingBox
}

impl PlacedObject {
	pub fn collides_with(&self, other: &PlacedObject) -> bool {
		if self.id == other.id {
			return false;
		}
		
		let (x1, y1, w1, h1) = (self.bounding_box.x, self.bounding_box.y, self.bounding_box.width, self.bounding_box.height);
		let (x2, y2, w2, h2) = (other.bounding_box.x, other.bounding_box.y, other.bounding_box.width, other.bounding_box.height);
		
		x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
	}
}

#[test]
fn ensure_sequential_no_duplicate_ids() {
	let mut id = 0;

	for object_class in ObjectClass::iter() {
		assert_eq!(object_class.id(), id, "Object class id is not sequential for {:?}", object_class);
		id += 1;
	}
}

#[test]
pub fn collision_detection_test() {
	let obj1 = PlacedObject {
		id: 0,
		bounding_box: BoundingBox {
			x: 0,
			y: 0,
			width: 10,
			height: 10
		}
	};
	
	let obj2 = PlacedObject {
		id: 1,
		bounding_box: BoundingBox {
			x: 5,
			y: 5,
			width: 10,
			height: 10
		}
	};
	
	let obj3 = PlacedObject {
		id: 2,
		bounding_box: BoundingBox {
			x: 20,
			y: 20,
			width: 10,
			height: 10
		}
	};
	
	assert!(obj1.collides_with(&obj2));
	assert!(!obj1.collides_with(&obj3));
	assert!(!obj2.collides_with(&obj3));
	assert!(!obj2.collides_with(&obj2));
}

// Used to generate the starting object mapping file
#[ignore]
#[test]
fn generate_objects_json_file() {
	let mut map = HashMap::new();
	map.insert("bicycle_1".to_string(), ObjectDetails {
		ground_width: 1.73
	});
	map.insert("bicycle_2".to_string(), ObjectDetails {
		ground_width: 1.73
	});
	map.insert("tire_1".to_string(), ObjectDetails {
		ground_width: 1.0
	});
	map.insert("tire_2".to_string(), ObjectDetails {
		ground_width: 1.0
	});
	
	let object_details_file = ObjectDetailsFile {
		map
	};
	
	let json = serde_json::to_string_pretty(&object_details_file).unwrap();
	
	std::fs::write("objects/objects.json", json).unwrap();
}