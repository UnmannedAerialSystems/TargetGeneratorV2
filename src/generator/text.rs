use image::{Rgb, Rgba};
use rand::thread_rng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;

pub fn random_letter() -> char {
    let mut rng = thread_rng();
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let letter = letters.chars().choose(&mut rng).unwrap();
    letter
}

pub fn random_color() -> TextColor {
    let mut rng = thread_rng();
    let colors = vec![
        TextColor::ORANGE,
        TextColor::GREY,
        TextColor::YELLOW,
        TextColor::BLACK,
        TextColor::WHITE,
        TextColor::PURPLE,
    ];
    *colors.choose(&mut rng).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextColor {
    ORANGE,
    GREY,
    YELLOW,
    BLACK,
    WHITE,
    PURPLE,
}

impl TextColor {
    pub fn get_rgb(&self) -> Rgba<u8> {
        match self {
            TextColor::ORANGE => Rgba([255, 165, 0, 100]),
            TextColor::GREY => Rgba([128, 128, 128, 100]),
            TextColor::YELLOW => Rgba([255, 255, 0, 100]),
            TextColor::BLACK => Rgba([0, 0, 0, 100]),
            TextColor::WHITE => Rgba([255, 255, 255, 100]),
            TextColor::PURPLE => Rgba([128, 0, 128, 100]),
        }
    }
}