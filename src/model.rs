use crate::canvas::Canvas;
use crate::math::transform::Pose;
use crate::math::{self, Mat3, Vec2, Vec3};
use crate::parsers::obj::{self, ObjError};

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Bad object file")]
    InvalidObjectFile(ObjError),
    #[error("Malformed model, could not triangulate all faces")]
    TriangulationError,
}

#[derive(Default)]
pub struct Vertex {
    pos: Vec3,
    _texture: Option<Vec2>,
    _normal: Vec3,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Face {
    v: [u32; 3],
}

impl Face {
    pub fn get_vertex_positions(&self, points: &Vec<Vertex>) -> [Vec3; 3] {
        let p0 = points[self.v[0] as usize].pos.clone();
        let p1 = points[self.v[1] as usize].pos.clone();
        let p2 = points[self.v[2] as usize].pos.clone();

        [p0, p1, p2]
    }

    pub fn backfacing(&self, points: &Vec<Vertex>) -> bool {
        let [p0, p1, p2] = self.get_vertex_positions(points);

        let vec_a = p1 - p0;
        let vec_b = p2 - p0;

        let vec2_a = Vec2::new(vec_a.x, vec_a.y);
        let vec2_b = Vec2::new(vec_b.x, vec_b.y);

        vec2_a.cross(vec2_b) < 0.0
    }
}

/// Bounding box of a triangle as `(x range, y range)`, each stored as `Vec2 { x: min, y: max }`.
fn triangle_bounds(t: &[Vec2; 3]) -> (Vec2, Vec2) {
    let x_min = t[0].x.min(t[1].x.min(t[2].x));
    let x_max = t[0].x.max(t[1].x.max(t[2].x));

    let y_min = t[0].y.min(t[1].y.min(t[2].y));
    let y_max = t[0].y.max(t[1].y.max(t[2].y));

    (Vec2::new(x_min, x_max), Vec2::new(y_min, y_max))
}

/// Columns are the triangle's vertices in homogeneous form `(x, y, 1)`, so
/// `M * (w0, w1, w2) = w0*a + w1*b + w2*c` (with the weights summing to 1). Its inverse maps a
/// point back to those weights: the barycentric coordinates. Depends only on the triangle,
/// so invert it once per triangle and reuse it for every point tested against it.
fn construct_homogeneous_matrix(triangle: &[Vec2; 3]) -> Mat3 {
    Mat3::new(
        Vec3::new(triangle[0].x, triangle[0].y, 1.0),
        Vec3::new(triangle[1].x, triangle[1].y, 1.0),
        Vec3::new(triangle[2].x, triangle[2].y, 1.0),
    )
}

/// Barycentric weights of `p`, given the inverse of `construct_homogeneous_matrix`.
/// All weights >= 0 means `p` is inside (or on the edge of) the triangle.
fn barycentric(inv_homogeneous: &Mat3, p: Vec2) -> Vec3 {
    *inv_homogeneous * Vec3::new(p.x, p.y, 1.0)
}

pub struct Model {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    pose: Pose,
    scale_factor: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            vertices: Vec::default(),
            faces: Vec::default(),
            pose: Pose::default(),
            scale_factor: 1.0,
        }
    }
}

impl Model {
    pub fn load_model(&mut self, model_filename: &str) -> Result<(), ModelError> {
        let obj_contents = match obj::parse_obj(model_filename) {
            Ok(contents) => contents,
            Err(e) => return Err(ModelError::InvalidObjectFile(e)),
        };

        // Ignore texture for now
        self.vertices.reserve(obj_contents.geometric_vertices.len());
        obj_contents.geometric_vertices.iter().for_each(|gv| {
            let mut v = Vertex::default();
            v.pos = Vec3::new(gv.x, gv.y, gv.z);
            self.vertices.push(v);
        });

        self.faces.reserve(obj_contents.faces.len());
        for face in obj_contents.faces.iter() {
            if face.len() == 3 {
                let v0 = face[0][0] as u32;
                let v1 = face[1][0] as u32;
                let v2 = face[2][0] as u32;
                self.faces.push(Face { v: [v0, v1, v2] });
            } else {
                let new_faces = self.triangulate_polygon(
                    face.iter()
                        .map(|x| &self.vertices[x[0] as usize].pos)
                        .collect(),
                )?;
                let mut mapped_faces = new_faces
                    .iter()
                    .map(|f| Face {
                        v: [
                            face[f[0] as usize][0] as u32,
                            face[f[1] as usize][0] as u32,
                            face[f[2] as usize][0] as u32,
                        ],
                    })
                    .collect();
                self.faces.append(&mut mapped_faces);
            }
        }

        Ok(())
    }

    pub fn set_scale_factor(&mut self, sf: f32) {
        self.scale_factor = sf;
    }

    pub fn set_pose(&mut self, new_pose: Pose) {
        self.pose = new_pose
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.pose.pos = pos;
    }

    pub fn translate(&mut self, v: Vec3) {
        self.pose.pos += v;
    }

    /// Model space -> screen space: scale, then offset by the pose position. Screen y points
    /// down while model y points up, so y is flipped. Rotation and projection go here later.
    fn to_screen(&self, p: Vec3) -> Vec2 {
        Vec2::new(
            self.pose.pos.x + p.x * self.scale_factor,
            self.pose.pos.y - p.y * self.scale_factor,
        )
    }

    fn screen_triangle(&self, face: &Face) -> [Vec2; 3] {
        face.get_vertex_positions(&self.vertices)
            .map(|p| self.to_screen(p))
    }

    pub fn render(&self, canvas: &mut Canvas) {
        let canvas_dimensions = canvas.get_dimensions();

        for face in self.faces.iter() {
            if face.backfacing(&self.vertices) {
                continue;
            }

            let triangle = self.screen_triangle(face);

            // Degenerate (zero-area) triangles have no inverse and cover no pixels
            let Ok(inv) = construct_homogeneous_matrix(&triangle).inv() else {
                continue;
            };

            let bounds = triangle_bounds(&triangle);

            // Clamp while still signed: a bound left of / above the canvas is negative, and
            // casting a negative i32 to usize wraps to a huge number
            let min_x = (bounds.0.x.floor() as i32).max(0);
            let max_x = (bounds.0.y.ceil() as i32).min(canvas_dimensions.0 as i32 - 1);

            let min_y = (bounds.1.x.floor() as i32).max(0);
            let max_y = (bounds.1.y.ceil() as i32).min(canvas_dimensions.1 as i32 - 1);

            // Entirely off-canvas
            if min_x > max_x || min_y > max_y {
                continue;
            }
            let (min_x, max_x) = (min_x as usize, max_x as usize);
            let (min_y, max_y) = (min_y as usize, max_y as usize);

            let r: u32 = rand::random_range(0x0..0xFF);
            let g: u32 = rand::random_range(0x0..0xFF);
            let b: u32 = rand::random_range(0x0..0xFF);
            let color = (r << 16) | (g << 8) | b;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    // Sample at the pixel's center
                    let pixel = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

                    if barycentric(&inv, pixel).any(|w| w < 0.0) {
                        continue;
                    }

                    canvas.set_pixel_value(x, y, color);
                }
            }
        }
    }

    fn helper_vec(&self, x_val: f32) -> Vec3 {
        if x_val.abs() < 0.9 {
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }
        }
    }

    fn uv_transform(&self, n: Vec3) -> Mat3 {
        let helper = self.helper_vec(n[0]);
        let u = helper.cross(n);
        let u = u / u.norm();
        let v = n.cross(u);
        let v = v / v.norm();

        Mat3 {
            x_axis: u,
            y_axis: v,
            z_axis: n,
        }
        .T()
    }

    fn get_neighbors(&self, idx: usize, valid_vertices: &Vec<usize>) -> (usize, usize) {
        let prev = if idx == 0 {
            *valid_vertices.last().unwrap()
        } else {
            valid_vertices[idx - 1]
        };
        let next = if idx == (valid_vertices.len() - 1) {
            valid_vertices[0]
        } else {
            valid_vertices[idx + 1]
        };

        (prev, next)
    }

    /// Triangulates a simple polygon with counter-clockwise winding. Returned triangles index
    /// into `polygon` and keep its winding.
    fn ear_clipping_algorithm(&self, polygon: &Vec<Vec2>) -> Result<Vec<[u32; 3]>, ModelError> {
        let mut valid_idxs: Vec<usize> = (0..polygon.len()).collect();
        let mut triangles: Vec<[u32; 3]> = Vec::with_capacity(polygon.len() - 2);

        let mut idx: usize = 0;
        let mut vertices_since_last_triangle = 0;
        while valid_idxs.len() > 3 {
            // Went all the way around without finding an ear
            if vertices_since_last_triangle >= valid_idxs.len() {
                return Err(ModelError::TriangulationError);
            }

            // Find the neighbors of the current index
            let (prev, next) = self.get_neighbors(idx, &valid_idxs);
            let current = valid_idxs[idx];
            let triangle = [polygon[prev], polygon[current], polygon[next]];

            if self.is_ear(&triangle, [prev, current, next], &valid_idxs, polygon) {
                triangles.push([prev as u32, current as u32, next as u32]);
                valid_idxs.remove(idx);
                // `idx` now points at what was `next`; wrap if we removed the last element
                if idx >= valid_idxs.len() {
                    idx = 0;
                }
                vertices_since_last_triangle = 0;
            } else {
                idx = (idx + 1) % valid_idxs.len();
                vertices_since_last_triangle += 1;
            }
        }

        triangles.push([
            valid_idxs[0] as u32,
            valid_idxs[1] as u32,
            valid_idxs[2] as u32,
        ]);

        Ok(triangles)
    }

    /// `triangle` is `[prev, current, next]`; it's an ear if the corner at `current` is convex
    /// and no other remaining vertex lies inside it.
    fn is_ear(
        &self,
        triangle: &[Vec2; 3],
        corner: [usize; 3],
        valid_idxs: &[usize],
        polygon: &[Vec2],
    ) -> bool {
        // Convex (left turn) for counter-clockwise winding; also rejects collinear corners
        let a = triangle[1] - triangle[0];
        let b = triangle[2] - triangle[1];
        if a.cross(b) <= 0.0 {
            return false;
        }

        let Ok(inv) = construct_homogeneous_matrix(triangle).inv() else {
            return false;
        };

        valid_idxs
            .iter()
            .filter(|i| !corner.contains(i))
            .all(|&i| barycentric(&inv, polygon[i]).any(|w| w < 0.0))
    }

    fn triangulate_polygon(&self, polygon: Vec<&Vec3>) -> Result<Vec<[u32; 3]>, ModelError> {
        let n = math::transform::newell_normal(&polygon);
        let m_rot = self.uv_transform(n);

        let poly_t: Vec<Vec2> = polygon
            .iter()
            .map(|p| {
                let projection = **p - (p.dot(n) * n);
                let transformed = m_rot * projection;

                Vec2 {
                    x: transformed.x,
                    y: transformed.y,
                }
            })
            .collect();

        Ok(self.ear_clipping_algorithm(&poly_t)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Model;
    use crate::canvas::{BlendMode, Canvas};
    use crate::math::{Vec2, Vec3};

    fn signed_area(t: [Vec2; 3]) -> f32 {
        (t[1] - t[0]).cross(t[2] - t[0]) / 2.0
    }

    #[test]
    fn ear_clipping_concave_polygon() {
        // Counter-clockwise "arrow" with a reflex (concave) vertex at index 3
        let polygon = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(0.0, 4.0),
        ];
        let tris = Model::default().ear_clipping_algorithm(&polygon).unwrap();
        assert_eq!(tris.len(), polygon.len() - 2);

        // Every triangle keeps the polygon's CCW winding, and together they cover its area
        let mut total = 0.0;
        for t in &tris {
            let area = signed_area(t.map(|i| polygon[i as usize]));
            assert!(area > 0.0, "triangle {t:?} is clockwise or degenerate");
            total += area;
        }
        // Shoelace area of the polygon: 16 - triangle cut out (4 * 3 / 2) = 10
        assert!((total - 10.0).abs() < 1e-5, "covered area {total}");
    }

    #[test]
    fn box_quads_become_twelve_triangles() {
        let mut model = Model::default();
        model.load_model("assets/models/box.obj").unwrap();
        assert_eq!(model.faces.len(), 12);
    }

    #[test]
    fn render_draws_scaled_model_at_its_position() {
        let mut model = Model::default();
        model.load_model("assets/models/box.obj").unwrap();
        model.set_scale_factor(40.0);
        model.set_position(Vec3::new(50.0, 50.0, 0.0));

        let mut canvas = Canvas::new(0, 0, 100, 100, BlendMode::Opaque);
        model.render(&mut canvas);

        // The box spans +/-0.5 * 40 = +/-20 px around (50, 50)
        assert_ne!(canvas.get_pixel_value(50, 50), 0, "center not drawn");
        assert_ne!(canvas.get_pixel_value(32, 68), 0, "near corner not drawn");
        assert_eq!(canvas.get_pixel_value(5, 5), 0, "drew outside the box");
        assert_eq!(canvas.get_pixel_value(95, 50), 0, "drew outside the box");
    }
}
