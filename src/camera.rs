use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;

use crate::buffer::Buffer;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, config: &CameraConfig) {
        self.view_proj = config.build_view_projection_matrix().into();
    }
}

pub struct CameraConfig {
    /// position of camera in world space
    pub eye: cgmath::Point3<f32>,

    /// position camera is looking at
    pub target: cgmath::Point3<f32>,

    /// determines what is up
    pub up: cgmath::Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

pub struct Camera {
    config: CameraConfig,
    uniform: CameraUniform,
    controller: CameraController,

    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Camera {
    pub fn new(config: CameraConfig) -> Self {
        let mut uniform = CameraUniform::new();
        uniform.update_view_proj(&config);
        Self {
            config,
            controller: CameraController::new(0.02),
            bind_group: None,
            bind_group_layout: None,
            buffer: None,
            uniform,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        self.controller.handle_key(code, is_pressed)
    }

    pub fn update(&mut self) {
        use cgmath::InnerSpace;
        let forward = self.config.target - self.config.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Forward/backward movement
        if self.controller.is_forward_pressed && forward_mag > self.controller.speed {
            self.config.eye += forward_norm * self.controller.speed;
            self.config.target += forward_norm * self.controller.speed;
        }
        if self.controller.is_backward_pressed {
            self.config.eye -= forward_norm * self.controller.speed;
            self.config.target -= forward_norm * self.controller.speed;
        }

        // Calculate right vector for lateral movement
        let right = forward_norm.cross(self.config.up).normalize();

        // Left/right lateral movement
        if self.controller.is_right_pressed {
            self.config.eye += right * self.controller.speed;
            self.config.target += right * self.controller.speed;
        }
        if self.controller.is_left_pressed {
            self.config.eye -= right * self.controller.speed;
            self.config.target -= right * self.controller.speed;
        }

        // Up/down movement along the up vector
        if self.controller.is_up_pressed {
            self.config.eye += self.config.up * self.controller.speed;
            self.config.target += self.config.up * self.controller.speed;
        }
        if self.controller.is_down_pressed {
            self.config.eye -= self.config.up * self.controller.speed;
            self.config.target -= self.config.up * self.controller.speed;
        }

        self.uniform.update_view_proj(&self.config);
    }
}

impl Buffer for Camera {
    fn init_buffer(&mut self, device: &wgpu::Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[self.uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
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
            Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.uniform])),
        };
    }
}

#[rustfmt::skip]
// Converts from OpenGL's view volume with z in [-1, 1] (which cgmath assumes) to WebGPU's volume with z in [0, 1]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl CameraConfig {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub struct CameraController {
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
    pub is_up_pressed: bool,
    pub is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }
}
