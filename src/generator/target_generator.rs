use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::Font;
use simple_logger::SimpleLogger;
use crate::backgrounds::background_loader::BackgroundLoader;
use crate::generator::text::{random_color, random_letter};

use crate::shapes::shapes::{Shape, ShapeColor, ShapeManager};

pub struct TargetGenerator<'a> {
    output: PathBuf,
    backgrounds_path: PathBuf,
    shapes_path: PathBuf,
    textfont_file: PathBuf,
    pub shape_manager: ShapeManager,
    // background_loader: BackgroundLoader,
    font: Font<'a>,
}

impl TargetGenerator<'_> {
    pub fn new<Q: AsRef<Path>>(output: Q, background_path: Q, shapes_path: Q) -> Result<Self> {
        let font = Vec::from(include_bytes!("../../fonts/DejaVuSans.ttf") as &[u8]);
        let font = Font::try_from_vec(font).unwrap();

        Ok(Self {
            output: output.as_ref().to_path_buf(),
            backgrounds_path: background_path.as_ref().to_path_buf(),
            shapes_path: shapes_path.as_ref().to_path_buf(),
            textfont_file: PathBuf::from("fonts/DejaVuSans.ttf"), // we can change this later
            shape_manager: ShapeManager::new(shapes_path)?,
            font,
        })
    }

    pub fn draw_random_letter(&self, shape: &mut Shape) -> Result<()> {
        let letter = random_letter();
        let center = shape.get_center();
        let image = shape.get_inner_image();

        let height = 80.0;
        let scale = rusttype::Scale {
            x: height * 1.5,
            y: height,
        };
        let color = random_color().get_rgb();

        draw_text_mut(image, color, center.0 as i32, center.1 as i32, scale, &self.font, &letter.to_string());
        Ok(())
    }

    pub fn random_generate(&self, amount: usize) -> Result<()> {
        let start = Instant::now(); // start timer

        Ok(())
    }

    pub fn generate(&self, amount: usize, colors: Vec<ShapeColor>) -> Result<()> {
        let start = Instant::now(); // start timer

        Ok(())
    }

    // text colors: orange, grey, yellow, black, white, purple
}

#[test]
#[ignore]
pub fn test_generate_image_nobg() {
    SimpleLogger::new().init().unwrap();

    let tg = TargetGenerator::new("output", "backgrounds", "shapes").unwrap();

    let mut shape = tg.shape_manager.random().unwrap().clone();

    tg.draw_random_letter(&mut shape).unwrap();
    shape.get_inner_image().save("output.png").unwrap();
}

#[test]
#[ignore]
pub fn test_writing_text() {
    use imageproc::drawing::{draw_text_mut, text_size};
    use rusttype::{Font, Scale};

    SimpleLogger::new().init().unwrap();

    let tg = TargetGenerator::new("output", "backgrounds", "shapes").unwrap();
    // use imageproc
    // https://github.com/image-rs/imageproc/blob/master/examples/font.rs

    let font = Vec::from(include_bytes!("../../fonts/DejaVuSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let mut shape = tg.shape_manager.random().unwrap().clone();
    let mut image = shape.get_inner_image().clone();

    let height = 12.4;
    let scale = Scale {
        x: height * 2.0,
        y: height,
    };

    let center = shape.get_center();

    let text = "Hello, world!";
    draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 100]), center.0 as i32, center.1 as i32, scale, &font, text);
    let (w, h) = text_size(scale, &font, text);
    println!("Text size: {}x{}", w, h);

    let _ = image.save("output.png").unwrap();
}