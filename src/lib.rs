pub mod error;

use std::path::Path;

use image::{DynamicImage, GenericImage, GenericImageView, GrayImage, ImageFormat, Rgb, RgbImage};
use itertools::Itertools;

type Matrix = Vec<Vec<f64>>;

fn transpose(matrix: &Matrix) -> Matrix {
    (0..matrix[0].len())
        .into_iter()
        .map(|x| {
            (0..matrix.len())
                .into_iter()
                .map(|y| matrix[y][x])
                .collect()
        })
        .collect()
}

#[derive(clap::ArgEnum, Debug, Copy, Clone)]
pub enum CarverMode {
    Vertical,
    Horizontal,
}

pub fn process(image: DynamicImage, mode: CarverMode, num_passes: u8, debug: bool) -> DynamicImage {
    let mut image = image;

    for i in 0..num_passes {
        let energy = calculate_energy(&image.to_rgb8());
        let seam = find_seam(&energy, mode);

        if debug {
            write_energy_with_seam(&energy, &seam, mode, Path::new(&format!("debug-{}.png", i)));
        }

        image = remove_seam(&mut image, seam, mode);
    }

    image
}

fn remove_seam(image: &mut DynamicImage, seam: Vec<u32>, mode: CarverMode) -> DynamicImage {
    let mut res = match mode {
        CarverMode::Vertical => DynamicImage::new_rgb8(image.width() - 1, image.height()),
        CarverMode::Horizontal => DynamicImage::new_rgb8(image.width(), image.height() - 1),
    };

    let mut counter = 0;
    for (x, y, rgb) in image.pixels() {
        match mode {
            CarverMode::Vertical => {
                if x != seam[(y / image.height()) as usize] {
                    res.put_pixel(counter, y, rgb);
                    counter += 1;
                }

                if counter == res.width() {
                    counter = 0
                };
            }
            CarverMode::Horizontal => {
                if y != seam[(x / image.width()) as usize] {
                    res.put_pixel(x, counter, rgb);
                    counter += 1;
                }

                if counter == res.height() {
                    counter = 0
                };
            }
        }
    }

    res
}

fn find_seam(energy: &Matrix, mode: CarverMode) -> Vec<u32> {
    let energy = match mode {
        CarverMode::Vertical => energy.clone(),
        CarverMode::Horizontal => transpose(energy),
    };

    let accumulated_cost_matrix = calculate_accumulated_cost(&energy);
    let mut x = accumulated_cost_matrix
        .last()
        .unwrap()
        .iter()
        .enumerate()
        .min_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap())
        .unwrap()
        .0 as u32;

    let mut seam = vec![x as u32];

    for y in (0..accumulated_cost_matrix.len() - 1).rev() {
        x = (-1..=1)
            .into_iter()
            .filter_map(|dx| {
                let x = x as i32 + dx;

                if x == -1 || x == energy[0].len() as i32 {
                    None
                } else {
                    Some((x as u32, accumulated_cost_matrix[y as usize][x as usize]))
                }
            })
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap()
            .0;

        seam.push(x);
    }

    seam.reverse();
    seam
}

fn calculate_accumulated_cost(energy: &Matrix) -> Matrix {
    let mut res = vec![energy.first().unwrap().clone()];

    for y in 1..energy.len() {
        let mut row = vec![];
        for x in 0..energy[0].len() {
            let min_weight = (-1..=1)
                .into_iter()
                .filter_map(|dx| {
                    let x = x as i32 + dx;

                    if x == -1 || x == energy[0].len() as i32 {
                        None
                    } else {
                        Some(res[(y - 1) as usize][x as usize])
                    }
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            row.push(energy[y][x] + min_weight);
        }
        res.push(row);
    }

    res
}

fn calculate_energy(image: &RgbImage) -> Matrix {
    image
        .enumerate_pixels()
        .map(|(x, y, _)| {
            let x_gradient_2 = gradient(
                image.get_pixel(
                    ((x as i32 - 1 + image.width() as i32) % image.width() as i32) as u32,
                    y,
                ),
                image.get_pixel((x + 1) % image.width(), y),
            );

            let y_gradient_2 = gradient(
                image.get_pixel(
                    x,
                    ((y as i32 - 1 + image.height() as i32) % image.height() as i32) as u32,
                ),
                image.get_pixel(x, (y + 1) % image.height()),
            );

            f64::sqrt((x_gradient_2 + y_gradient_2) as f64)
        })
        .chunks(image.width() as usize)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect_vec()
}

fn gradient(p1: &Rgb<u8>, p2: &Rgb<u8>) -> f64 {
    (0..3)
        .map(|sub_pixel| {
            let diff = i64::abs(p1[sub_pixel] as i64 - p2[sub_pixel] as i64);
            diff * diff
        })
        .sum::<i64>() as f64
}

fn write_energy_with_seam(matrix: &Matrix, seam: &[u32], mode: CarverMode, path: &Path) {
    let max_energy = matrix
        .iter()
        .flatten()
        .max_by(|&a, &b| a.partial_cmp(b).unwrap())
        .unwrap();

    let normalized_matrix = matrix
        .iter()
        .flatten()
        .map(|energy| (255f64 * (energy / max_energy)) as u8)
        .collect();

    let mut image = DynamicImage::ImageLuma8(
        GrayImage::from_vec(
            matrix[0].len() as u32,
            matrix.len() as u32,
            normalized_matrix,
        )
        .unwrap(),
    )
    .to_rgb8();

    match mode {
        CarverMode::Vertical => {
            for (y, x) in (0..image.height()).zip(seam) {
                image.put_pixel(*x, y, Rgb::from([255, 0, 0]));
            }
        }
        CarverMode::Horizontal => {
            for (x, y) in (0..image.width()).zip(seam) {
                image.put_pixel(x, *y, Rgb::from([255, 0, 0]));
            }
        }
    }

    image.save_with_format(path, ImageFormat::Png).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    use image::io::Reader as ImageReader;

    fn read_image(path: &Path) -> Result<DynamicImage> {
        Ok(ImageReader::open(path)?.with_guessed_format()?.decode()?)
    }

    #[test]
    fn calculate_energy_works_with_given_image() {
        let image = read_image(Path::new("./test_files/3x4.png")).unwrap();

        let expected = vec![
            vec![
                f64::sqrt(20808f64),
                f64::sqrt(52020f64),
                f64::sqrt(20808f64),
            ],
            vec![
                f64::sqrt(20808f64),
                f64::sqrt(52225f64),
                f64::sqrt(21220f64),
            ],
            vec![
                f64::sqrt(20809f64),
                f64::sqrt(52024f64),
                f64::sqrt(20809f64),
            ],
            vec![
                f64::sqrt(20808f64),
                f64::sqrt(52225f64),
                f64::sqrt(21220f64),
            ],
        ];

        let actual = calculate_energy(image.as_rgb8().unwrap());

        assert_eq!(expected, actual)
    }

    #[test]
    fn transpose_matrix_works() {
        let expected = vec![vec![1f64, 2f64, 3f64], vec![4f64, 5f64, 6f64]];
        let actual = transpose(&vec![vec![1f64, 4f64], vec![2f64, 5f64], vec![3f64, 6f64]]);

        assert_eq!(expected, actual)
    }

    #[test]
    fn calculate_accumulated_cost_works_on_given_matrix() {
        let matrix = vec![
            vec![10000f64, 10000f64, 10000f64],
            vec![10000f64, 52225f64, 10000f64],
            vec![10000f64, 52024f64, 10000f64],
            vec![10000f64, 10000f64, 10000f64],
        ];

        let expected = vec![
            vec![10000f64, 10000f64, 10000f64],
            vec![20000f64, 62225f64, 20000f64],
            vec![30000f64, 72024f64, 30000f64],
            vec![40000f64, 40000f64, 40000f64],
        ];
        let actual = calculate_accumulated_cost(&matrix);

        assert!(actual.iter().enumerate().all(|(y, row)| {
            row.iter()
                .enumerate()
                .all(|(x, weight)| f64::abs(weight - expected[y][x]) < 0.001)
        }))
    }

    #[test]
    fn find_seam_works_for_given_image() {
        let matrix = vec![
            vec![57685.0, 50893.0, 91370.0, 25418.0, 33055.0, 37246.0],
            vec![15421.0, 56334.0, 22808.0, 54796.0, 11641.0, 25496.0],
            vec![12344.0, 19236.0, 52030.0, 17708.0, 44735.0, 20663.0],
            vec![17074.0, 23678.0, 30279.0, 80663.0, 37831.0, 45595.0],
            vec![32337.0, 30796.0, 4909.0, 73334.0, 40613.0, 36556.0],
        ];

        fn works_for_vertical(matrix: &Matrix) {
            let expected = vec![3, 4, 3, 2, 2];
            let actual = find_seam(matrix, CarverMode::Vertical);
            assert_eq!(expected, actual);
        }

        fn works_for_horizontal(matrix: &Matrix) {
            let expected = vec![2, 2, 1, 2, 1, 2];
            let actual = find_seam(matrix, CarverMode::Horizontal);
            assert_eq!(expected, actual);
        }

        works_for_vertical(&matrix);
        works_for_horizontal(&matrix);
    }
}
