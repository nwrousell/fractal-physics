#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, u: f32, v: f32) -> Self {
        Vertex {
            position: [x, y, z],
            normal: [nx, ny, nz],
            tex_coords: [u, v],
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// pub const VERTICES: &[Vertex] = &[
//     // Triangle 1: Top-left, Bottom-left, Top-right (CCW)
//     Vertex {
//         position: [-0.25, 0.25, 0.0],
//         tex_coords: [0.0, 0.0],
//     },
//     Vertex {
//         position: [-0.25, -0.25, 0.0],
//         tex_coords: [0.0, 1.0],
//     },
//     Vertex {
//         position: [0.25, 0.25, 0.0],
//         tex_coords: [1.0, 0.0],
//     },
//     // Triangle 2: Top-right, Bottom-left, Bottom-right (CCW)
//     Vertex {
//         position: [0.25, 0.25, 0.0],
//         tex_coords: [1.0, 0.0],
//     },
//     Vertex {
//         position: [-0.25, -0.25, 0.0],
//         tex_coords: [0.0, 1.0],
//     },
//     Vertex {
//         position: [0.25, -0.25, 0.0],
//         tex_coords: [1.0, 1.0],
//     },
// ];
