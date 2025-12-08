use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{
    buffer::Buffer,
    camera::{Camera, CameraConfig},
    scene::{Scene, Vertex},
    texture::{PostprocessTexture, Texture},
};

pub struct Game {
    surface: Option<wgpu::Surface<'static>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    pub window: Option<Arc<Window>>,

    depth_texture: Texture,

    postprocess_texture: PostprocessTexture,
    do_postprocess: bool,

    vertex_buffer: wgpu::Buffer,
    camera: Camera,
    scene: Scene,
}

impl Game {
    pub async fn new_headless(
        scene: Scene,
        width: u32,
        height: u32,
        do_postprocess: bool,
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

        Ok(Game::new(adapter, scene, config, do_postprocess, None, None).await?)
    }

    pub async fn new_window(
        window: Arc<Window>,
        scene: Scene,
        do_postprocess: bool,
    ) -> anyhow::Result<Self> {
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
        Ok(Game::new(
            adapter,
            scene,
            config,
            do_postprocess,
            Some(window),
            Some(surface),
        )
        .await?)
    }

    async fn new(
        adapter: wgpu::Adapter,
        mut scene: Scene,
        config: wgpu::SurfaceConfiguration,
        do_postprocess: bool,
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

        scene.init_buffers(&device);

        let camera_config = CameraConfig {
            eye: (30.0, 30.0, -80.0).into(),
            target: (30.0, 30.0, 80.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 300.0,
        };
        let mut camera = Camera::new(camera_config);
        camera.init_buffer(&device);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: scene.vertices(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera.bind_group_layout().unwrap(),
                    scene.lights.bind_group_layout().unwrap(),
                    &scene
                        .object_collections
                        .first()
                        .unwrap()
                        .bind_group_layout()
                        .unwrap(),
                ],
                push_constant_ranges: &[],
            });

        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        };
        let render_pipeline = Game::create_render_pipeline(
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
        let postprocess_texture = PostprocessTexture::new(&device, texture_extent, config.format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            render_pipeline,
            window,

            depth_texture,

            postprocess_texture,
            do_postprocess,

            vertex_buffer,
            camera,
            scene,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
            self.is_surface_configured = true;
        }
    }

    pub fn render_to_window(&mut self) -> Result<(), wgpu::SurfaceError> {
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

        let first_view = if self.do_postprocess {
            &self.postprocess_texture.texture.view
        } else {
            &view
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: first_view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group().unwrap(), &[]);
            render_pass.set_bind_group(1, self.scene.lights.bind_group().unwrap(), &[]);

            for object_collection in &self.scene.object_collections {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_bind_group(2, object_collection.bind_group().unwrap(), &[]);

                let (vertex_range, instance_range) = object_collection.object_ranges();
                render_pass.draw(vertex_range, instance_range);
            }
        }

        if self.do_postprocess {
            self.postprocess_texture.render_pass(&mut encoder, &view);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera.handle_key(code, is_pressed);
        }
    }

    pub fn update(&mut self) {
        self.camera.update();
        self.camera.write_buffer(&self.queue);
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
