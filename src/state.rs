use std::sync::Arc;

use wgpu::{TextureView, util::DeviceExt};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{
    camera::{Camera, CameraController, CameraUniform},
    procgen::{generate_world, generate_world_from_png},
    scene::{Scene, Shape, Vertex},
};

pub struct State {
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    postprocess_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    pub window: Option<Arc<Window>>,
    // diffuse_bind_group: wgpu::BindGroup,
    // diffuse_texture: texture::Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    scene: Scene,
    ctm_bind_groups: Vec<(Shape, (wgpu::Buffer, wgpu::BindGroup))>,
    intermediate_texture_view: TextureView,
    intermediate_texture_bind_group: wgpu::BindGroup,
}

impl State {
    pub async fn new_headless(
        png_path: Option<&str>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            // backends: wgpu::Backends::GL,
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(State::new(adapter, png_path, config, None, None).await?)
    }

    pub async fn new_with_png(window: Arc<Window>, png_path: Option<&str>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            // backends: wgpu::Backends::GL,
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code we're using assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // SAFETY: The surface lifetime is tied to the window, but since we store the window
        // in an Arc in the State struct, the window will outlive the State, making it safe
        // to extend the lifetime to 'static.
        let surface: wgpu::Surface<'static> = unsafe { std::mem::transmute(surface) };
        Ok(State::new(adapter, png_path, config, Some(window), Some(surface)).await?)
    }

    async fn new(
        adapter: wgpu::Adapter,
        png_path: Option<&str>,
        config: wgpu::SurfaceConfiguration,
        window: Option<Arc<Window>>,
        surface: Option<wgpu::Surface<'static>>,
    ) -> anyhow::Result<Self> {
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let rects = match png_path {
            Some(path) => generate_world_from_png(path)?,
            None => generate_world()?,
        };

        let scene = Scene::new(4, rects);
        let camera = Camera {
            eye: (30.0, 30.0, -80.0).into(),
            target: (30.0, 30.0, 80.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 300.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: scene.light_buffer(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
            label: Some("camera_bind_group"),
        });

        let ctm_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("CTM Bind Group Layout"),
            });

        let mut ctm_bind_groups = Vec::new();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: scene.vertices(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cube_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CTM Buffer (Cube)"),
            contents: scene.ctms(Shape::Cube),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        let cube_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &ctm_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: cube_buffer.as_entire_binding(),
            }],
            label: Some("Cube Bind Group"),
        });
        ctm_bind_groups.push((Shape::Cube, (cube_buffer, cube_bind_group)));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    // &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &ctm_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let camera_controller = CameraController::new(0.02);

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };
        let render_pipeline = State::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            config.format,
            shader,
            &[Vertex::desc()],
            "Render Pipeline",
        );

        // post-processing
        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Intermediate Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let intermediate_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let intermediate_texture_view =
            intermediate_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let intermediate_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("intermediate_texture_bind_group_layout"),
            });

        let intermediate_texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &intermediate_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&intermediate_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&intermediate_texture_sampler),
                    },
                ],
                label: Some("intermediate_texture_bind_group"),
            });

        let postprocess_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Postprocess Pipeline Layout"),
                bind_group_layouts: &[&intermediate_texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Postprocess Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("postprocess.wgsl").into()),
        };
        let postprocess_render_pipeline = State::create_render_pipeline(
            &device,
            &postprocess_pipeline_layout,
            config.format,
            shader,
            &[],
            "Postprocessing Pipeline",
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            postprocess_render_pipeline,
            vertex_buffer,
            window,
            // diffuse_bind_group,
            // diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            ctm_bind_groups,

            scene,
            intermediate_texture_view,
            intermediate_texture_bind_group,
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        shader: wgpu::ShaderModuleDescriptor,
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
        label: &str,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // entry point for vertex shader
                buffers: vertex_buffer_layouts, // vertex buffer layouts (analogous to VAOs in OpenGL)
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"), // entry point for fragment shader
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // "interpret vertex buffers as list of triangles"
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // CCW determines front face
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // not using depth buffer
            multisample: wgpu::MultisampleState {
                count: 1, // not doing multisampling
                mask: !0,
                alpha_to_coverage_enabled: false, // not doing antialiasing
            },
            multiview: None, // not doing array textures
            cache: None,     // not caching shader compilation data
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            match &self.surface {
                Some(surface) => {
                    surface.configure(&self.device, &self.config);
                }
                None => todo!(),
            }
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        match (&self.surface, &self.window) {
            (Some(surface), Some(window)) => {
                let output = surface.get_current_texture()?;

                window.request_redraw();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                self.render_to_view(view)?;
                output.present();
                Ok(())
            }
            _ => panic!("render called without a window"),
        }
    }

    pub fn render_to_view(&mut self, view: wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.intermediate_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

            for (shape, (_, bind_group)) in &self.ctm_bind_groups {
                render_pass.set_bind_group(1, bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                let (vertex_range, instance_range) = self.scene.shape_ranges(shape);
                render_pass.draw(vertex_range, instance_range);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Postprocessing Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.postprocess_render_pipeline);
            render_pass.set_bind_group(0, &self.intermediate_texture_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera_controller.handle_key(code, is_pressed);
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    // https://sotrh.github.io/learn-wgpu/showcase/windowless/
    pub async fn render_to_file<P: AsRef<std::path::Path>>(
        &mut self,
        output_path: P,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        };
        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&Default::default());

        self.render_to_view(texture_view)?;

        // we need to store this for later
        let u32_size = std::mem::size_of::<u32>() as u32;
        let bytes_per_pixel = u32_size;

        // Align bytes_per_row to COPY_BYTES_PER_ROW_ALIGNMENT (256 bytes)
        const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;
        let unpadded_bytes_per_row = bytes_per_pixel * width;
        let bytes_per_row = ((unpadded_bytes_per_row + COPY_BYTES_PER_ROW_ALIGNMENT - 1)
            / COPY_BYTES_PER_ROW_ALIGNMENT)
            * COPY_BYTES_PER_ROW_ALIGNMENT;

        let output_buffer_size = (bytes_per_row * height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
        // this tells wpgu that we want to read this buffer from the cpu
        | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = self.device.create_buffer(&output_buffer_desc);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            texture_desc.size,
        );

        self.queue.submit(Some(encoder.finish()));

        {
            let buffer_slice = output_buffer.slice(..);

            // NOTE: We have to create the mapping THEN device.poll() before await
            // the future. Otherwise the application will freeze.
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.device.poll(wgpu::PollType::wait_indefinitely())?;
            rx.receive().await.unwrap().unwrap();

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            // Extract only the unpadded row data (skip padding bytes)
            let mut unpadded_data = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
            for row in 0..height {
                let row_start = (row * bytes_per_row) as usize;
                let row_end = row_start + unpadded_bytes_per_row as usize;
                unpadded_data.extend_from_slice(&data[row_start..row_end]);
            }

            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, unpadded_data).unwrap();
            buffer.save(output_path).unwrap();
        }
        output_buffer.unmap();

        Ok(())
    }
}
