use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub type Pair = (i32, i32);

pub trait Polygon {
    fn from_values(x: Pair, y: Pair, z: Pair, color: u32) -> Self;
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
            let x: Pair = (
                parts[0].parse::<i32>().unwrap(),
                parts[1].parse::<i32>().unwrap(),
            );
            let y: Pair = (
                parts[2].parse::<i32>().unwrap(),
                parts[3].parse::<i32>().unwrap(),
            );
            let z: Pair = (
                parts[4].parse::<i32>().unwrap(),
                parts[5].parse::<i32>().unwrap(),
            );
            let hex_color = parts[6].strip_prefix("#").unwrap();
            let color = u32::from_str_radix(hex_color, 16).unwrap();

            polygons.push(Polygon::from_values(x, y, z, color));
        }
    }

    Ok(())
}
