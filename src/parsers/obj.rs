use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num;
use std::ops::{IndexMut, RangeInclusive};
use std::path::Path;

use crate::math::{Vec3, Vec4};
use crate::parsers::obj::ObjError::IoError;

#[derive(thiserror::Error, Debug)]
pub enum ObjError {
    #[error(transparent)]
    IoError(io::Error),
    #[error("Vertex line malformed")]
    VertexLineError,
    #[error("Texture line malformed")]
    TextureLineError,
    #[error("Normal line malformed")]
    NormalLineError,
    #[error("Face line malformed")]
    FaceLineError(&'static str),
    #[error(transparent)]
    ParseFloatError(num::ParseFloatError),
    #[error(transparent)]
    ParseIntError(num::ParseIntError),
}

#[derive(Default)]
pub struct ObjContents {
    pub geometric_vertices: Vec<Vec4>,
    pub texture_coords: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Vec<[i32; 3]>>, // In the form of v/vt/vn. -1 indicates not there
}

impl ObjContents {
    pub fn summarize(&self) {
        println!("Vertices ({}):", self.geometric_vertices.len());
        for (i, v) in self.geometric_vertices.iter().enumerate() {
            println!("  v{i}: ({}, {}, {}, {})", v.x, v.y, v.z, v.w);
        }

        println!("Texture coords ({}):", self.texture_coords.len());
        for (i, vt) in self.texture_coords.iter().enumerate() {
            println!("  vt{i}: ({}, {}, {})", vt.x, vt.y, vt.z);
        }

        println!("Normals ({}):", self.normals.len());
        for (i, vn) in self.normals.iter().enumerate() {
            println!("  vn{i}: ({}, {}, {})", vn.x, vn.y, vn.z);
        }

        println!("Faces ({}):", self.faces.len());
        for (i, face) in self.faces.iter().enumerate() {
            print!("  f{i}: ");
            for [v, vt, vn] in face {
                print!("{v}");
                if *vt != -1 {
                    print!("/{vt}");
                }
                if *vn != -1 {
                    print!("/{vn}");
                }
                print!(" ");
            }
            println!();
        }
    }
}

fn read_float_data<T: IndexMut<usize, Output = f32>>(
    line: &str,
    prefix: &str,
    valid_lens: RangeInclusive<usize>,
    vertex: &mut T,
    err: ObjError,
) -> Result<(), ObjError> {
    let Some(rest) = line.strip_prefix(prefix) else {
        return Err(err);
    };
    let line_contents: Vec<&str> = rest.split_whitespace().collect();

    if !valid_lens.contains(&line_contents.len()) {
        return Err(err);
    }

    for (idx, val_str) in line_contents.iter().enumerate() {
        match val_str.parse::<f32>() {
            Ok(val) => vertex[idx] = val,
            Err(e) => return Err(ObjError::ParseFloatError(e)),
        }
    }

    Ok(())
}

fn parse_int(int_str: &str) -> Result<i32, ObjError> {
    match int_str.parse::<i32>() {
        Ok(val) => Ok(val),
        Err(e) => return Err(ObjError::ParseIntError(e)),
    }
}

fn determine_current_incices(idx_str: &str) -> Result<[Option<i32>; 3], ObjError> {
    let inds: Vec<&str> = idx_str.split("/").collect();

    match inds.len() {
        1 => {
            let v = Some(parse_int(inds[0])?);
            return Ok([v, None, None]);
        }
        2 => {
            let v = Some(parse_int(inds[0])?);
            let vt = Some(parse_int(inds[1])?);
            return Ok([v, vt, None]);
        }
        3 => {
            let v = Some(parse_int(inds[0])?);
            let vt = if inds[1].is_empty() {
                None
            } else {
                Some(parse_int(inds[1])?)
            };
            let vn = Some(parse_int(inds[2])?);

            return Ok([v, vt, vn]);
        }
        _ => return Err(ObjError::FaceLineError("Improper number of face indices")),
    };
}

fn read_face_line(line: &str) -> Result<Vec<[Option<i32>; 3]>, ObjError> {
    let Some(rest) = line.strip_prefix("f ") else {
        return Err(ObjError::FaceLineError("Improper prefix"));
    };

    let line_contents: Vec<&str> = rest.split_whitespace().collect();
    if line_contents.len() < 3 {
        return Err(ObjError::FaceLineError(
            "Faces require at least 3 vertices to form",
        ));
    }

    let face_indices = line_contents
        .into_iter()
        .map(determine_current_incices)
        .collect::<Result<Vec<_>, _>>()?;

    let shape = |c: &[Option<i32>; 3]| c.map(|i| i.is_some());
    let first_shape = shape(&face_indices[0]);
    if face_indices[1..].iter().any(|f| shape(f) != first_shape) {
        return Err(ObjError::FaceLineError("Inconsistent vertex style"));
    }

    Ok(face_indices)
}

fn associate_face_vertex(idx_opt: Option<i32>, vec_size: usize) -> Result<i32, ObjError> {
    if idx_opt.is_none() {
        return Ok(-1);
    }

    let idx = idx_opt.unwrap();
    if idx > 0 && (idx as usize) <= vec_size {
        return Ok(idx - 1);
    }

    if idx < 0 && (((vec_size as i32) + idx) >= 0) {
        return Ok((vec_size as i32) + idx);
    }

    Err(ObjError::FaceLineError(
        "Index could not be correlated with vertex",
    ))
}

pub fn parse_obj(obj_filename: &str) -> Result<ObjContents, ObjError> {
    let obj_filepath = Path::new(obj_filename);
    let mut obj_contents = ObjContents::default();

    let obj_file = match File::open(obj_filepath) {
        Ok(file) => file,
        Err(e) => return Err(ObjError::IoError(e)),
    };
    let obj_reader = BufReader::new(obj_file);
    let lines = obj_reader.lines();

    for line_result in lines {
        let line = line_result.map_err(IoError)?;
        let line = line.trim_start();

        match line.split_whitespace().next() {
            Some("v") => {
                let mut new_vertex = Vec4::default();
                new_vertex.w = 1.0;

                read_float_data(
                    line,
                    "v ",
                    3..=4,
                    &mut new_vertex,
                    ObjError::VertexLineError,
                )?;

                obj_contents.geometric_vertices.push(new_vertex);
            }
            Some("vt") => {
                let mut new_vertex = Vec3::default();
                new_vertex.z = 0.0;

                read_float_data(
                    line,
                    "vt ",
                    2..=3,
                    &mut new_vertex,
                    ObjError::TextureLineError,
                )?;

                obj_contents.texture_coords.push(new_vertex);
            }
            Some("vn") => {
                let mut new_vertex = Vec3::default();

                read_float_data(
                    line,
                    "vn ",
                    3..=3,
                    &mut new_vertex,
                    ObjError::NormalLineError,
                )?;

                obj_contents.normals.push(new_vertex);
            }
            Some("f") => {
                let face_vertices_opt = read_face_line(line)?;
                let mut face_vertices: Vec<[i32; 3]> = Vec::with_capacity(face_vertices_opt.len());

                for vertices_opt in face_vertices_opt {
                    let v_idx = associate_face_vertex(
                        vertices_opt[0],
                        obj_contents.geometric_vertices.len(),
                    )?;
                    let vt_idx =
                        associate_face_vertex(vertices_opt[1], obj_contents.texture_coords.len())?;
                    let vn_idx =
                        associate_face_vertex(vertices_opt[2], obj_contents.normals.len())?;

                    face_vertices.push([v_idx, vt_idx, vn_idx]);
                }

                obj_contents.faces.push(face_vertices);
            }
            _ => {}
        }
    }

    Ok(obj_contents)
}
