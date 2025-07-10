
use std::fs::File;
use std::io::Read;
use std::path::Path;

const PPM_MAGIC_NUMER: [u8; 2] = [b'P', b'6'];

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub image_data: Vec<u32>
}


#[derive(thiserror::Error, Debug)]
pub enum ImageError {
    #[error(transparent)]
    Io(std::io::Error),
    #[error("Invalid Format ({0})")]
    InvalidFormat(String),
    #[error("Unsupported pixel type (>8 bits/channel)")]
    Unsupported(String)
}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> Self {
        ImageError::Io(err)
    }
}

impl Image {
    pub fn new(image_filename: String) -> Result<Self, ImageError> {
        let path = Path::new(&image_filename);
        
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => return Err(ImageError::Io(err)),
        };

        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;
        
        Image::parse_image(&mut file_buffer)
    }

    fn safe_idx_increment(idx: Option<usize>, max_val: usize) -> Option<usize> {
        if idx.is_none() {
            return None;
        }

        if idx.unwrap() + 1 > max_val {
            return None;
        }

        idx.map(|i| i + 1)
    } 

    fn parse_image(buf: &[u8]) -> Result<Self, ImageError> {

        if buf.len() < 2 || &buf[0..2] != PPM_MAGIC_NUMER {
            return Err(ImageError::InvalidFormat("Invalid magic number".to_string()));
        }

        let mut width= Vec::<u8>::new();
        let mut height = Vec::<u8>::new();
        let mut pixel_max = Vec::<u8>::new();
        let mut img_start: Option<usize> = None;
        let mut current_byte: u8;
        let mut entry_idx: usize = 0;

        for idx in 2..buf.len() {
            current_byte = buf[idx];
            if current_byte.is_ascii_whitespace() { 
                if width.len() > 0 && height.len() > 0 && pixel_max.len() > 0 {

                    img_start = Image::safe_idx_increment(Some(idx), buf.len());
                    if img_start.is_none() { 
                        println!("Bad initial Increment");
                        break; 
                    }

                    let mut last_idx = img_start.unwrap();
                    while buf[img_start.unwrap()].is_ascii_whitespace() {
                        img_start = Image::safe_idx_increment(img_start, buf.len());
                        if img_start.is_none() { 
                            println!("Bad final idx ({last_idx})");
                            break; 
                        }
                        last_idx = img_start.unwrap();
                    }

                    break;
                }
                else {
                    entry_idx = 0;
                    continue;
                }
            }

            match (width.len() == 0, height.len() == 0, pixel_max.len() == 0) {
                (true, true, true) => {
                    width.push(current_byte);
                }
                (false, true, true) => {
                    if entry_idx == 0 {
                        height.push(current_byte);
                    } else {
                        width.push(current_byte);
                    }
                }
                (false, false, true) => {
                    if entry_idx == 0 {
                        pixel_max.push(current_byte)
                    } else {
                        height.push(current_byte);
                    }
                }
                (false, false, false) => {
                    pixel_max.push(current_byte);
                }
                _ => return Err(ImageError::InvalidFormat("Unexpected value sequence encountered".to_string()))
            }

            entry_idx += 1;
        }

        match (width.len() > 0, height.len() > 0, pixel_max.len() > 0, img_start) {
            (true, true, true, Some(img_base)) => {
                // Convert a vector of ASCII digit bytes to a decimal value
                let dec_width = width.iter().fold(0u32, |acc, &b| acc * 10 + (b - b'0') as u32);
                let dec_height = height.iter().fold(0u32, |acc, &b| acc * 10 + (b - b'0') as u32);
                let dec_pixel_max = pixel_max.iter().fold(0u32, |acc, &b| acc * 10 + (b - b'0') as u32);
                if dec_width < 1 {
                    return Err(ImageError::InvalidFormat("Width must be greater than 0".to_string()));
                }
                if dec_height < 1 {
                    return Err(ImageError::InvalidFormat("Height must be greater than 0".to_string()));
                }
                if dec_pixel_max < 1 || dec_pixel_max > (u16::MAX as u32) {
                    return Err(ImageError::InvalidFormat("Max value must be in range 0 < maxval < 65535".to_string()));
                }
                if dec_pixel_max < (u8::MAX as u32) {
                    return Err(ImageError::Unsupported("".to_string()));
                }
                if (buf.len() - img_base) % 3 != 0 {
                    return Err(ImageError::InvalidFormat("Improper number of pixels; Not enough values to split R-G-B properly".to_string()));
                }
                let num_pixels = (buf.len() - img_base) / 3;
                if num_pixels != ((dec_width * dec_height) as usize) {
                    return Err(ImageError::InvalidFormat("Width by Height dimensions must match the image data".to_string()))
                }
                let mut rgb_data = vec![0u32; num_pixels];
                let bytes_per_pixel = 3;
                for pixel_idx in 0..num_pixels {
                    let base_buf_idx = pixel_idx * bytes_per_pixel + img_base;
                    for triplet_idx in 0..bytes_per_pixel {
                        rgb_data[pixel_idx] += (buf[base_buf_idx + triplet_idx] as u32) << (8 * (bytes_per_pixel - triplet_idx - 1));
                    }
                }
                Ok(Self {
                    width: dec_width,
                    height: dec_height,
                    image_data: rgb_data
                })
            },
            _ => Err(ImageError::InvalidFormat("Invalid header format".to_string()))
        }

        
    }
}
