use std::ops::Range;

use cgmath::{Matrix, SquareMatrix};
use wgpu::util::DeviceExt;

use super::tessellate::Face;
use crate::buffer::Buffer;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Shape {
    Face(Face),
    Cube
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectData {
    pub ctm: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 3],
    pub color: [f32; 4],
}

impl ObjectData {
    pub fn new(ctm: cgmath::Matrix4<f32>, color: [f32; 4]) -> Self {
        let normal_matrix =
            cgmath::Matrix3::from_cols(ctm.x.truncate(), ctm.y.truncate(), ctm.z.truncate())
                .transpose()
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
            color,
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertex_offset: u32,
    pub num_vertices: u32,
}

impl Mesh {
    pub fn new(vertex_offset: u32, num_vertices: u32) -> Self {
        Mesh {
            vertex_offset,
            num_vertices,
        }
    }
}

#[allow(dead_code)]
pub struct ObjectCollection {
    pub shape: Shape,
    pub object_data: Vec<ObjectData>,
    pub mesh: Mesh,

    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl ObjectCollection {
    pub fn new(shape: Shape, object_data: Vec<ObjectData>, mesh: Mesh) -> Self {
        Self {
            shape,
            object_data,
            mesh,
            buffer: None,
            bind_group: None,
            bind_group_layout: None,
        }
    }

    pub fn object_ranges(&self) -> (Range<u32>, Range<u32>) {
        let vertex_range =
            self.mesh.vertex_offset..(self.mesh.vertex_offset + self.mesh.num_vertices);
        let instance_range = 0..(self.object_data.len() as u32);

        (vertex_range, instance_range)
    }
}

impl Buffer for ObjectCollection {
    fn init_buffer(&mut self, device: &wgpu::Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ObjectCollection Buffer"),
            contents: bytemuck::cast_slice(&self.object_data),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("object_collection_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("object_collection_bind_group"),
        });

        self.buffer = Some(buffer);
        self.bind_group = Some(bind_group);
        self.bind_group_layout = Some(bind_group_layout);
    }

    fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    fn bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
        self.bind_group_layout.as_ref()
    }

    fn write_buffer(&self, queue: &wgpu::Queue) {
        match &self.buffer {
            None => panic!("write_buffer called without buffer set"),
            Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.object_data)),
        };
    }
}
