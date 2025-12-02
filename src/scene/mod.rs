mod tessellate;
mod types;
mod vertex;

use std::{collections::HashMap, ops::Range};

use tessellate::Cube;
use types::Mesh;
pub use types::{RectangularPrism, Scene, Shape};
pub use vertex::Vertex;

use crate::scene::types::{LightUniform, ObjectData};

impl Scene {
    pub fn new(tessellation_param_1: u32, rects: Vec<RectangularPrism>) -> Self {
        // tessellate shapes
        let mut vertices = Vec::new();
        let mut meshes = HashMap::new();

        Cube::tessellate(&mut vertices, tessellation_param_1);
        meshes.insert(Shape::Cube, Mesh::new(0, vertices.len() as u64));

        // populate objects
        let mut objects = HashMap::new();
        let mut cubes = Vec::new();
        for rect in rects {
            cubes.push(ObjectData::new(rect.to_ctm()));
        }
        objects.insert(Shape::Cube, cubes);

        // lights
        let light = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        Self {
            objects,
            vertices,
            meshes,
            light,
        }
    }

    pub fn vertices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn ctms(&self, shape: Shape) -> &[u8] {
        bytemuck::cast_slice(&self.objects[&shape])
    }

    pub fn shape_ranges(&self, shape: &Shape) -> (Range<u32>, Range<u32>) {
        let mesh = &self.meshes[shape];
        let vertex_range =
            (mesh.vertex_offset as u32)..((mesh.vertex_offset + mesh.num_vertices) as u32);
        let instance_range = 0..(self.objects[shape].len() as u32);

        (vertex_range, instance_range)
    }

    pub fn light_buffer(&self) -> &[u8] {
        bytemuck::bytes_of(&self.light)
    }
}
