use crate::document;
use crate::render::grid;
use crate::render::shaders;
use crate::render::ui;
use crate::render::voxel;
use crate::voxels;

pub struct ViewRenderer {
    grid_renderer: grid::GridRenderer,
    voxel_renderer: voxel::VoxelRenderer,

    view_constant_buffer: wgpu::Buffer,
}

impl ViewRenderer {
    pub fn new(
        device: &wgpu::Device,
        modules: &shaders::ShaderModules,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let view_constant_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Constants"),
            size: std::mem::size_of::<ViewConstants>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_renderer =
            grid::GridRenderer::new(device, modules, surface_format, &view_constant_buffer);

        let voxel_renderer =
            voxel::VoxelRenderer::new(device, modules, surface_format, &view_constant_buffer);

        Self {
            grid_renderer,
            voxel_renderer,
            view_constant_buffer,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_target_view: &wgpu::TextureView,
        depth_target_view: &wgpu::TextureView,
        view_rect: &ui::ScissorRect,
        doc: &document::Document,
    ) {
        let aspect_ratio = view_rect.width as f32 / view_rect.height as f32;
        let (view_matrix, projection_matrix) = doc.viewport.camera.compute_matrices(aspect_ratio);
        let view_constants = ViewConstants {
            view_matrix,
            projection_matrix,
        };

        queue.write_buffer(
            &self.view_constant_buffer,
            0,
            bytemuck::cast_slice(&[view_constants]),
        );

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

            let mut flat_voxel_grid = voxels::VoxelGrid::new();
            for layer in &doc.layers {
                if !layer.visible {
                    continue;
                }

                match layer.blend_mode {
                    document::BlendMode::Add => {
                        flat_voxel_grid.add(&layer.voxel_grid);
                    }
                    document::BlendMode::Subtract => {
                        flat_voxel_grid.subtract(&layer.voxel_grid);
                    }
                }
            }

            self.voxel_renderer.draw(queue, &mut pass, &flat_voxel_grid);

            if doc.viewport.grid_enabled {
                self.grid_renderer.draw(&mut pass);
            }
        }

        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewConstants {
    view_matrix: glam::Mat4,
    projection_matrix: glam::Mat4,
}
