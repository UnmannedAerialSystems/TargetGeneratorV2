use std::collections::HashSet;
use std::path::{Path, PathBuf};
use image::DynamicImage;
use anyhow::{anyhow, Result};
use rand::prelude::SliceRandom;

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
	
	pub fn load_objects(&mut self) -> Result<()> {
		let entries = std::fs::read_dir(&self.path_buf)?;
		for entry in entries {
			let entry = entry?;
			let path = entry.path();
			
			let mut name_parts = path.file_stem()
				.ok_or(anyhow!("Couldn't extract file name"))?
				.to_str().unwrap()
				.split('_');
			
			let prefix = name_parts.next().unwrap();
			let id = name_parts.next().unwrap().parse::<u16>()?;
			
			let object_class = ObjectClass::from_prefix(prefix);
			let dynamic_image = image::open(path)?;
			self.objects.push(Object {
				object_class,
				id,
				dynamic_image,
			});
		}
		
		Ok(())
	}
	
	pub fn generate_set(&self, amount: u32) -> Result<HashSet<&Object>> {
		let mut rng = rand::thread_rng();
		let mut set = HashSet::new();
		
		if amount > self.objects.len() as u32 {
			return Err(anyhow!("Amount of objects requested is greater than the amount of objects available"));
		}
		
		while set.len() < amount as usize {
			let object = self.objects.choose(&mut rng).unwrap();
			set.insert(object);
		}
		
		Ok(set)
	}
}

#[derive(Debug)]
pub struct Object {
	object_class: ObjectClass,
	id: u16,
	dynamic_image: DynamicImage,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
}