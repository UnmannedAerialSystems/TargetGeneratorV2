use std::fs;
use std::path::Path;
use crate::generator::error::GenerationError;
use image::metadata::Orientation;
use image::DynamicImage;

/// The standard Pixels Per Meter value that is used to calculate the size of objects in pixels.
/// In reality this value is dependent on the altitude of the drone and various properties of the
/// camera that is being used.
pub const STANDARD_PPM: f32 = 45.0;

/// Use the real size of an object and the Pixels Per Meter value to calculate the size in 
/// pixels that it should be in order to be at scale
fn resize_ratio(object_real_size: f32, pixels_per_meter: f32) -> f32 {
	object_real_size * pixels_per_meter
}

/// Calculate the new sizes of an object in pixels based on the requested Pixel Per Meter value
/// 1. Calculate the aspect ratio
/// 2. Calculate the width of the object in pixels that we expect based on the real width and the Pixels Per Meter value
/// 3. Calculate the height from this new width using the previously calculated aspect ratio
pub fn new_sizes(object_width: u32, object_height: u32, pixels_per_meter: f32, real_width: f32) -> anyhow::Result<(u32, u32), GenerationError> {
	let (w, h) = (object_width as f32, object_height as f32);
	let aspect_ratio = w / h;
	let new_width = resize_ratio(real_width, pixels_per_meter) as u32;
	let new_height = (new_width as f32 / aspect_ratio) as u32;
	
	if new_height == 0 || new_width == 0 {
		return Err(GenerationError::SizeError);
	}
	
	Ok((new_width, new_height))
}

pub fn rotate_90s(image: &DynamicImage, angle: i32) -> DynamicImage {
	let mut i = image.clone();
	
	let angle = (angle) / 90;
	
	match angle {
		1 => i.apply_orientation(Orientation::Rotate90),
		2 => i.apply_orientation(Orientation::Rotate180),
		3 => i.apply_orientation(Orientation::Rotate270),
		_ => (),
	}
	
	i
}

pub fn post_rotate_dimension(width: u32, height: u32, angle: f32) -> (u32, u32) {
	let (width, height) = (width as f32, height as f32);
	let angle = angle.to_radians();
	let (sin, cos) = angle.sin_cos();

	let new_width = (width * cos + height * sin).abs() as u32;
	let new_height = (width * sin + height * cos).abs() as u32;

	(new_width, new_height)
}

// TODO: various problems with this at present. Cuts off the image mostly in multiple ways
pub fn rotate_image(image: &DynamicImage, angle: f32) -> DynamicImage {
	// 
	
	todo!()
}

pub fn is_image_type(path: &str) -> bool {
	path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg")
}

pub fn cleanup_output<P: AsRef<Path>>(folder_path: P) -> std::io::Result<()> {
	for entry in fs::read_dir(folder_path)? {
		let entry = entry?;
		let path = entry.path();
		
		if entry.file_type()?.is_file() && (is_image_type(path.to_str().unwrap()) || path.extension().unwrap() == "json") {
			fs::remove_file(entry.path())?;
		}
	}
	Ok(())
}

#[test]
fn test_is_image_type() {
	assert_eq!(is_image_type("test.png"), true);
	assert_eq!(is_image_type("test.jpg"), true);
	assert_eq!(is_image_type("test.jpeg"), true);
	assert_eq!(is_image_type("test.txt"), false);
}

#[test]
fn test_resize_ratio() {
	assert_eq!(resize_ratio(1.0, 35.0), 35.0);
	assert_eq!(resize_ratio(1.0, 70.0), 70.0);
	assert_eq!(resize_ratio(1.0, 105.0), 105.0);
}