use image::Rgba;

/// The config values for generating target images. Setting these values is optional, they will default 
/// to the predefined values.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TargetGeneratorConfig {
	/// whether or not to visualize the bounding boxes of the objects
	pub visualize_bboxes: bool,
	/// the color to use for the maskover effect, which basically fills the bounding box with a color
	pub maskover_color: Option<Rgba<u8>>,
	/// whether or not to allow duplicates of the same object within the same generated target image
	pub permit_duplicates: bool,
	/// whether or not to allow objects to collide with each other, AKA overlap
	pub permit_collisions: bool,
	/// the size of the cache in MBs, which holds resized objects (initialization only)
	pub cache_size: u8, // TODO: currently only used for initial size, can't be changed
	/// The number of worker threads to use for generating the target images
	pub worker_threads: u8,
	/// Whether or not to compress the generated target images
	pub compress: bool,
	/// Should the objects be randomly rotated (currently only supports 90 degree rotations)
	pub do_random_rotation: bool,
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
			compress: true,
			do_random_rotation: true
		}
	}
}