use image::Rgba;

#[derive(Debug, Clone, PartialEq)]
pub struct TargetGeneratorConfig {
	pub visualize_bboxes: bool,
	pub maskover_color: Option<Rgba<u8>>,
	pub permit_duplicates: bool,
	pub permit_collisions: bool,
	pub cache_size: u8,
}

impl Default for TargetGeneratorConfig {
	fn default() -> Self {
		Self {
			visualize_bboxes: false,
			maskover_color: None,
			permit_duplicates: false,
			permit_collisions: false,
			cache_size: 10,
		}
	}
}