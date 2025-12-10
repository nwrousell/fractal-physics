mod lights;
mod objects;
mod player;
mod tessellate;
mod vertex;

use std::collections::HashSet;

use cgmath::{InnerSpace, Vector3};
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

pub struct AABB {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

pub struct Scene {
    pub vertices: Vec<Vertex>,
    pub object_collections: Vec<ObjectCollection>,
    pub lights: Lights,
    pub obstacles: Vec<AABB>,
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

        let mut obstacles = Vec::new();

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
            // let min = Vector3::new(
            //     voxel.pos.x as f32,
            //     voxel.pos.y as f32,
            //     voxel.pos.z as f32,
            // );
            // print!("Voxel at {:?} width {:?} height {:?} depth {:?}\n", voxel.pos, voxel.width , voxel.height, voxel.depth);

            // let max = min + Vector3::new(voxel.width, voxel.height, voxel.depth);
            let vox_pos_f = Vector3::new(
                voxel.pos.x as f32,
                voxel.pos.y as f32,
                voxel.pos.z as f32,
            );

            let min = vox_pos_f - Vector3::new(voxel.width / 2.0, voxel.height / 2.0, voxel.depth / 2.0);
            let max = vox_pos_f + Vector3::new(voxel.width / 2.0, voxel.height / 2.0, voxel.depth / 2.0);


            obstacles.push(AABB { min, max });
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
        let mut player = Player::new(); 
        // initial position
        player.x = cgmath::Vector3::new(21.0, 1.0, 0.0);

        player.ctm = cgmath::Matrix4::from(player.R)
            * cgmath::Matrix4::from_translation(player.x);
        
        let player_mesh_start = vertices.len() as u32;
        tessellate::tessellate_cube(&mut vertices, tessellation_param);
        let player_mesh_count = vertices.len() as u32 - player_mesh_start;
        let player_mesh = Mesh::new(player_mesh_start, player_mesh_count);

        let player_color = [1.0, 0.2, 0.2, 1.0];
        let player_instance = ObjectData::new(player.ctm, player_color);

        object_collections.push(ObjectCollection::new(
            Shape::Cube,
            vec![player_instance],
            player_mesh,
        ));

        let lights = Lights::new(vec![LightUniform::new(
            [30.0, 30000.0, 50000.0],
            [1.0, 1.0, 1.0],
        )]);

        Self {
            object_collections,
            vertices,
            lights,
            player,
            obstacles
        }
    }

    pub fn init_buffers(&mut self, device: &wgpu::Device) {
        for object_collection in &mut self.object_collections {
            object_collection.init_buffer(device);
        }
        self.lights.init_buffer(device);
        self.player.init_buffer(device);
    }

    pub fn handle_collisions(&mut self) {
        let player = &mut self.player;


        let mut most_collided: Option<(Vector3<f32>, f32, f32)> = None;

        for cube in &self.obstacles {

            let min = cube.min;
            let max = cube.max;

            let player_min = player.x - player.half_extents;
            let player_max = player.x + player.half_extents;


            let overlap_x = (player_max.x - min.x).min(max.x - player_min.x);
            let overlap_y = (player_max.y - min.y).min(max.y - player_min.y);
            let overlap_z = (player_max.z - min.z).min(max.z - player_min.z);


            if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {

                let (pen, normal, area) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
                    (overlap_x, Vector3::new(if player.x.x > (min.x + max.x) * 0.5 { 1.0 } else { -1.0 }, 0.0, 0.0), overlap_y * overlap_z)
                } else if overlap_y <= overlap_z {
                    (overlap_y, Vector3::new(0.0, if player.x.y > (min.y + max.y) * 0.5 { 1.0 } else { -1.0 }, 0.0), overlap_x * overlap_z)
                } else {
                    (overlap_z, Vector3::new(0.0, 0.0, if player.x.z > (min.z + max.z) * 0.5 { 1.0 } else { -1.0 }), overlap_x * overlap_y)
                };

                if most_collided.is_none() || pen > most_collided.unwrap().1 {
                    most_collided = Some((normal, pen, area));
                }
            }
        }


        if let Some((normal, pen, _)) = most_collided {
            let backoff = 0.5;

            player.x += normal * pen;

            let v_along_normal = player.v.dot(normal);
            if v_along_normal < 0.0 {
                player.v -= normal * v_along_normal * backoff;
            }
        }
    }


    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.player.update();

        self.handle_collisions();

        for object_collection in &mut self.object_collections {
            if object_collection.shape == Shape::Cube {
                if let Some(player_instance) = object_collection.object_data.get_mut(0) {
                    player_instance.ctm = self.player.ctm.into();

                    object_collection.write_buffer(queue);
                }
            }
        }
        // self.player.write_buffer(queue);
    }

    pub fn vertices(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }
}
