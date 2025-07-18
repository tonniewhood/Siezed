use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

#[allow(non_camel_case_types)]
pub enum CompressionType {
    BI_RGB,
    BI_RLE8,
    BI_RLE4,
}

#[derive(Debug)]
pub struct HeaderData {
    pub file_size: u32,
    pub data_start: u32,
    pub width: u32,
    pub height: u32,
    pub num_planes: u32,
    pub bits_per_pixel: u32,
    pub compression: u32,
    pub compressed_image_size: u32,
    pub colors_used: u32,
    pub number_important_colors: u32,
    pub all_colors: Vec<super::Pixel>,
}

impl HeaderData {
    pub fn new<R: BufRead>(reader: &mut R) -> anyhow::Result<Self> {
        let mut header: [u8; 54] = [0; 54];

        reader.read_exact(&mut header)?;

        if String::from_utf8(header[0..2].to_vec())? != "BM" {
            return Err(anyhow::anyhow!(
                "File not a proper bmp image (Magic bytes mismatch)"
            ));
        }

        let file_size = header[2..6]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let data_start = header[10..14]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let width = header[0x12..0x16]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let height = header[0x16..0x1A]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let num_planes = header[0x1A..0x1C]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let bits_per_pixel = header[0x1C..0x1E]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let compression = header[0x1E..0x22]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let compressed_image_size = header[0x22..0x26]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let colors_used = header[0x2E..0x32]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);
        let number_important_colors = header[0x32..0x36]
            .iter()
            .rev()
            .fold(0, |acc, &byte| (acc << 8) | byte as u32);

        let mut color_table = vec![0u8; colors_used as usize * 3];
        reader.read_exact(&mut color_table)?;

        let mut all_colors = vec![super::Pixel::default(); colors_used as usize];
        for (idx, pixel) in all_colors.iter_mut().enumerate() {
            pixel.r = header[0x36 + idx * 3];
            pixel.g = header[0x37 + idx * 3];
            pixel.b = header[0x38 + idx * 3];
        }

        Ok(Self {
            file_size,
            data_start,
            width,
            height,
            num_planes,
            bits_per_pixel,
            compression,
            compressed_image_size,
            colors_used,
            number_important_colors,
            all_colors,
        })
    }
}

pub fn monochrome_bmp(_data: &HeaderData) -> anyhow::Result<super::Image> {
    panic!("monochrome not implemented yet");
}

pub fn bit4_palletized_bmp(_data: &HeaderData) -> anyhow::Result<super::Image> {
    panic!("4 bit palletized not implemented yet");
}

pub fn bit8_palletized_bmp(_data: &HeaderData) -> anyhow::Result<super::Image> {
    panic!("8 bit palletized not implemented yet");
}

#[allow(non_snake_case)]
pub fn bit16_RGB(_data: &HeaderData) -> anyhow::Result<super::Image> {
    panic!("16 bit RGB not implemented yet");
}

#[allow(non_snake_case)]
pub fn bit24_RGB<R: BufRead + Seek>(
    data: &HeaderData,
    reader: &mut R,
    no_aspect: bool,
) -> anyhow::Result<super::Image> {
    reader.seek(SeekFrom::Start(data.data_start as u64))?;

    let mut rgb_bytes: [u8; 3] = [0; 3];
    let mut rgb_data = vec![super::Pixel::default(); data.width as usize * data.height as usize];

    for scan_row in (0..(data.height as usize)).rev() {
        for scan_col in 0..(data.width as usize) {
            reader.read_exact(&mut rgb_bytes)?;
            let pixel_idx = scan_row * data.width as usize + scan_col;
            rgb_data[pixel_idx].r = rgb_bytes[2];
            rgb_data[pixel_idx].g = rgb_bytes[1];
            rgb_data[pixel_idx].b = rgb_bytes[0];
            rgb_data[pixel_idx].argb = 0xFF << 24
                | (rgb_bytes[2] as u32) << 16
                | (rgb_bytes[1] as u32) << 8
                | (rgb_bytes[0] as u32);
        }

        let row_padding = (4 - ((data.width as usize * 3) % 4)) % 4;
        if row_padding > 0 {
            reader.consume(row_padding);
        }
    }

    println!("Read Image");

    Ok(super::Image {
        width: data.width,
        height: data.height,
        image_data: rgb_data,
        locked_aspect_ratio: !no_aspect,
    })
}

pub fn parse_bmp(filepath: &Path, no_aspect: bool) -> anyhow::Result<super::Image> {
    let mut file_reader = BufReader::new(File::open(filepath)?);

    let header_data = HeaderData::new(&mut file_reader)?;

    match header_data.bits_per_pixel {
        // 1 => monochrome_bmp(&header_data),
        // 4 => bit4_palletized_bmp(&header_data),
        // 8 => bit8_palletized_bmp(&header_data),
        // 16 => bit16_RGB(&header_data),
        24 => bit24_RGB(&header_data, &mut file_reader, no_aspect),
        _ => Err(anyhow::anyhow!(
            "Improper number of bits per pixel {}",
            header_data.bits_per_pixel
        )),
    }
}
