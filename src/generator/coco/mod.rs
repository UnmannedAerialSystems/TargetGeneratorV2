// https://docs.aws.amazon.com/rekognition/latest/customlabels-dg/md-coco-overview.html

use serde::{Deserialize, Serialize};

pub struct CocoGenerator {
	image_id: u32,
}

impl CocoGenerator {
	pub fn new() -> Self {
		Self {
			image_id: 0,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoFormatLicense {

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

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CocoCategory {

}