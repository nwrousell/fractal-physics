mod lights;
mod objects;
mod player;
mod tessellate;
mod vertex;

use std::collections::HashSet;

use cgmath::{EuclideanSpace, Point3};
use tessellate::Face;
pub use vertex::Vertex;

use crate::{
    buffer::Buffer,
    scene::{
        lights::{LightUniform, Lights},
        objects::{Mesh, ObjectCollection, ObjectData, Shape},
        player::Player,
    },
};

/// A voxel position in the grid (integer coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl VoxelPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Get the neighbor position in the direction of the given face
    pub fn neighbor(&self, face: Face) -> VoxelPos {
        match face {
            Face::Front => VoxelPos::new(self.x, self.y, self.z + 1),
            Face::Back => VoxelPos::new(self.x, self.y, self.z - 1),
            Face::Top => VoxelPos::new(self.x, self.y + 1, self.z),
            Face::Bottom => VoxelPos::new(self.x, self.y - 1, self.z),
            Face::Right => VoxelPos::new(self.x + 1, self.y, self.z),
            Face::Left => VoxelPos::new(self.x - 1, self.y, self.z),
        }
    }
}

/// A voxel with position and color
#[derive(Debug, Clone)]
pub struct Voxel {
    pub pos: VoxelPos,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub color: [f32; 4],
}

impl Voxel {
    pub fn new(pos: VoxelPos, width: f32, height: f32, depth: f32, color: [f32; 4]) -> Self {
        Self {
            pos,
            width,
            height,
            depth,
            color,
        }
    }

    pub fn to_ctm(&self) -> cgmath::Matrix4<f32> {
        let scale = cgmath::Matrix4::from_nonuniform_scale(self.width, self.height, self.depth);
        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            self.pos.x as f32,
            self.pos.y as f32,
            self.pos.z as f32,
        ));
        translation * scale
    }
}

pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub object_collections: Vec<ObjectCollection>,
    pub lights: Lights,

    pub player: Player,
}

impl Scene {
    pub fn new(tessellation_param: u32, voxels: Vec<Voxel>) -> Self {
        let mut vertices = Vec::new();

        // Tessellate each face type and record its mesh
        let mut face_meshes: Vec<Mesh> = Vec::new();
        for face in Face::ALL {
            let start = vertices.len() as u32;
            face.tessellate(&mut vertices, tessellation_param);
            let count = vertices.len() as u32 - start;
            face_meshes.push(Mesh::new(start, count));
        }

        // Build occupancy set from voxels for face culling
        let occupied: HashSet<VoxelPos> = voxels.iter().map(|v| v.pos).collect();

        // Generate face instances for each face type
        let mut face_instances: [Vec<ObjectData>; 6] = Default::default();

        for voxel in &voxels {
            let ctm = voxel.to_ctm();
            let color = voxel.color;

            for (i, face) in Face::ALL.iter().enumerate() {
                let neighbor = voxel.pos.neighbor(*face);
                // Only create face instance if neighbor is empty
                if !occupied.contains(&neighbor) {
                    face_instances[i].push(ObjectData::new(ctm, color));
                }
            }
        }

        // Create object collections for each face type
        let mut object_collections = Vec::new();
        for (i, instances) in face_instances.into_iter().enumerate() {
            if !instances.is_empty() {
                object_collections.push(ObjectCollection::new(
                    Shape::Face(Face::ALL[i]),
                    instances,
                    face_meshes[i].clone(),
                ));
            }
        }

        // create player
        // TODO
        let player = Player::new();

        let lights = Lights::new(vec![LightUniform::new(
            [30.0, 30.0, 50000.0],
            [1.0, 1.0, 1.0],
        )]);

        Self {
            object_collections,
            vertices,
            lights,
            player,
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
