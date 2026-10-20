//! Plain geometry data (vertices, meshes) shared by the parsers and the rasterizer.
//! Kept separate from `raster` so that parsing a mesh doesn't depend on drawing it.
use std::default::Default;
use std::io;
use std::ops::RangeInclusive;
use std::path::Path;

use crate::canvas::Canvas;
use crate::parsers::mesh;

#[derive(Clone, Copy)]
pub enum DrawType {
    Wire,
    Fill,
}

#[derive(Default)]
pub struct Vertex {
    x: i32,
    y: i32,
}

#[derive(Default)]
pub struct Triangle {
    p0: Vertex,
    p1: Vertex,
    p2: Vertex,
    color: u32,
    x_bounds: (i32, i32),
    y_bounds: (i32, i32),
}

impl Vertex {
    pub fn from_pair(pair: mesh::Pair) -> Vertex {
        Vertex {
            x: pair.0,
            y: pair.1,
        }
    }
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
}

impl mesh::Polygon for Triangle {
    fn from_values(x: mesh::Pair, y: mesh::Pair, z: mesh::Pair, color: u32) -> Triangle {
        let x_min = x.0.min(y.0.min(z.0));
        let x_max = x.0.max(y.0.max(z.0));
        let y_min = x.1.min(y.1.min(z.1));
        let y_max = x.1.max(y.1.max(z.1));

        Triangle {
            p0: Vertex::from_pair(x),
            p1: Vertex::from_pair(y),
            p2: Vertex::from_pair(z),
            color: color,
            x_bounds: (x_min, x_max),
            y_bounds: (y_min, y_max),
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn from_path(mesh_path: &Path) -> io::Result<Mesh> {
        let mut triangles: Vec<Triangle> = Vec::new();
        match mesh::parse_2dv(mesh_path, &mut triangles) {
            Ok(_) => Ok(Mesh { triangles }),
            Err(err) => Err(err),
        }
    }

    pub fn load_mesh(&mut self, mesh_path: &Path) -> io::Result<()> {
        mesh::parse_2dv(mesh_path, self.triangles.as_mut())
    }

    pub fn triangles(&self) -> &Vec<Triangle> {
        &self.triangles
    }

    fn point_in_dimenstions(&self, p: &Vertex, dims: &(usize, usize)) -> bool {
        (0 <= p.x && (p.x as usize) < dims.0) && (0 <= p.y && (p.y as usize) < dims.1)
    }

    fn draw_horizontal(&self, p0: &Vertex, p1: &Vertex, color: u32, canvas: &mut Canvas) {
        let dx = p1.x - p0.x;
        let mut dy = p1.y - p0.y;
        let canvas_dims = canvas.get_dimensions();

        let dir = if dy < 0 { -1 } else { 1 };
        dy *= dir;

        if dx != 0 {
            let mut y = p0.y;
            let mut err = 2 * dy - dx;

            for idx in 0..(dx + 1) {
                let x = p0.x + idx;
                if (0 <= x && (x as usize) < canvas_dims.0)
                    && (0 <= y && (y as usize) < canvas_dims.1)
                {
                    canvas.set_pixel_value(x as usize, y as usize, color);
                }

                if err >= 0 {
                    y += dir;
                    err -= 2 * dx;
                }
                err += 2 * dy;
            }
        }
    }

    fn draw_vertical(&self, p0: &Vertex, p1: &Vertex, color: u32, canvas: &mut Canvas) {
        let mut dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let canvas_dims = canvas.get_dimensions();

        let dir = if dx < 0 { -1 } else { 1 };
        dx *= dir;

        if dy != 0 {
            let mut x = p0.x;
            let mut err = 2 * dx - dy;

            for idx in 0..(dy + 1) {
                let y = p0.y + idx;
                let p = Vertex::from_pair((x, y));
                if self.point_in_dimenstions(&p, &canvas_dims) {
                    canvas.set_pixel_value(x as usize, y as usize, color);
                }

                if err >= 0 {
                    x += dir;
                    err -= 2 * dy;
                }
                err += 2 * dx;
            }
        }
    }

    fn draw_line(&self, p0: &Vertex, p1: &Vertex, color: u32, canvas: &mut Canvas) {
        if (p1.x - p0.x).abs() > (p1.y - p0.y).abs() {
            if p0.x <= p1.x {
                self.draw_horizontal(p0, p1, color, canvas);
            } else {
                self.draw_horizontal(p1, p0, color, canvas);
            }
        } else {
            if p0.y <= p1.y {
                self.draw_vertical(p0, p1, color, canvas);
            } else {
                self.draw_vertical(p1, p0, color, canvas);
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

    fn on_segment(&self, p: &Vertex, a: &Vertex, b: &Vertex) -> bool {
        // Cross product for aread of parallelogram
        let area = (p.x - a.x) * (b.y - a.y) - (p.y - a.y) * (b.x - a.x);

        // If the area is non-zero, there exists a paralellogram, so not on
        if area != 0 {
            return false;
        }

        // Validate that the bounds fit
        (a.x.min(b.x) <= p.x && p.x <= a.x.max(b.x)) && (a.y.min(b.y) <= p.y && p.y <= a.y.max(b.y))
    }

    fn point_intersects(&self, p: &Vertex, a: &Vertex, b: &Vertex) -> bool {
        let height_bounded = (a.y > p.y) != (b.y > p.y);
        if !height_bounded {
            return false;
        }

        if (b.y - a.y) == 0 {
            println!("A: ({}, {}), B: ({}, {})", a.x, a.y, b.x, b.y);
        }

        let projection_hits = p.x < ((b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x);

        projection_hits && height_bounded
    }

    fn is_inside_triangle(&self, p: &Vertex, triangle: &Triangle) -> bool {
        if self.on_segment(&p, &triangle.p0, &triangle.p1)
            || self.on_segment(&p, &triangle.p0, &triangle.p2)
            || self.on_segment(&p, &triangle.p1, &triangle.p2)
        {
            return true;
        }

        let mut inside = false;
        if self.point_intersects(&p, &triangle.p0, &triangle.p1) {
            inside = !inside;
        }

        if self.point_intersects(&p, &triangle.p0, &triangle.p2) {
            inside = !inside;
        }

        if self.point_intersects(&p, &triangle.p1, &triangle.p2) {
            inside = !inside;
        }

        inside
    }

    fn fill_triangles(&self, canvas: &mut Canvas) {
        let canvas_dims = canvas.get_dimensions();

        for triangle in self.triangles.iter() {
            for y in triangle.y_range() {
                for x in triangle.x_range() {
                    let p = Vertex::from_pair((x, y));

                    if self.is_inside_triangle(&p, &triangle)
                        && self.point_in_dimenstions(&p, &canvas_dims)
                    {
                        canvas.set_pixel_value(x as usize, y as usize, triangle.color);
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
