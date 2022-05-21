use clap::Parser;

/// Simple cli program to resize images with a seam carving algorithm
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the input image
    #[clap(short, long)]
    pub input: String,

    /// Path to the output image
    #[clap(short, long)]
    pub output: String,

    /// Carver mode (vertical, horizontal)
    #[clap(short, long, arg_enum)]
    pub mode: carver::CarverMode,

    /// Number of passes (seams to carve)
    #[clap(short, long, default_value_t = 1)]
    pub passes: u8,

    /// Debug mode
    #[clap(long)]
    pub debug: bool,
}
