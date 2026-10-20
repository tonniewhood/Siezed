//! Plain geometry data (vertices, meshes) shared by the parsers and the rasterizer.
//! Kept separate from `raster` so that parsing a mesh doesn't depend on drawing it.
use std::default::Default;
use std::io;
use std::ops::RangeInclusive;
use std::path::Path;

use crate::canvas::Canvas;
use crate::math::{Mat3, Vec2, Vec3, matrix};
use crate::parsers::mesh;

#[derive(Clone, Copy)]
pub enum DrawType {
    Wire,
    Fill,
}

#[derive(Default, Clone, Copy)]
pub struct Triangle {
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    color: u32,
    x_bounds: (i32, i32),
    y_bounds: (i32, i32),
}

impl Triangle {
    pub fn sumarize(&self) {
        println!("P0: ({}, {})", self.p0.x, self.p0.y);
        println!("P1: ({}, {})", self.p1.x, self.p1.y);
        println!("P2: ({}, {})", self.p2.x, self.p2.y);
        println!("Color: #{:X}", self.color)
    }

    pub fn dx(&self) -> u32 {
        (self.x_bounds.1 - self.x_bounds.0).abs() as u32
    }

    pub fn dy(&self) -> u32 {
        (self.y_bounds.1 - self.y_bounds.0).abs() as u32
    }

    pub fn x_range(&self) -> RangeInclusive<i32> {
        (self.x_bounds.0)..=(self.x_bounds.1)
    }

    pub fn y_range(&self) -> RangeInclusive<i32> {
        (self.y_bounds.0)..=(self.y_bounds.1)
    }

    pub fn update_bounds(&mut self) {
        let x_min = self.p0.x.min(self.p1.x.min(self.p2.x)).round() as i32;
        let x_max = self.p0.x.max(self.p1.x.max(self.p2.x)).round() as i32;
        self.x_bounds = (x_min, x_max);

        let y_min = self.p0.y.min(self.p1.y.min(self.p2.y)).round() as i32;
        let y_max = self.p0.y.max(self.p1.y.max(self.p2.y)).round() as i32;
        self.y_bounds = (y_min, y_max);
    }
}

impl mesh::Polygon for Triangle {
    fn from_values(a: Vec2, b: Vec2, c: Vec2, color: u32) -> Triangle {
        let x_min = a.x.min(b.x.min(c.x)).round() as i32;
        let x_max = a.x.max(b.x.max(c.x)).round() as i32;
        let y_min = a.y.min(b.y.min(c.y)).round() as i32;
        let y_max = a.y.max(b.y.max(c.y)).round() as i32;

        Triangle {
            p0: a,
            p1: b,
            p2: c,
            color: color,
            x_bounds: (x_min, x_max),
            y_bounds: (y_min, y_max),
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    triangles: Vec<Triangle>,
    visible_bounds: [Vec2; 2],
}

impl Mesh {
    pub fn from_path(mesh_path: &Path) -> io::Result<Mesh> {
        let mut triangles: Vec<Triangle> = Vec::new();
        match mesh::parse_2dv(mesh_path, &mut triangles) {
            Ok(_) => Ok(Mesh {
                triangles: triangles,
                visible_bounds: [Vec2::ZERO, Vec2::ZERO],
            }),
            Err(err) => Err(err),
        }
    }

    pub fn load_mesh(&mut self, mesh_path: &Path) -> io::Result<()> {
        mesh::parse_2dv(mesh_path, self.triangles.as_mut())
    }

    pub fn set_bounds(&mut self, bounds: (u32, u32)) {
        self.visible_bounds = [
            Vec2 {
                x: 0.0,
                y: bounds.0 as f32,
            },
            Vec2 {
                x: 0.0,
                y: bounds.1 as f32,
            },
        ];
    }

    pub fn triangles(&self) -> &Vec<Triangle> {
        &self.triangles
    }

    pub fn triangles_mut(&mut self) -> &mut Vec<Triangle> {
        &mut self.triangles
    }

    // Kind of going to be bastardized
    // In the future will probably be a kind of affine pair
    pub fn transform(&mut self, transforms: &mut [&mut Vec2], step: f32) {
        // println!(
        //     "t1: {:?}, t2: {:?}, t3: {:?}",
        //     transforms[0], transforms[1], transforms[2]
        // );
        let tri = &mut self.triangles[0];
        for (p, t) in [&mut tri.p0, &mut tri.p1, &mut tri.p2]
            .into_iter()
            .zip(transforms)
        {
            p.foreach_mut(|idx, x| {
                let mut new_x = *x + t[idx] * step;
                if new_x < (self.visible_bounds[idx].x) || (self.visible_bounds[idx].y) < new_x {
                    new_x = *x - t[idx] * step;
                    t[idx] = -t[idx];
                }

                *x = new_x;

                // println!("new_x: {}", *x);
            });
        }

        self.triangles[0].update_bounds();
    }

    fn point_in_dimenstions(&self, p: &Vec2) -> bool {
        (self.visible_bounds[0].x <= p.x && p.x < self.visible_bounds[0].y)
            && (self.visible_bounds[1].x <= p.y && p.y < self.visible_bounds[1].y)
    }

    fn draw_line(&self, p0: &Vec2, p1: &Vec2, color: u32, canvas: &mut Canvas) {
        let delta = *p1 - *p0;
        let steps = delta.x.abs().max(delta.y.abs()).round() as usize;
        let step = if steps == 0 {
            Vec2::ZERO
        } else {
            delta / steps as f32
        };

        for i in 0..=steps {
            let p = *p0 + step * i as f32;
            let pixel = Vec2::new(p.x.round(), p.y.round());

            if self.point_in_dimenstions(&pixel) {
                canvas.set_pixel_value(pixel.x as usize, pixel.y as usize, color);
            }
        }
    }

    fn draw_wire_frame(&self, canvas: &mut Canvas) {
        for triangle in self.triangles.iter() {
            self.draw_line(&triangle.p0, &triangle.p1, triangle.color, canvas);
            self.draw_line(&triangle.p0, &triangle.p2, triangle.color, canvas);
            self.draw_line(&triangle.p1, &triangle.p2, triangle.color, canvas);
        }
    }

    fn on_segment(&self, p: &Vec2, a: &Vec2, b: &Vec2) -> bool {
        // Cross product for aread of parallelogram
        let area = (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x);

        // If the area is non-zero, there exists a paralellogram, so not on
        if area.abs() > f32::EPSILON {
            return false;
        }

        // Validate that the bounds fit
        (a.x.min(b.x) <= p.x && p.x <= a.x.max(b.x)) && (a.y.min(b.y) <= p.y && p.y <= a.y.max(b.y))
    }

    fn point_intersects(&self, p: &Vec2, a: &Vec2, b: &Vec2) -> bool {
        let height_bounded = (a.y > p.y) != (b.y > p.y);
        if !height_bounded {
            return false;
        }

        if (b.y - a.y).abs() < f32::EPSILON {
            println!("A: ({}, {}), B: ({}, {})", a.x, a.y, b.x, b.y);
        }

        let projection_hits = p.x < ((b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x);

        projection_hits && height_bounded
    }

    fn compute_barycenter(
        &self,
        p: &Vec2,
        triangle: &Triangle,
    ) -> Result<Vec3, matrix::MatrixError> {
        let homogeneous_matrix = Mat3 {
            x_axis: Vec3 {
                x: triangle.p0.x,
                y: triangle.p0.y,
                z: 1.0,
            },
            y_axis: Vec3 {
                x: triangle.p1.x,
                y: triangle.p1.y,
                z: 1.0,
            },
            z_axis: Vec3 {
                x: triangle.p2.x,
                y: triangle.p2.y,
                z: 1.0,
            },
        };

        let homogenous_point = Vec3 {
            x: p.x,
            y: p.y,
            z: 1.0,
        };

        match homogeneous_matrix.inv() {
            Ok(mat) => Ok(mat * homogenous_point),
            Err(err) => Err(err),
        }
    }

    fn fill_triangles(&self, canvas: &mut Canvas) {
        for triangle in self.triangles.iter() {
            for y in triangle.y_range() {
                for x in triangle.x_range() {
                    let p = Vec2 {
                        x: x as f32,
                        y: y as f32,
                    };

                    // Yes it's lazy
                    let barycenter = self.compute_barycenter(&p, triangle).unwrap();

                    if barycenter.all(|x| x >= 0.0) && self.point_in_dimenstions(&p) {
                        let r = (255.0 * barycenter.x).round() as u32;
                        let g = (255.0 * barycenter.y).round() as u32;
                        let b = (255.0 * barycenter.z).round() as u32;
                        let color = (r << 16) | (g << 8) | b;
                        canvas.set_pixel_value(x as usize, y as usize, color);
                    }
                }
            }
        }
    }

    pub fn draw_2d_shape(&self, canvas: &mut Canvas, draw_type: DrawType) {
        match draw_type {
            DrawType::Wire => self.draw_wire_frame(canvas),
            DrawType::Fill => self.fill_triangles(canvas),
        }
    }
}
