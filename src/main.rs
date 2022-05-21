mod opts;

use std::path::Path;

use clap::Parser;

use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageFormat};

use crate::opts::Args;
use carver::error::Result;

fn read_image(path: &Path) -> Result<DynamicImage> {
    Ok(ImageReader::open(path)?.with_guessed_format()?.decode()?)
}

fn write_image(path: &Path, image: DynamicImage) -> Result<()> {
    Ok(image.save_with_format(path, ImageFormat::Png)?)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input_image = read_image(Path::new(&args.input))?;
    let output_image = carver::process(input_image, args.mode, args.passes, args.debug);

    write_image(Path::new(&args.output), output_image)?;

    Ok(())
}
