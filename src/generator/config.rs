use image::Rgba;

#[derive(Debug, Clone, PartialEq)]
pub struct TargetGeneratorConfig {
	pub visualize_bboxes: bool,
	pub maskover_color: Option<Rgba<u8>>
}

impl Default for TargetGeneratorConfig {
	fn default() -> Self {
		Self {
			visualize_bboxes: false,
			maskover_color: None
		}
	}
}