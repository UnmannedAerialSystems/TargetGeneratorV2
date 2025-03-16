// https://docs.aws.amazon.com/rekognition/latest/customlabels-dg/md-coco-overview.html

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Datelike, Local};
use serde::{Deserialize, Serialize};

/// Bounding box format: [x, y, width, height] where 0,0 is the top left corner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,)]
pub struct BoundingBox {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32,
}

impl BoundingBox {
	pub fn collides_with(&self, other: &Self) -> bool {
		self.x < other.x + other.width && self.x + self.width > other.x && self.y < other.y + other.height && self.y + self.height > other.y
	}
}

pub struct CocoGenerator {
	image_id: u32,
	annotation_id: u32,
	category_id: u32,
	pub file: CocoFormatFile,
	file_path: PathBuf,
}

impl CocoGenerator {
	pub fn new<Q: AsRef<Path>>(file_path: Q, categories: Vec<CocoCategory>) -> Self {
		let mut s = Self {
			image_id: 0,
			annotation_id: 0,
			category_id: 0,
			file: CocoFormatFile::default(),
			file_path: file_path.as_ref().to_path_buf(),
		};
		
		s.file.categories = categories;
		
		s
	}
	
	/// Write the contents of the annotations.json file to disk
	pub fn save(&self) {
		let file = OpenOptions::new().write(true).truncate(true).create(true).open(&self.file_path).unwrap();
		
		serde_json::to_writer_pretty(file, &self.file).unwrap();
	}
	
	/// Add a background image in, then return the image id
	pub fn add_image(&mut self, width: u32, height: u32, file_name: String, date_captured: String) -> u32 {
		self.file.images.push(CocoImage {
			id: self.image_id,
			license: None,
			coco_url: None,
			flickr_url: None,
			width,
			height,
			file_name,
			date_captured,
		});
		self.image_id += 1;
		
		self.image_id
	}
	
	pub fn add_annotation(&mut self, image_id: u32, category_id: u32, iscrowd: u8, segmentation: Vec<Vec<f32>>, area: f64, bbox: BoundingBox) -> u32 {
		self.file.annotations.push(CocoAnnotation {
			id: Some(self.annotation_id),
			image_id,
			category_id,
			iscrowd,
			segmentation,
			area,
			bbox,
		});
		self.annotation_id += 1;
		
		self.annotation_id
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CocoFormatFile {
	info: CocoFormatInfo,
	licenses: Vec<CocoFormatLicense>,
	images: Vec<CocoImage>,
	annotations: Vec<CocoAnnotation>,
	categories: Vec<CocoCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoFormatInfo {
	description: String,
	url: String,
	version: String,
	year: u32,
	contributor: String,
	date_created: String,
}

impl Default for CocoFormatInfo {
	fn default() -> Self {
		let datetime: DateTime<Local> = SystemTime::now().into();
		
		Self {
			description: "Auto Generated Dataset in COCO format".to_string(),
			url: "".to_string(),
			version: "1.0".to_string(),
			year: datetime.year() as u32,
			contributor: "".to_string(),
			date_created: datetime.to_string(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoFormatLicense {
	url: String,
	id: u32,
	name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoImage {
	id: u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	license: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	coco_url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	flickr_url: Option<String>,
	width: u32,
	height: u32,
	file_name: String,
	date_captured: String, // format "YYYY-MM-DD HH:MM:SS" ?
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoAnnotation {
	#[serde(skip_serializing_if = "Option::is_none")]
	id: Option<u32>,
	image_id: u32,
	category_id: u32,
	iscrowd: u8,
	segmentation: Vec<Vec<f32>>,
	area: f64,
	bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoCategory {
	pub(crate) id: u32,
	name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub supercategory: Option<String>,
}

impl CocoCategory {
	pub fn new(id: u32, name: String) -> Self {
		Self {
			id,
			name,
			supercategory: None,
		}
	}
}

pub trait CocoCategoryInfo {
	fn categories(&self) -> Vec<CocoCategory>;
}

#[test]
fn test_collision_detector() {
	let a = BoundingBox { x: 0, y: 0, width: 10, height: 10 };
	let b = BoundingBox { x: 5, y: 5, width: 10, height: 10 };
	let c = BoundingBox { x: 20, y: 20, width: 10, height: 10 };
	
	assert!(a.collides_with(&b));
	assert!(!a.collides_with(&c));
	assert!(b.collides_with(&a));
	assert!(a.collides_with(&a));
}