use std::collections::HashMap;
use winit::keyboard::KeyCode;
use cgmath::{
    Matrix4, Matrix3, Vector3, Vector4,
    InnerSpace, SquareMatrix, Matrix
};
use crate::buffer::Buffer;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlayerUniform {
    pub ctm: [[f32; 4]; 4],
}

impl PlayerUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            ctm: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, ctm: cgmath::Matrix4<f32>) {
        self.ctm = ctm.into();
    }
}


pub struct Player {
    pub is_key_pressed: HashMap<KeyCode, bool>,
    // add pos, rot, etc.
    // PlayerUniform (look at CameraUniform), that computes CTM based on pos, rot, etc.
    pub ctm: Matrix4<f32>,      // Current transform matrix

    pub inertia: Matrix3<f32>,  // Body-frame inertia tensor
    pub R: Matrix3<f32>,        // Rotation matrix
    pub omega: Vector3<f32>,    // Angular velocity
    pub q: Vector4<f32>,        // Quaternion (w, x, y, z)

    pub mass: f32,
    pub x: Vector3<f32>,        // Position
    pub v: Vector3<f32>,        // Linear velocity
    pub P: Vector3<f32>,        // Momentum

    pub Iinv: Matrix3<f32>,     // Inverse inertia tensor
    pub L: Vector3<f32>,        // Angular momentum

    pub force: Vector3<f32>,
    pub torque: Vector3<f32>,

    pub uniform: PlayerUniform,
    pub buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub half_extents: Vector3<f32>,
}

const KEYS: [KeyCode; 4] = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyD, KeyCode::KeyS];

impl Player {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        for key in KEYS {
            keys.insert(key, false);
        }

        Self {
            is_key_pressed: keys,
            ctm: Matrix4::identity(),
            buffer: None,
            bind_group: None,
            bind_group_layout: None,
            uniform: PlayerUniform::new(),

            inertia: Matrix3::identity(),
            R: Matrix3::identity(),
            omega: Vector3::new(0.0, 0.0, 0.0),

            // quaternion: (w, x, y, z)
            q: Vector4::new(1.0, 0.0, 0.0, 0.0),

            mass: 1.0,
            x: Vector3::new(0.0, 0.0, 0.0),
            v: Vector3::new(0.0, 0.0, 0.0),
            P: Vector3::new(0.0, 0.0, 0.0),

            Iinv: Matrix3::identity(),
            L: Vector3::new(0.0, 0.0, 0.0),

            force: Vector3::new(0.0, 0.0, 0.0),
            torque: Vector3::new(0.0, 0.0, 0.0),
            half_extents: Vector3::new(0.5, 0.5, 1.0),
        }
    }

    pub fn print_ctm(&self) {
        println!("Player CTM:");
        for row in 0..4 {
            println!(
                "[{:.3}, {:.3}, {:.3}, {:.3}]",
                self.ctm[row][0], self.ctm[row][1], self.ctm[row][2], self.ctm[row][3]
            );
        }
        println!("-------------------------");
    }

    pub fn print_matrix(&self, mat: Matrix3<f32>) {
        println!("Matrix3:");
        for row in 0..3 {
            println!(
                "[{:.3}, {:.3}, {:.3}]",
                mat[row][0], mat[row][1], mat[row][2]
            );
        }
        println!("-------------------------");
    }

    pub fn clear_forces(&mut self) {
        self.force = Vector3::new(0.0, 0.0, 0.0);
        self.torque = Vector3::new(0.0, 0.0, 0.0);
    }

    pub fn apply_force(&mut self, f: Vector3<f32>) {
        self.force += f;
    }

    pub fn apply_force_at_point(&mut self, f: Vector3<f32>, point_world: Vector3<f32>) {
        print!("I AM APPLYING FORCE at point\n");
        self.force += f;
        let r = point_world - self.x;
        self.torque += r.cross(f);
    }

    pub fn compute_auxiliary(&mut self) {
        let inertia_inv = self.inertia.invert().unwrap_or(Matrix3::from_value(0.0));
        self.Iinv = self.R * inertia_inv * self.R.transpose();
    }


    pub fn quat_mul(&self, q1: Vector4<f32>, q2: Vector4<f32>) -> Vector4<f32> {
        let (w1, x1, y1, z1) = (q1.x, q1.y, q1.z, q1.w);
        let (w2, x2, y2, z2) = (q2.x, q2.y, q2.z, q2.w);

        Vector4::new(
            w1*w2 - x1*x2 - y1*y2 - z1*z2,
            w1*x2 + x1*w2 + y1*z2 - z1*y2,
            w1*y2 - x1*z2 + y1*w2 + z1*x2,
            w1*z2 + x1*y2 - y1*x2 + z1*w2,
        )
    }


    pub fn quat_to_mat3(&self, q: Vector4<f32>) -> Matrix3<f32> {
        let (w, x, y, z) = (q.x, q.y, q.z, q.w);

        Matrix3::new(
            1.0 - 2.0*(y*y + z*z),   2.0*(x*y - w*z),       2.0*(x*z + w*y),
            2.0*(x*y + w*z),         1.0 - 2.0*(x*x + z*z), 2.0*(y*z - w*x),
            2.0*(x*z - w*y),         2.0*(y*z + w*x),       1.0 - 2.0*(x*x + y*y),
        )
    }


    pub fn simulate(&mut self, dt: f32) {

        self.compute_auxiliary();

        let friction_coefficient = 0.5;
        let friction_force = -self.v * friction_coefficient;
        let dv = (self.force + friction_force) / self.mass * dt;
        self.v += dv;
        let angular_friction_coefficient = 0.1;

        // linear motion
        // let dv = (self.force / self.mass) * dt;
        // self.v += dv;
        self.P = self.v * self.mass;
        self.x += self.v * dt;

        // angular motion
        // let domega = self.Iinv * self.torque * dt;
        // self.omega += domega;
        // self.L = self.inertia * self.omega;
        let angular_friction = -self.omega * angular_friction_coefficient;
        let domega = self.Iinv * (self.torque + angular_friction) * dt;
        self.omega += domega;
        self.L = self.inertia * self.omega;

        // quaternion integration
        let omega_len = self.omega.magnitude();
        let angle = omega_len * dt;

        if angle > 1e-6 {
            let axis = self.omega / omega_len;
            let half = 0.5 * angle;

            let (s, c) = (half.sin(), half.cos());

            let dq = Vector4::new(
                c,
                axis.x * s,
                axis.y * s,
                axis.z * s,
            );

            self.q = self.quat_mul(self.q, dq);
            self.q = self.q.normalize();
        }

        self.R = self.quat_to_mat3(self.q);

        // self.print_matrix(self.R);

        self.ctm = Matrix4::from_translation(self.x) * Matrix4::from(self.R);
        self.print_ctm();
        self.uniform.update(self.ctm);
        self.clear_forces();
    }

    pub fn handle_key(&mut self, key: KeyCode, is_pressed: bool) {
        // a and d for rotation, and then w for forwards movement (s later for backwards)
        match key {
            KeyCode::KeyW => self.is_key_pressed.insert(KeyCode::KeyW, is_pressed),
            KeyCode::KeyA => self.is_key_pressed.insert(KeyCode::KeyA, is_pressed),
            KeyCode::KeyD => self.is_key_pressed.insert(KeyCode::KeyD, is_pressed),
            KeyCode::KeyS => self.is_key_pressed.insert(KeyCode::KeyS, is_pressed),
            _ => None,
        };
        self.update();
    }

    pub fn update(&mut self) {
        // update pos/rot
        let torque_strength = 5.0;
        let max_steering_angle = std::f32::consts::PI / 8.0; // max wheel turn angle (~22.5 degrees)
        let steering_input = if self.is_key_pressed[&KeyCode::KeyA] {
            1.0 
        } else if self.is_key_pressed[&KeyCode::KeyD] {
            -1.0 
        } else {
            0.0
        };

        let steering_torque = max_steering_angle * steering_input;

        self.torque += Vector3::new(0.0, steering_torque, 0.0);
        if self.is_key_pressed[&KeyCode::KeyW] {
            let forward_force = self.R * Vector3::new(0.0, 0.0, -20.0);
            self.apply_force(forward_force);
        } 
        if self.is_key_pressed[&KeyCode::KeyS] {
            let backward_force = self.R * Vector3::new(0.0, 0.0, 20.0);
            self.apply_force(backward_force);
        }

        if !self.is_key_pressed[&KeyCode::KeyW] && !self.is_key_pressed[&KeyCode::KeyS] {
            self.v *= 0.98; 
        }

        if steering_input == 0.0 {
            self.omega *= 0.98;
        }
        // basic physics controls 
        // if self.is_key_pressed[&KeyCode::KeyA] {
        //     // self.apply_force_at_point(
        //     //     Vector3::new(-5.0, 0.0, 0.0),
        //     //     self.x + Vector3::new(0.5, 0.0, 0.0)
        //     // );
        //     // self.torque += self.R * Vector3::new(0.0, 1.0, 0.0) * torque_strength;
        //     self.torque += Vector3::new(0.0, torque_strength, 0.0);
        // }
        // if self.is_key_pressed[&KeyCode::KeyD] {
        //     // self.apply_force_at_point(
        //     //     Vector3::new(5.0, 0.0, 0.0),
        //     //     self.x + Vector3::new(-0.5, 0.0, 0.0)
        //     // );
        //     // self.torque += self.R * Vector3::new(0.0, 1.0, 0.0) * torque_strength;
        //     self.torque += Vector3::new(0.0, -torque_strength, 0.0);
        // }
        // if self.is_key_pressed[&KeyCode::KeyA] || self.is_key_pressed[&KeyCode::KeyD] {
        //     self.v = Vector3::new(0.0, 0.0, 0.0);
        //     self.P = Vector3::new(0.0, 0.0, 0.0);
        // }
        // if self.is_key_pressed[&KeyCode::KeyW] {
        //     let f = self.R * Vector3::new(0.0, 0.0, -20.0);
        //     self.apply_force(f);
        //     self.torque = Vector3::new(0.0, 0.0, 0.0);
        //     self.omega  = Vector3::new(0.0, 0.0, 0.0);
        //     self.L      = Vector3::new(0.0, 0.0, 0.0);
        // }
        // if self.is_key_pressed[&KeyCode::KeyS] {
        //     let f = self.R * Vector3::new(0.0, 0.0, 20.0);
        //     self.apply_force(f);
        //     self.torque = Vector3::new(0.0, 0.0, 0.0);
        //     self.omega  = Vector3::new(0.0, 0.0, 0.0);
        //     self.L      = Vector3::new(0.0, 0.0, 0.0);
        // }

        self.simulate(1.0/64.0);

        // self.print_ctm();

        // call func to update uniform ctm
    }
}

impl Buffer for Player {
    fn init_buffer(&mut self, device: &wgpu::Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Buffer"),
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
            label: Some("player_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("player_bind_group"),
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

    // fn write_buffer(&self, queue: &wgpu::Queue) {
    fn write_buffer(&self, queue: &wgpu::Queue) {

        print!("Writing player buffer\n");
        if let Some(buffer) = &self.buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        } else {
            panic!("Player uniform buffer not initialized");
        }
    }
    //     match &self.buffer {
    //         None => panic!("write_buffer called without buffer set"),
    //         Some(buffer) => queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[self.uniform])),
    //     };
    // }

}
