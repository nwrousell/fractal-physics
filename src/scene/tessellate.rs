use crate::scene::Vertex;
use cgmath::{Point3, Vector3, prelude::*};

pub struct Cube {}

impl Cube {
    pub fn tessellate(vertices: &mut Vec<Vertex>, tessellation_param_1: u32) {
        // Front face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(-0.5, 0.5, 0.5),
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(-0.5, -0.5, 0.5),
        );

        // Top face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(-0.5, 0.5, 0.5),
            Point3::new(0.5, 0.5, -0.5),
        );

        // Back face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(0.5, 0.5, -0.5),
            Point3::new(-0.5, 0.5, -0.5),
            Point3::new(0.5, -0.5, -0.5),
        );

        // Bottom face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(0.5, -0.5, -0.5),
            Point3::new(-0.5, -0.5, -0.5),
            Point3::new(0.5, -0.5, 0.5),
        );

        // Right face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(0.5, 0.5, 0.5),
            Point3::new(0.5, 0.5, -0.5),
            Point3::new(0.5, -0.5, 0.5),
        );

        // Left face
        Cube::make_face(
            vertices,
            tessellation_param_1,
            Point3::new(-0.5, 0.5, -0.5),
            Point3::new(-0.5, 0.5, 0.5),
            Point3::new(-0.5, -0.5, -0.5),
        );
    }

    fn insert_vertex(vertices: &mut Vec<Vertex>, point: Point3<f32>, normal: Vector3<f32>) {
        vertices.push(Vertex::new(
            point.x, point.y, point.z, normal.x, normal.y, normal.z, 0f32, 0f32,
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
        Cube::insert_vertex(vertices, top_left, normal);
        Cube::insert_vertex(vertices, bottom_left, normal);
        Cube::insert_vertex(vertices, bottom_right, normal);

        // triangle 2
        Cube::insert_vertex(vertices, bottom_right, normal);
        Cube::insert_vertex(vertices, top_right, normal);
        Cube::insert_vertex(vertices, top_left, normal);
    }

    fn make_face(
        vertices: &mut Vec<Vertex>,
        tessellation_param_1: u32,
        top_left: Point3<f32>,
        top_right: Point3<f32>,
        bottom_left: Point3<f32>,
    ) {
        for i in 0..tessellation_param_1 {
            for j in 0..tessellation_param_1 {
                let h_percent = (i as f32) / (tessellation_param_1 as f32);
                let v_percent = (j as f32) / (tessellation_param_1 as f32);

                let tile_top_left = top_left
                    + h_percent * (top_right - top_left)
                    + v_percent * (bottom_left - top_left);

                let scalar = 1f32 / (tessellation_param_1 as f32);
                Cube::make_tile(
                    vertices,
                    tile_top_left,
                    tile_top_left + scalar * (top_right - top_left),
                    tile_top_left + scalar * (bottom_left - top_left),
                    tile_top_left
                        + scalar * (top_right - top_left)
                        + scalar * (bottom_left - top_left),
                );
            }
        }
    }
}
