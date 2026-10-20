use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use crate::math::Vec2;

pub trait Polygon {
    fn from_values(a: Vec2, b: Vec2, c: Vec2, color: u32) -> Self;
}

pub fn parse_2dv<T: Polygon>(file_path: &Path, polygons: &mut Vec<T>) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let num_lines = lines.size_hint();

    polygons.clear();
    polygons.reserve(num_lines.1.unwrap_or(num_lines.0));

    for line_result in lines {
        let line = line_result?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() == 7 {
            let a = Vec2 {
                x: parts[0].parse::<f32>().unwrap(),
                y: parts[1].parse::<f32>().unwrap(),
            };
            let b = Vec2 {
                x: parts[2].parse::<f32>().unwrap(),
                y: parts[3].parse::<f32>().unwrap(),
            };
            let c = Vec2 {
                x: parts[4].parse::<f32>().unwrap(),
                y: parts[5].parse::<f32>().unwrap(),
            };
            let hex_color = parts[6].strip_prefix("#").unwrap();
            let color = u32::from_str_radix(hex_color, 16).unwrap();

            polygons.push(Polygon::from_values(a, b, c, color));
        }
    }

    Ok(())
}
