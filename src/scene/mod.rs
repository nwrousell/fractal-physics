mod lights;
mod objects;
mod tessellate;
mod vertex;

use cgmath::{EuclideanSpace, Point3};
use tessellate::Cube;
pub use vertex::Vertex;

use crate::{
    buffer::Buffer,
    scene::{
        lights::{LightUniform, Lights},
        objects::{Mesh, ObjectCollection, ObjectData, Shape},
    },
};

#[derive(Debug)]
pub struct RectangularPrism {
    /// ! this seems sus - maybe one of the issues with WFC -> Rects
    /// position of top-front-left
    pub position: Point3<f32>,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl RectangularPrism {
    pub fn new(position: Point3<f32>, width: f32, height: f32, depth: f32) -> Self {
        RectangularPrism {
            position,
            width,
            height,
            depth,
        }
    }

    pub fn to_ctm(&self) -> cgmath::Matrix4<f32> {
        let scale = cgmath::Matrix4::from_nonuniform_scale(self.width, self.height, self.depth);
        let offset = cgmath::Vector3::new(0.5 * self.width, -0.5 * self.height, -0.5 * self.depth);
        let translation = cgmath::Matrix4::from_translation(self.position.to_vec() + offset);
        translation * scale
    }
}

pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub object_collections: Vec<ObjectCollection>,
    pub lights: Lights,
}

impl Scene {
    pub fn new(tessellation_param_1: u32, rects: Vec<RectangularPrism>) -> Self {
        let mut vertices = Vec::new();
        Cube::tessellate(&mut vertices, tessellation_param_1);
        let cube_mesh = Mesh::new(0, vertices.len().try_into().unwrap());

        let mut cubes = Vec::new();
        for rect in rects {
            cubes.push(ObjectData::new(rect.to_ctm()));
        }

        let object_collections = vec![ObjectCollection::new(Shape::Cube, cubes, cube_mesh)];

        let lights = Lights::new(vec![LightUniform::new(
            [30.0, 30.0, 50000.0],
            [1.0, 1.0, 1.0],
        )]);

        Self {
            object_collections,
            vertices,
            lights,
        }
    }

    pub fn init_buffers(&mut self, device: &wgpu::Device) {
        for object_collection in &mut self.object_collections {
            object_collection.init_buffer(device);
        }
        self.lights.init_buffer(device);
    }

    pub fn vertices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}
