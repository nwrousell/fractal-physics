use wgpu::util::DeviceExt;

use crate::buffer::Buffer;

/// Represents a point light
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub used: u32, // 0 = false, 1 = true (using u32 for proper alignment)
    pub color: [f32; 3],
    pub _padding: u32,
}

impl LightUniform {
    /// Create a new light with the given position and color
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            used: 0, // Will be set to 1 by Lights::new()
            color,
            _padding: 0,
        }
    }
}

pub struct Lights {
    lights: [LightUniform; 8],

    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Lights {
    pub fn new(lights: Vec<LightUniform>) -> Self {
        assert!(
            lights.len() <= 8,
            "Lights vec must have length <= 8, got {}",
            lights.len()
        );

        // Initialize array with unused lights (used = 0)
        let mut lights_array = [LightUniform {
            position: [0.0; 3],
            used: 0,
            color: [0.0; 3],
            _padding: 0,
        }; 8];

        // Copy provided lights and mark them as used
        for (i, mut light) in lights.into_iter().enumerate() {
            light.used = 1; // Mark as used
            lights_array[i] = light;
        }

        Self {
            lights: lights_array,
            buffer: None,
            bind_group: None,
            bind_group_layout: None,
        }
    }
}

impl Buffer for Lights {
    fn init_buffer(&mut self, device: &wgpu::Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&self.lights),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("light_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
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
            Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.lights)),
        };
    }
}
