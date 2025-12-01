use std::collections::HashMap;

use cgmath::SquareMatrix;

use crate::scene::Vertex;

/// Represents a point light
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Shape {
    Cube,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectData {
    pub ctm: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
}

impl ObjectData {
    pub fn new(ctm: cgmath::Matrix4<f32>) -> Self {
        let normal_matrix =
            cgmath::Matrix3::from_cols(ctm.x.truncate(), ctm.y.truncate(), ctm.z.truncate())
                .invert()
                .unwrap();

        let normal_matrix_padded = [
            [normal_matrix.x.x, normal_matrix.x.y, normal_matrix.x.z, 0.0],
            [normal_matrix.y.x, normal_matrix.y.y, normal_matrix.y.z, 0.0],
            [normal_matrix.z.x, normal_matrix.z.y, normal_matrix.z.z, 0.0],
        ];

        ObjectData {
            ctm: ctm.into(),
            normal_matrix: normal_matrix_padded.into(),
        }
    }
}

pub struct Mesh {
    pub vertex_offset: u64,
    pub num_vertices: u64,
}

impl Mesh {
    pub fn new(vertex_offset: u64, num_vertices: u64) -> Self {
        Mesh {
            vertex_offset,
            num_vertices,
        }
    }
}

pub struct Scene {
    pub objects: HashMap<Shape, Vec<ObjectData>>,
    pub light: LightUniform,

    pub vertices: Vec<Vertex>,
    pub meshes: HashMap<Shape, Mesh>,
}
