use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerationError {
	#[error("Calculated new sizes provided an invalid size")]
	SizeError,
	#[error("No objects were provided to generate")]
	NoObjects,
	#[error("Path provided is not a directory")]
	NotADirectory,
	#[error("Missing objects.json file")]
	MissingObjectsJSON,
	#[error("{0}")]
	GenericError(String),
	#[error("Not enough objects available to generate")]
	NotEnoughObjectsAvailable,

	// conversions
	#[error("Error parsing integer")]
	ParseIntError(#[from] std::num::ParseIntError),
	#[error("Error parsing image")]
	ImageError(#[from] image::ImageError),
	#[error("Serde decoding or encoding error")]
	SerdeError(#[from] serde_json::Error),
	#[error("IO error occurred while generating target")]
	IOError(#[from] std::io::Error),
}