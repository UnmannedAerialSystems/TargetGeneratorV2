use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use log::{debug, trace};
use rand::{Rng, thread_rng};
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
    background_loader: BackgroundLoader,
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
            background_loader: BackgroundLoader::new(background_path)?,
            font,
        })
    }

    pub fn generate_target(&self, frequency: f32, resize_factor: f32) -> Result<()> {
        let start = Instant::now(); // start timer
        debug!("Beginning to generate a target...");

        let mut background = self.background_loader.random().unwrap().clone();
        let (w, h) = (background.width(), background.height());

        let amount = (frequency * 20.0) as i32;

        for i in 0..amount {
            let mut shape = self.shape_manager.random_view().unwrap().clone();
            self.draw_random_letter(&mut shape, 1.5, true)?;

            // resizing occurs here so images are not too large for the background
            // resizing might be slow, see https://docs.rs/image/latest/image/imageops/enum.FilterType.html
            let nwidth = (shape.view_inner_image().width() as f32 * resize_factor) as u32;
            let nheight = (shape.view_inner_image().height() as f32 * resize_factor) as u32;

            image::imageops::resize(shape.get_inner_image(), nwidth, nheight, image::imageops::FilterType::Triangle);
            let image = shape.view_inner_image();


            let x = thread_rng().gen_range(0..w) as i64;
            let y = thread_rng().gen_range(0..h) as i64;

            image::imageops::overlay(&mut background, image, x, y);

            trace!("Generation {} completed at {}ms", i, start.elapsed().as_millis());
        }

        debug!("Generation completed, generated {} in average {}ms", amount, start.elapsed().as_millis() / amount as u128);

        background.save("output.png")?;

        Ok(())
    }

    pub fn draw_random_letter(&self, shape: &mut Shape, text_size: f32, do_random_rotate: bool) -> Result<()> {
        let letter = random_letter();
        let center = shape.get_center();
        let center = (
            (center.0 as f32 * 0.95 * (1.0 - (1.0 / text_size))) as u32, // TODO: more tuning here, maybe change text_size beforehand based on shape
            (center.1 as f32 * 1.1 * (1.0 - (1.0 / text_size))) as u32,
        );

        let image = shape.get_inner_image();

        let scale = rusttype::Scale {
            x: (image.height() as f32 / 2.0) * 1.5 * text_size,
            y: (image.height() as f32 / 2.0) * text_size,
        };
        let color = random_color().get_rgb();

        draw_text_mut(image, color, center.0 as i32, center.1 as i32, scale, &self.font, &letter.to_string());
        Ok(())
    }

    // text colors: orange, grey, yellow, black, white, purple
}

#[test]
#[ignore]
pub fn test_generate_image_nobg() {
    SimpleLogger::new().init().unwrap();

    let tg = TargetGenerator::new("output", "backgrounds", "shapes").unwrap();

    let mut shape = tg.shape_manager.random_view().unwrap().clone();

    tg.draw_random_letter(&mut shape, 1.5, true).unwrap();
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

    let mut shape = tg.shape_manager.random_view().unwrap().clone();
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

#[test]
#[ignore]
pub fn test_generate_target() {
    SimpleLogger::new().init().unwrap();

    let tg = TargetGenerator::new("output", "backgrounds", "shapes").unwrap();
    tg.generate_target( 0.5, 0.1).unwrap();
}