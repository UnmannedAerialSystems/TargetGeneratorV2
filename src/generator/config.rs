//! Configuration for the target generator, which consists of the following:
//! - visualize_bboxes: whether or not to visualize the bounding boxes of the objects
//! - maskover_color: the color to use for the maskover effect, which basically fills the bounding box with a color
//! - permit_duplicates: whether or not to allow duplicates of the same object within the same generated target image
//! - permit_collisions: whether or not to allow objects to collide with each other, AKA overlap
//! - cache_size: the size of the cache in MBs, which holds resized objects (initialization only)
//! - worker_threads: the number of worker threads to use for generating the target image
use image::Rgba;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TargetGeneratorConfig {
	pub visualize_bboxes: bool,
	pub maskover_color: Option<Rgba<u8>>,
	pub permit_duplicates: bool,
	pub permit_collisions: bool,
	pub cache_size: u8, // TODO: currently only used for initial size, can't be changed
	pub worker_threads: u8,
	pub compress: bool
}

impl Default for TargetGeneratorConfig {
	fn default() -> Self {
		Self {
			visualize_bboxes: false,
			maskover_color: None,
			permit_duplicates: false,
			permit_collisions: false,
			cache_size: 10,
			worker_threads: 15,
			compress: true
		}
	}
}