use anyhow::Ok;

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::path::Path;

fn read_token<R: BufRead>(reader: &mut R) -> anyhow::Result<String> {
    let mut buf = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            break;
        }
        let byte = available[0];
        if byte == b'#' {
            let mut dummy = String::new();
            reader.read_line(&mut dummy)?;
            continue;
        }
        if byte.is_ascii_whitespace() {
            reader.consume(1);
            if buf.is_empty() {
                continue;
            } else {
                break;
            }
        }
        buf.push(byte);
        reader.consume(1);
    }
    Ok(String::from_utf8(buf).expect("header is ASCII"))
}

pub fn parse_ppm(path: &Path, no_aspect: bool) -> anyhow::Result<super::Image> {
    let mut file_buf = BufReader::new(File::open(path)?);

    // magic ("P6" or "P3")
    let magic = read_token(&mut file_buf)?;
    if magic != "P6" && magic != "P3" {
        return Err(anyhow::anyhow!(
            "File not a proper ppm image (Magic bytes mismatch)"
        ));
    }
    // 2) width
    let width: usize = read_token(&mut file_buf)
        .map_err(|_| anyhow::anyhow!("Missing width"))?
        .parse()?;
    // 3) height
    let height: usize = read_token(&mut file_buf)
        .map_err(|_| anyhow::anyhow!("Missing height"))?
        .parse()?;
    // 4) max color value
    let pixel_max: usize = read_token(&mut file_buf)
        .map_err(|_| anyhow::anyhow!("Missing pixel max"))?
        .parse()?;

    if width == 0 || height == 0 {
        return Err(anyhow::anyhow!("Width and height must be non-zero"));
    }
    if pixel_max == 0 || pixel_max > u8::MAX as usize {
        return Err(anyhow::anyhow!(
            "Pixels with values greater than 255 or zero are unsupported"
        ));
    }
    let img_size =
        file_buf.get_ref().metadata()?.len() as usize - file_buf.stream_position()? as usize;
    if img_size % 3 != 0 {
        return Err(anyhow::anyhow!(
            "Improper number of pixels; Not enough values to split R-G-B properly",
        ));
    }
    let num_pixels = img_size / 3;
    if num_pixels != (width * height) as usize {
        return Err(anyhow::anyhow!(
            "Width by Height dimensions must match the image data",
        ));
    }
    let mut rgb_data = vec![super::Pixel::default(); num_pixels];
    let mut pixel_buf: [u8; 3] = [0; 3];

    // Iterate over the image data in chunks of 3 bytes (R, G, B)
    for idx in 0..num_pixels {
        file_buf.read_exact(&mut pixel_buf)?;
        rgb_data[idx].r = pixel_buf[0];
        rgb_data[idx].g = pixel_buf[1];
        rgb_data[idx].b = pixel_buf[2];
        rgb_data[idx].argb = 0xFF << 24
            | (pixel_buf[0] as u32) << 16
            | (pixel_buf[1] as u32) << 8
            | (pixel_buf[2] as u32);
    }

    Ok(super::Image {
        width: width as u32,
        height: height as u32,
        image_data: rgb_data,
        locked_aspect_ratio: !no_aspect,
        is_grayscale: false,
    })
}
