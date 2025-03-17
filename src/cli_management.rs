use std::path::PathBuf;
use clap::Parser;
use log::debug;
use targetgen_lib::generator::TargetGenerator;

#[derive(Parser, Debug)]
#[clap(name = "targetgen", version = "0.1.0", author = "Declan Emery", about = "A tool for generating synthetic bird's eye view images for training machine learning models.")]
pub struct TargetgenCli {
	#[clap(short, long, help = "The path to the backgrounds image directory.")]
	pub backgrounds: PathBuf,
	
	#[clap(short, long, help = "The path to the objects image directory.")]
	pub objects: PathBuf,

	#[clap(short, long, help = "The output folder.")]
	pub output: PathBuf,
	
	#[clap(short, long, help = "The path to the annotations file.")]
	pub annotations: PathBuf,

	#[clap(short, long, help = "Enable logging.")]
	pub enable_logging: Option<bool>,
	
	#[clap(short, long, help = "The number of target images to generate.")]
	pub num_targets: Option<u32>,

	#[clap(short, long, help = "The number of objects per image.")]
	pub num_objects: Option<u32>,

	#[clap(short, long, help = "Whether or not to visualize the bounding boxes of the objects.")]
	pub visualize_bboxes: Option<bool>,
	
	#[clap(short, long, help = "The color to use for the maskover effect, which basically fills the bounding box with a color.")]
	pub maskover_color: Option<String>,
	
	#[clap(short, long, help = "Whether or not to allow duplicates of the same object within the same generated target image.")]
	pub permit_duplicates: Option<bool>,
	
	#[clap(short, long, help = "Whether or not to allow objects to collide with each other, AKA overlap.")]
	pub permit_collisions: Option<bool>,
	
	#[clap(short, long, help = "The size of the cache in MBs, which holds resized objects (initialization only).")]
	pub cache_size: Option<u8>,
	
	#[clap(short, long, help = "The number of worker threads to use for generating the target images.")]
	pub worker_threads: Option<u8>,
	
	#[clap(short, long, help = "Whether or not to compress the generated target images.")]
	pub compress: Option<bool>,
	
	#[clap(short, long, help = "Should the objects be randomly rotated (currently only supports 90 degree rotations).")]
	pub do_random_rotation: Option<bool>,
}

pub fn run(args: TargetgenCli) {
	if let Some(enable_logging) = args.enable_logging {
		if enable_logging {
			simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init().unwrap();
		}
	}
	
	debug!("Running with args: {:?}", args);

	let mut tg = TargetGenerator::new(args.backgrounds, args.objects, args.annotations).unwrap();
	
	let num_targets = args.num_targets.unwrap_or(1);
	let num_objects = args.num_objects.unwrap_or(6);
	
	if let Some(visualize_bboxes) = args.visualize_bboxes {
		tg.config.visualize_bboxes = visualize_bboxes;
	}
	
	/*if let Some(maskover_color) = args.maskover_color {
		tg.config.maskover_color = Some(maskover_color.parse().unwrap());
	}*/
	
	if let Some(permit_duplicates) = args.permit_duplicates {
		tg.config.permit_duplicates = permit_duplicates;
	}
	
	if let Some(permit_collisions) = args.permit_collisions {
		tg.config.permit_collisions = permit_collisions;
	}
	
	if let Some(cache_size) = args.cache_size {
		tg.config.cache_size = cache_size;
	}
	
	if let Some(worker_threads) = args.worker_threads {
		tg.config.worker_threads = worker_threads;
	}
	
	if let Some(compress) = args.compress {
		tg.config.compress = compress;
	}
	
	if let Some(do_random_rotation) = args.do_random_rotation {
		tg.config.do_random_rotation = do_random_rotation;
	}
	
	tg.generate_targets(num_targets, ..num_objects, args.output).unwrap();
	
	tg.close();
	
	debug!("Finished running.");
}