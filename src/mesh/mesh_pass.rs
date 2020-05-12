use cgmath::SquareMatrix;

use super::{
    super::Scene,
    consts::DEPTH_FORMAT,
    mesh_part::MeshPartKind,
    mesh_pipeline::MeshPipeline,
};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PointLightData {
    pub pos: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    _pad: u32,
}

impl PointLightData {
    pub fn new(pos: [f32; 3], intensity: f32, color: [f32; 3]) -> Self {
        PointLightData { pos, intensity, color, _pad: 0 }
    }

    pub fn zero() -> Self {
        PointLightData {
            pos: [0.0; 3],
            intensity: 0.0,
            color: [0.0; 3],
            _pad: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpotLightData {
    pub pos: [f32; 3],
    pub angle: f32,
    pub color: [f32; 3],
    pub range: f32,
    pub dir: [f32; 3],
    pub smoothness: f32,
    pub intensity: f32,
    _pad0: u32, _pad1: u32, _pad2: u32,
}

impl SpotLightData {
    pub fn zero() -> Self {
        SpotLightData {
            pos: [0.0; 3],
            color: [0.0; 3],
            dir: [0.0; 3],
            angle: 0.0,
            range: 0.0,
            smoothness: 0.0,
            intensity: 0.0,
            _pad0: 0, _pad1: 0, _pad2: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GlobalUniforms {
    view_proj: [f32; 16],
    camera_pos: [f32; 3],
    num_point_lights: i32,
    point_lights: [PointLightData; 32],
    num_spot_lights: i32,
    _pad0: [u32; 3],
    spot_lights: [SpotLightData; 32],
}

unsafe impl bytemuck::Pod for GlobalUniforms { }
unsafe impl bytemuck::Zeroable for GlobalUniforms { }

pub struct MeshPass {
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
    pub global_buf: wgpu::Buffer,

    pub untextured: MeshPipeline,
    pub textured_unlit: MeshPipeline,
    pub textured: MeshPipeline,
    pub textured_norm: MeshPipeline,
    pub textured_emissive: MeshPipeline,

    pub depth_texture: wgpu::TextureView,
    pub bloom_texture: wgpu::TextureView,
}

impl MeshPass {
    pub fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
    ) -> Self {
        let init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Create pipeline layout
        let mesh_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                        },
                    },
                ],
            });
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                        },
                    },
                ],
            });

        let mx_total = cgmath::Matrix4::identity();
        let global_uniforms = GlobalUniforms {
            view_proj: *mx_total.as_ref(),
            camera_pos: [0.0, 0.0, 0.0],
            num_point_lights: 0,
            point_lights: [PointLightData::new([0.0; 3], 0.0, [0.0; 3]); 32],
            num_spot_lights: 0,
            _pad0: [0; 3],
            spot_lights: [SpotLightData::zero(); 32],
        };
        let global_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[global_uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_buf,
                        range: 0 .. std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
                    },
                },
            ],
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let bloom_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        // Done
        queue.submit(Some(init_encoder.finish()));

        let untextured = MeshPipeline::untextured(
            sc_desc, device, &global_bind_group_layout, &mesh_bind_group_layout,
        );
        let textured_unlit = MeshPipeline::textured_unlit(
            sc_desc, device, &global_bind_group_layout, &mesh_bind_group_layout,
        );
        let textured = MeshPipeline::textured(
            sc_desc, device, &global_bind_group_layout, &mesh_bind_group_layout,
        );
        let textured_norm = MeshPipeline::textured_norm(
            sc_desc, device, &global_bind_group_layout, &mesh_bind_group_layout,
        );
        let textured_emissive = MeshPipeline::textured_emissive(
            sc_desc, device, &global_bind_group_layout, &mesh_bind_group_layout,
        );

        MeshPass {
            global_bind_group_layout,
            mesh_bind_group_layout,
            global_bind_group,
            global_buf,

            untextured,
            textured_unlit,
            textured,
            textured_norm,
            textured_emissive,

            depth_texture: depth_texture.create_default_view(),
            bloom_texture: bloom_texture.create_default_view(),
        }
    }

    pub fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &mut wgpu::Device,
    ) {
        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        }).create_default_view();

        self.bloom_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        }).create_default_view();
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        render_target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
    ) {
        // Update camera
        let mx_total = scene.camera.total_matrix();
        let mut point_lights = [PointLightData::new([0.0, 0.0, 0.0], 0.0, [0.0; 3]); 32];
        for (i, light) in scene.point_lights.values().enumerate() {
            point_lights[i] = *light;
        }

        let mut spot_lights = [SpotLightData::zero(); 32];
        for (i, light) in scene.spot_lights.values().enumerate() {
            spot_lights[i] = *light;
        }
        let global_uniforms = GlobalUniforms {
            view_proj: *mx_total.as_ref(),
            camera_pos: [
                scene.camera.position().x,
                scene.camera.position().y,
                scene.camera.position().z,
            ],
            num_point_lights: scene.point_lights.len() as i32,
            point_lights: point_lights,
            num_spot_lights: scene.spot_lights.len() as i32,
            _pad0: [0; 3],
            spot_lights: spot_lights,
        };
        let global_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[global_uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        );
        encoder.copy_buffer_to_buffer(
            &global_buf, 0, &self.global_buf, 0,
            std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
        );


        // Upload mesh transform matrices
        for mesh in scene.meshes.values() {
            let transform = mesh.transform();
            let mx_ref: &[f32; 16] = transform.as_ref();
            let temp_buf = device.create_buffer_with_data(
                bytemuck::cast_slice(mx_ref.as_ref()),
                wgpu::BufferUsage::COPY_SRC,
            );
            encoder.copy_buffer_to_buffer(&temp_buf, 0, &mesh.uniform_buf(), 0, 64);
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &render_target,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.bloom_texture,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_depth: 5.00,
                    clear_stencil: 0,
                }),
            });
            //rpass.set_blend_color(wgpu::Color::TRANSPARENT);
            rpass.set_bind_group(0, &self.global_bind_group, &[]);
            for mesh in scene.meshes.values() {
                rpass.set_bind_group(1, &mesh.bind_group(), &[]);
                for part in &mesh.parts {
                    match part.kind() {
                        MeshPartKind::Untextured => {
                            rpass.set_pipeline(&self.untextured.pipeline);
                        }
                        MeshPartKind::TexturedUnlit => {
                            rpass.set_pipeline(&self.textured_unlit.pipeline);
                        }
                        MeshPartKind::Textured => {
                            rpass.set_pipeline(&self.textured.pipeline);
                        }
                        MeshPartKind::TexturedNorm => {
                            rpass.set_pipeline(&self.textured_norm.pipeline);
                        }
                        MeshPartKind::TexturedEmissive => {
                            rpass.set_pipeline(&self.textured_emissive.pipeline);
                        }
                    }
                    rpass.set_bind_group(2, &part.bind_group(), &[]);
                    rpass.set_index_buffer(&part.index_buf(), 0, 0);
                    rpass.set_vertex_buffer(0, &part.vertex_buf(), 0, 0);
                    rpass.draw_indexed(0 .. part.index_count() as u32, 0, 0 .. 1);
                }
            }
        }
    }
}
