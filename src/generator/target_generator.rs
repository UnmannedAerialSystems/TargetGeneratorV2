use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;

pub struct TargetGenerator {
    output: PathBuf,
    backgrounds_path: PathBuf,
    shapes_path: PathBuf,
}

impl TargetGenerator {
    pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, shapes_path: Q) -> Self {
        Self {
            output: output.as_ref().to_path_buf(),
            backgrounds_path: background_path.as_ref().to_path_buf(),
            shapes_path: shapes_path.as_ref().to_path_buf(),
        }
    }

    pub fn generate(&self, amount: usize) -> Result<()> {
        let start = Instant::now(); // start timer

        Ok(())
    }
}