use crate::scene::Vertex;
use cgmath::{Point3, Vector3, prelude::*};

/// The 6 faces of a cube, each can be tessellated and instanced separately
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Front,  // +Z
    Back,   // -Z
    Top,    // +Y
    Bottom, // -Y
    Right,  // +X
    Left,   // -X
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Front,
        Face::Back,
        Face::Top,
        Face::Bottom,
        Face::Right,
        Face::Left,
    ];

    /// Returns the face points (top_left, top_right, bottom_left) for tessellation
    fn face_points(&self) -> (Point3<f32>, Point3<f32>, Point3<f32>) {
        match self {
            Face::Front => (
                Point3::new(-0.5, 0.5, 0.5),
                Point3::new(0.5, 0.5, 0.5),
                Point3::new(-0.5, -0.5, 0.5),
            ),
            Face::Top => (
                Point3::new(0.5, 0.5, 0.5),
                Point3::new(-0.5, 0.5, 0.5),
                Point3::new(0.5, 0.5, -0.5),
            ),
            Face::Back => (
                Point3::new(0.5, 0.5, -0.5),
                Point3::new(-0.5, 0.5, -0.5),
                Point3::new(0.5, -0.5, -0.5),
            ),
            Face::Bottom => (
                Point3::new(0.5, -0.5, -0.5),
                Point3::new(-0.5, -0.5, -0.5),
                Point3::new(0.5, -0.5, 0.5),
            ),
            Face::Right => (
                Point3::new(0.5, 0.5, 0.5),
                Point3::new(0.5, 0.5, -0.5),
                Point3::new(0.5, -0.5, 0.5),
            ),
            Face::Left => (
                Point3::new(-0.5, 0.5, -0.5),
                Point3::new(-0.5, 0.5, 0.5),
                Point3::new(-0.5, -0.5, -0.5),
            ),
        }
    }

    fn face_points_prism(&self, width: f32, height: f32, depth: f32) -> (Point3<f32>, Point3<f32>, Point3<f32>) {
        match self {
            Face::Front => (
                Point3::new(-width / 2.0, height / 2.0, depth / 2.0),  // Top left
                Point3::new(width / 2.0, height / 2.0, depth / 2.0),   // Top right
                Point3::new(-width / 2.0, -height / 2.0, depth / 2.0), // Bottom left
            ),
            Face::Top => (
                Point3::new(width / 2.0, height / 2.0, depth / 2.0),   // Top right
                Point3::new(-width / 2.0, height / 2.0, depth / 2.0),  // Top left
                Point3::new(width / 2.0, height / 2.0, -depth / 2.0),  // Bottom right
            ),
            Face::Back => (
                Point3::new(width / 2.0, height / 2.0, -depth / 2.0),  // Top right
                Point3::new(-width / 2.0, height / 2.0, -depth / 2.0), // Top left
                Point3::new(width / 2.0, -height / 2.0, -depth / 2.0),// Bottom right
            ),
            Face::Bottom => (
                Point3::new(width / 2.0, -height / 2.0, -depth / 2.0), // Top right
                Point3::new(-width / 2.0, -height / 2.0, -depth / 2.0),// Top left
                Point3::new(width / 2.0, -height / 2.0, depth / 2.0),  // Bottom right
            ),
            Face::Right => (
                Point3::new(width / 2.0, height / 2.0, depth / 2.0),   // Top right
                Point3::new(width / 2.0, height / 2.0, -depth / 2.0),  // Top left
                Point3::new(width / 2.0, -height / 2.0, depth / 2.0),  // Bottom right
            ),
            Face::Left => (
                Point3::new(-width / 2.0, height / 2.0, -depth / 2.0), // Top left
                Point3::new(-width / 2.0, height / 2.0, depth / 2.0),  // Top right
                Point3::new(-width / 2.0, -height / 2.0, -depth / 2.0),// Bottom left
            ),
        }
    }

    /// Tessellate this face into vertices
    pub fn tessellate(&self, vertices: &mut Vec<Vertex>, tessellation_param: u32) {
        let (top_left, top_right, bottom_left) = self.face_points();
        make_face(
            vertices,
            tessellation_param,
            top_left,
            top_right,
            bottom_left,
        );
    }

    // pub fn tessellate_prism(&self, vertices: &mut Vec<Vertex>, tessellation_param: u32) {
    //     let (top_left, top_right, bottom_left) = self.face_points_prism(width, height, depth);
    //     make_face(
    //         vertices,
    //         tessellation_param,
    //         top_left,
    //         top_right,
    //         bottom_left,
    //     );
    // }
}

fn insert_vertex(vertices: &mut Vec<Vertex>, point: Point3<f32>, normal: Vector3<f32>) {
    vertices.push(Vertex::new(
        point.x, point.y, point.z, normal.x, normal.y, normal.z,
    ));
}

fn make_tile(
    vertices: &mut Vec<Vertex>,
    top_left: Point3<f32>,
    top_right: Point3<f32>,
    bottom_left: Point3<f32>,
    bottom_right: Point3<f32>,
) {
    let normal = Vector3::cross(top_right - top_left, bottom_left - top_left).normalize();

    // triangle 1
    insert_vertex(vertices, top_left, normal);
    insert_vertex(vertices, bottom_left, normal);
    insert_vertex(vertices, bottom_right, normal);

    // triangle 2
    insert_vertex(vertices, bottom_right, normal);
    insert_vertex(vertices, top_right, normal);
    insert_vertex(vertices, top_left, normal);
}

fn make_face(
    vertices: &mut Vec<Vertex>,
    tessellation_param: u32,
    top_left: Point3<f32>,
    top_right: Point3<f32>,
    bottom_left: Point3<f32>,
) {
    for i in 0..tessellation_param {
        for j in 0..tessellation_param {
            let h_percent = (i as f32) / (tessellation_param as f32);
            let v_percent = (j as f32) / (tessellation_param as f32);

            let tile_top_left = top_left
                + h_percent * (top_right - top_left)
                + v_percent * (bottom_left - top_left);

            let scalar = 1f32 / (tessellation_param as f32);
            make_tile(
                vertices,
                tile_top_left,
                tile_top_left + scalar * (top_right - top_left),
                tile_top_left + scalar * (bottom_left - top_left),
                tile_top_left + scalar * (top_right - top_left) + scalar * (bottom_left - top_left),
            );
        }
    }
}

pub fn tessellate_cube(vertices: &mut Vec<Vertex>, tessellation_param: u32) {
    for face in Face::ALL {
        face.tessellate(vertices, tessellation_param);
    }
}

// pub fn tessellate_rectangular_prism(vertices: &mut Vec<Vertex>, tessellation_param: u32, width: f32, height: f32, depth: f32) {
//     for face in Face::ALL {
//         face.tessellate(vertices, tessellation_param, width, height, depth);
//     }
// }