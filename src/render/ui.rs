use std::collections::HashMap;

use crate::render::shaders;

// allocate 1MB for each buffer for now
const BUFFER_SIZE: usize = 1024 * 1024;

pub struct UiViewport {
    pub size_in_pixels: [u32; 2],
    pub pixels_per_point: f32,
}

#[derive(Debug)]
pub struct ScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ScissorRect {
    pub fn from_egui_rect(rect: egui::Rect, viewport: &UiViewport) -> Self {
        assert!(rect.min.x < rect.max.x);
        assert!(rect.min.y < rect.max.y);

        let mut min_x = (rect.min.x * viewport.pixels_per_point).round() as u32;
        let mut min_y = (rect.min.y * viewport.pixels_per_point).round() as u32;
        let mut max_x = (rect.max.x * viewport.pixels_per_point).round() as u32;
        let mut max_y = (rect.max.y * viewport.pixels_per_point).round() as u32;

        let (viewport_width, viewport_height) =
            (viewport.size_in_pixels[0], viewport.size_in_pixels[1]);
        min_x = min_x.min(viewport_width - 1);
        min_y = min_y.min(viewport_height - 1);
        max_x = max_x.min(viewport_width - 1);
        max_y = max_y.min(viewport_height - 1);

        Self {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

struct Primitive {
    pub base_index: u32,
    pub base_vertex: u32,
    pub index_count: u32,
    pub texture_id: egui::TextureId,
    pub scissor_rect: ScissorRect,
}

struct UiTexture {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewportConstants {
    pub viewport_transform: [f32; 4],
}

pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,

    index_data: Vec<u8>,
    vertex_data: Vec<u8>,

    index_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    textures: HashMap<egui::TextureId, UiTexture>,
    sampler: wgpu::Sampler,

    viewport_constants: wgpu::Buffer,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        modules: &shaders::ShaderModules,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: 20,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Unorm8x4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: modules.get("ui/ui.vs.glsl").unwrap(),
                entry_point: "main",
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: modules.get("ui/ui.fs.glsl").unwrap(),
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let index_data = Vec::with_capacity(BUFFER_SIZE);
        let vertex_data = Vec::with_capacity(BUFFER_SIZE);

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Index Buffer"),
            size: BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Vertex Buffer"),
            size: BUFFER_SIZE as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let textures = HashMap::new();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let viewport_constants = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Viewport Constants"),
            size: std::mem::size_of::<ViewportConstants>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            index_data,
            vertex_data,
            index_buffer,
            vertex_buffer,
            textures,
            sampler,
            viewport_constants,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target: &wgpu::TextureView,
        textures_delta: &egui::TexturesDelta,
        clipped_primitives: &Vec<egui::ClippedPrimitive>,
        viewport: UiViewport,
    ) {
        let viewport_constants = ViewportConstants {
            viewport_transform: [
                2.0 * viewport.pixels_per_point / (viewport.size_in_pixels[0] as f32),
                -2.0 * viewport.pixels_per_point / (viewport.size_in_pixels[1] as f32),
                -1.0,
                1.0,
            ],
        };

        queue.write_buffer(
            &self.viewport_constants,
            0,
            bytemuck::cast_slice(&[viewport_constants]),
        );

        for (texture_id, image_delta) in &textures_delta.set {
            let image_size = image_delta.image.size();
            let size = wgpu::Extent3d {
                width: image_size[0] as u32,
                height: image_size[1] as u32,
                depth_or_array_layers: 1,
            };

            let data = match &image_delta.image {
                egui::epaint::image::ImageData::Color(color_image) => color_image.pixels.clone(),
                egui::epaint::image::ImageData::Font(font_image) => {
                    font_image.srgba_pixels(None).collect()
                }
            };

            if let Some(pos) = image_delta.pos {
                // partial update
                let ui_texture = &self.textures[texture_id];
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &ui_texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: pos[0] as u32,
                            y: pos[1] as u32,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytemuck::cast_slice(&data),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(size.width * 4),
                        rows_per_image: None,
                    },
                    size,
                );
            } else {
                // new texture
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: None,
                    view_formats: &[],
                });

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytemuck::cast_slice(&data),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(size.width * 4),
                        rows_per_image: None,
                    },
                    size,
                );

                let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.viewport_constants.as_entire_binding(),
                        },
                    ],
                });

                let ui_texture = UiTexture {
                    texture,
                    bind_group,
                };

                self.textures.insert(*texture_id, ui_texture);
            }
        }

        self.index_data.clear();
        self.vertex_data.clear();

        let mut base_index = 0u32;
        let mut base_vertex = 0u32;

        let mut primitives = Vec::new();
        for clipped_primitive in clipped_primitives {
            match &clipped_primitive.primitive {
                egui::epaint::Primitive::Mesh(mesh) => {
                    let scissor_rect =
                        ScissorRect::from_egui_rect(clipped_primitive.clip_rect, &viewport);

                    // skip degenerate primitives
                    if scissor_rect.width == 0 || scissor_rect.height == 0 {
                        continue;
                    }

                    self.index_data.extend(bytemuck::cast_slice(&mesh.indices));
                    self.vertex_data
                        .extend(bytemuck::cast_slice(&mesh.vertices));

                    primitives.push(Primitive {
                        base_index,
                        base_vertex,
                        index_count: mesh.indices.len() as u32,
                        texture_id: mesh.texture_id,
                        scissor_rect,
                    });

                    base_index += mesh.indices.len() as u32;
                    base_vertex += mesh.vertices.len() as u32;
                }
                _ => unimplemented!("Unsupported UI primitive type"),
            }
        }

        assert!(self.index_data.len() <= BUFFER_SIZE);
        assert!(self.vertex_data.len() <= BUFFER_SIZE);

        queue.write_buffer(&self.index_buffer, 0, &self.index_data);
        queue.write_buffer(&self.vertex_buffer, 0, &self.vertex_data);

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Ui") });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Ui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(&self.pipeline);

            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            for primitive in &primitives {
                let ui_texture = &self.textures[&primitive.texture_id];
                pass.set_bind_group(0, &ui_texture.bind_group, &[]);

                let rect = &primitive.scissor_rect;
                pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);

                pass.draw_indexed(
                    primitive.base_index..(primitive.base_index + primitive.index_count),
                    primitive.base_vertex as i32,
                    0..1,
                );
            }
        }

        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));

        for texture_id in &textures_delta.free {
            self.textures.remove(texture_id);
        }
    }
}
