# TargetGeneratorV2
The purpose of this tool is to generate a set of sample images that closely resemble
the types of images that might be taken from a camera mounted on a drone. The images
are simulated for approximately 150-300ft of altitude.

In addition, each image is designed to have a wide range of visual characteristics to test a computer vision pipeline's
accuracy in real-world conditions. Some of the characteristics include:
- Varying background terrain fidelity, brightness, and contrast
- Varying image angles
- Varying object sizes, shapes, and colors
- Ambient light variations
- Targets of non-interest such as tents, trees, runways markers, roads, people

This tool was made to compete in the [SUAS](https://suas-competition.org/) competition

## Running
To run the tool, you must install Rust on your machine

1. Download Rust from [here](https://www.rust-lang.org/tools/install)
2. Navigate to the root of the project after cloning it your machine
   - `git clone https://github.com/dec4234/TargetGeneratorV2` 
3. Run `cargo run --release` to generate the images
   - Note that the tool will eventually include custom CLI options to give it more flexibility.
