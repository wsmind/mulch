use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,

    recreate_targets: bool,
    depth_target_view: Option<wgpu::TextureView>,
}

impl Renderer {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &W,
        window_size: [u32; 2],
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface =
            unsafe { instance.create_surface(window) }.expect("Failed to create render surface");
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .expect("Failed to initialize graphics adapter");

        println!("{:?}", adapter.get_info());

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                limits: Default::default(),
                label: None,
            },
            None,
        ))
        .expect("Failed to create device");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .filter(|format| !format.describe().srgb)
            .next()
            .expect("Failed to find a suitable compatible surface format");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: window_size[0],
            height: window_size[1],
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        println!("{:?}", surface_config);

        Self {
            device,
            queue,
            surface,
            surface_config,

            recreate_targets: true,
            depth_target_view: None,
        }
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
        if new_size[0] == 0 || new_size[1] == 0 {
            return;
        }

        self.surface_config.width = new_size[0];
        self.surface_config.height = new_size[1];

        self.recreate_targets = true;
    }

    fn refresh_targets(&mut self) {
        if self.recreate_targets {
            self.surface.configure(&self.device, &self.surface_config);

            let depth_target = self.device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Depth Buffer"),
                view_formats: &[],
            });

            let depth_target_view =
                depth_target.create_view(&wgpu::TextureViewDescriptor::default());

            self.depth_target_view = Some(depth_target_view);

            self.recreate_targets = false;
        }
    }

    pub fn render(&mut self) {
        self.refresh_targets();

        let current_texture = self.surface.get_current_texture();
        if current_texture.is_err() {
            // bail and try again next frame
            self.recreate_targets = true;
            return;
        }

        let color_target = current_texture.unwrap();
        let color_target_view = &color_target
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let depth_target_view = self.depth_target_view.as_ref().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render"),
            });

        {
            let mut _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_target_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));

        color_target.present();
    }
}