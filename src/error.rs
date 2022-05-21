use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    IO(#[from] std::io::Error),

    #[error("Invalid image format")]
    InvalidImage(#[from] image::ImageError),
}
