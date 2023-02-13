use crate::render;
use crate::render::shaders;
use crate::render::ui;

pub struct ViewRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl ViewRenderer {
    pub fn new(
        device: &wgpu::Device,
        modules: &shaders::ShaderModules,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: modules.get("test.vs.glsl").unwrap(),
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: modules.get("test.fs.glsl").unwrap(),
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { pipeline }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target_view: &wgpu::TextureView,
        depth_target_view: &wgpu::TextureView,
        view_rect: &ui::ScissorRect,
        viewport: &render::Viewport,
    ) {
        if !viewport.option {
            return;
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Viewport"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_target_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_viewport(
                view_rect.x as f32,
                view_rect.y as f32,
                view_rect.width as f32,
                view_rect.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(view_rect.x, view_rect.y, view_rect.width, view_rect.height);

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));
    }
}
