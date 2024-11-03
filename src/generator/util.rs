use log::debug;
use crate::generator::error::GenerationError;

pub const STANDARD_PPM: f32 = 25.0;

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