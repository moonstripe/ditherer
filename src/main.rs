use clap::{arg, command, Parser};
use image::{
    DynamicImage, GenericImageView, GrayImage, ImageBuffer, ImageEncoder, ImageReader, Luma, Rgba,
};
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct DithererArgs {
    #[arg(short = 'i', long, value_name = "INPUT_IMG")]
    input: Option<PathBuf>,

    #[arg(short = 'o', long, value_name = "OUTPUT_IMG")]
    output: Option<PathBuf>,

    #[arg(short, long, value_name = "MATRIX_SIZE")]
    matrix_size: BayerMatrixOption,

    #[arg(
        short,
        long,
        help = "Preserve colors using brightness channel dithering"
    )]
    color: bool,

    #[arg(
        short,
        long,
        value_name = "PRESERVE_ORDER",
        help = "Preserve order in 'dark' or 'light' pixels"
    )]
    preserve_order: Option<PreserveOrder>,
}

#[derive(Clone, Debug)]
enum PreserveOrder {
    Dark,
    Light,
}

impl FromStr for PreserveOrder {
    type Err = PreserveOrderParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "dark" => Ok(PreserveOrder::Dark),
            "light" => Ok(PreserveOrder::Light),
            _ => Err(PreserveOrderParseError),
        }
    }
}
#[derive(Debug)]
struct PreserveOrderParseError;

impl fmt::Display for PreserveOrderParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid preserve order option. Choose from: dark, light."
        )
    }
}

impl Error for PreserveOrderParseError {}

#[derive(Clone, Debug)]
enum BayerMatrixOption {
    M2,
    M4,
    M8,
}

impl FromStr for BayerMatrixOption {
    type Err = BayerMatrixParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "m2" => Ok(BayerMatrixOption::M2),
            "m4" => Ok(BayerMatrixOption::M4),
            "m8" => Ok(BayerMatrixOption::M8),
            _ => Err(BayerMatrixParseError),
        }
    }
}

#[derive(Debug)]
struct BayerMatrixParseError;

impl fmt::Display for BayerMatrixParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Bayer Matrix option. Choose from: m2, m4, m8.")
    }
}

impl Error for BayerMatrixParseError {}

const BAYER_MATRIX_2X2: [u8; 4] = [0, 2, 3, 1];
const BAYER_MATRIX_4X4: [u8; 16] = [
    0, 128, 32, 160, 192, 64, 224, 96, 48, 176, 16, 144, 240, 112, 208, 80,
];
const BAYER_MATRIX_8X8: [u8; 64] = [
    0, 128, 32, 160, 48, 176, 16, 144, 192, 64, 224, 96, 240, 112, 208, 80, 32, 160, 48, 176, 16,
    144, 32, 160, 160, 96, 224, 64, 240, 80, 192, 128, 48, 176, 16, 144, 32, 160, 48, 176, 176,
    224, 96, 64, 240, 80, 192, 128, 16, 144, 32, 160, 48, 176, 16, 144, 144, 80, 208, 128, 192,
    128, 160, 96,
];

fn main() -> Result<(), Box<dyn Error>> {
    let args = DithererArgs::parse();

    let image = if let Some(input_path) = args.input {
        ImageReader::open(input_path)?.decode()?
    } else {
        let mut buffer = Vec::new();
        std::io::stdin().lock().read_to_end(&mut buffer)?;
        image::load_from_memory(&buffer)?
    };

    let dithered_image = if args.color {
        let preserve_order = args.preserve_order.unwrap_or(PreserveOrder::Dark);
        apply_bayer_dithering_color(&image, args.matrix_size, preserve_order)
    } else {
        luma_to_rgba8(&apply_bayer_dithering_grayscale(&image, args.matrix_size))
    };

    if let Some(output_path) = args.output {
        dithered_image.save(output_path)?;
    } else {
        let mut stdout = std::io::stdout();
        let encoder = image::codecs::png::PngEncoder::new(&mut stdout);
        encoder.write_image(
            &dithered_image,
            dithered_image.width(),
            dithered_image.height(),
            image::ExtendedColorType::Rgba8,
        )?;
        stdout.flush()?;
    }

    Ok(())
}
fn luma_to_rgba8(luma_img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = luma_img.dimensions();
    let mut rgba_img = ImageBuffer::new(width, height);

    for (x, y, luma_pixel) in luma_img.enumerate_pixels() {
        let luma_value = luma_pixel.0[0];
        rgba_img.put_pixel(x, y, Rgba([luma_value, luma_value, luma_value, 255]));
    }

    rgba_img
}
fn compute_luminance(pixel: &[u8; 3]) -> u8 {
    (0.299 * pixel[0] as f64 + 0.587 * pixel[1] as f64 + 0.114 * pixel[2] as f64).clamp(0.0, 255.0)
        as u8
}

fn apply_bayer_dithering_grayscale(
    image: &DynamicImage,
    bayer_option: BayerMatrixOption,
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let gray_image = image.to_luma8();
    let (width, height) = gray_image.dimensions();

    let (bayer_matrix, matrix_size): (&[u8], u32) = match bayer_option {
        BayerMatrixOption::M2 => (&BAYER_MATRIX_2X2, 2),
        BayerMatrixOption::M4 => (&BAYER_MATRIX_4X4, 4),
        BayerMatrixOption::M8 => (&BAYER_MATRIX_8X8, 8),
    };

    let mut output_image = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = gray_image.get_pixel(x, y);
            let intensity = pixel[0];

            let index = ((y % matrix_size) * matrix_size + (x % matrix_size)) as usize;
            let threshold = bayer_matrix[index];

            let new_intensity = if intensity > threshold { 255 } else { 0 };
            output_image.put_pixel(x, y, Luma([new_intensity]));
        }
    }

    output_image
}

fn apply_bayer_dithering_color(
    image: &DynamicImage,
    bayer_option: BayerMatrixOption,
    preserve_order: PreserveOrder,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (width, height) = image.dimensions();

    let (bayer_matrix, matrix_size): (&[u8], u32) = match bayer_option {
        BayerMatrixOption::M2 => (&BAYER_MATRIX_2X2, 2),
        BayerMatrixOption::M4 => (&BAYER_MATRIX_4X4, 4),
        BayerMatrixOption::M8 => (&BAYER_MATRIX_8X8, 8),
    };

    let mut output_image = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y).0;
            let intensity = compute_luminance(&[pixel[0], pixel[1], pixel[2]]);

            let index = ((y % matrix_size) * matrix_size + (x % matrix_size)) as usize;
            let threshold = bayer_matrix[index];
            let new_intensity = match preserve_order {
                PreserveOrder::Light => {
                    if intensity > threshold {
                        255
                    } else {
                        0
                    }
                }
                PreserveOrder::Dark => {
                    if intensity > threshold {
                        0
                    } else {
                        255
                    }
                }
            };

            output_image.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], new_intensity]));
        }
    }

    output_image
}
