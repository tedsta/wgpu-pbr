use std::mem;

use crate::{PointLight, SpotLight}; 
use super::{
    super::Scene,
    consts::DEPTH_FORMAT,
    material::MaterialKind,
    mesh_pipeline::MeshPipeline,
};

pub struct MeshPass {
    pub(crate) global_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) mesh_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) global_bind_group: wgpu::BindGroup,

    pub(crate) untextured: MeshPipeline,
    pub(crate) textured_unlit: MeshPipeline,
    pub(crate) textured: MeshPipeline,
    pub(crate) textured_norm: MeshPipeline,
    pub(crate) textured_emissive: MeshPipeline,

    global_buf: wgpu::Buffer,
    pub(crate) depth_texture: wgpu::TextureView,
    pub(crate) bloom_texture: wgpu::TextureView,
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
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                    ),
                ],
            });
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    wgpu::BindGroupLayoutEntry::new(
                        0,
                        wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
                            ),
                        },
                    ),
                ],
            });

        let global_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh-global-buf"),
            size: std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(global_buf.slice(..)),
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
        // Prepare to upload point lights
        let null_point_light = PointLightUpload {
            pos: [0.0; 3], intensity: 0.0, color: [0.0; 3], _pad: 0,
        };
        let mut point_lights = [null_point_light; 32];
        for (i, light) in scene.point_lights.values().enumerate() {
            if i >= 32 { break; }
            point_lights[i] = light.into();
        }

        // Prepare to upload spot lights
        let null_spot_light = SpotLightUpload {
            pos: [0.0; 3],
            color: [0.0; 3],
            dir: [0.0; 3],
            angle: 0.0,
            range: 0.0,
            smoothness: 0.0,
            intensity: 0.0,
            _pad0: 0, _pad1: 0, _pad2: 0,
        };
        let mut spot_lights = [null_spot_light; 32];
        for (i, light) in scene.spot_lights.values().enumerate() {
            if i >= 32 { break; }
            spot_lights[i] = light.into();
        }

        // Upload global uniforms
        let view_proj = scene.camera.total_matrix();
        let global_uniforms = GlobalUniforms {
            view_proj: *view_proj.as_array(),
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
            let temp_buf = device.create_buffer_with_data(
                bytemuck::cast_slice(transform.as_slice()),
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
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.bloom_texture,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(5.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });
            //rpass.set_blend_color(wgpu::Color::TRANSPARENT);
            rpass.set_bind_group(0, &self.global_bind_group, &[]);
            for mesh in scene.meshes.values() {
                rpass.set_bind_group(1, &mesh.bind_group(), &[]);
                for part in &mesh.parts {
                    match part.material.kind() {
                        MaterialKind::Untextured => {
                            rpass.set_pipeline(&self.untextured.pipeline);
                        }
                        MaterialKind::TexturedUnlit => {
                            rpass.set_pipeline(&self.textured_unlit.pipeline);
                        }
                        MaterialKind::Textured => {
                            rpass.set_pipeline(&self.textured.pipeline);
                        }
                        MaterialKind::TexturedNorm => {
                            rpass.set_pipeline(&self.textured_norm.pipeline);
                        }
                        MaterialKind::TexturedEmissive => {
                            rpass.set_pipeline(&self.textured_emissive.pipeline);
                        }
                    }
                    rpass.set_bind_group(2, &part.material.bind_group(), &[]);
                    rpass.set_index_buffer(part.index_buf().slice(..));
                    rpass.set_vertex_buffer(0, part.vertex_buf().slice(..));
                    rpass.draw_indexed(0 .. part.index_count() as u32, 0, 0 .. 1);
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GlobalUniforms {
    view_proj: [f32; 16],
    camera_pos: [f32; 3],
    num_point_lights: i32,
    point_lights: [PointLightUpload; 32],
    num_spot_lights: i32,
    _pad0: [u32; 3],
    spot_lights: [SpotLightUpload; 32],
}

unsafe impl bytemuck::Pod for GlobalUniforms { }
unsafe impl bytemuck::Zeroable for GlobalUniforms { }

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct PointLightUpload {
    pos: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    _pad: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct SpotLightUpload {
    pos: [f32; 3],
    angle: f32,
    color: [f32; 3],
    range: f32,
    dir: [f32; 3],
    smoothness: f32,
    intensity: f32,
    _pad0: u32, _pad1: u32, _pad2: u32,
}

impl From<&PointLight> for PointLightUpload {
    fn from(v: &PointLight) -> Self {
        PointLightUpload {
            pos: v.pos,
            color: v.color,
            intensity: v.intensity,

            _pad: 0,
        }
    }
}

impl From<&SpotLight> for SpotLightUpload {
    fn from(v: &SpotLight) -> Self {
        SpotLightUpload {
            pos: v.pos,
            angle: v.angle,
            color: v.color,
            range: v.range,
            dir: v.dir,
            smoothness: v.smoothness,
            intensity: v.intensity,

            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }
    }
}

