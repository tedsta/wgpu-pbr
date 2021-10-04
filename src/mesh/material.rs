use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::mesh_pass::MeshPass;

#[derive(Copy, Clone)]
pub enum MaterialKind {
    Untextured,
    TexturedUnlit,
    Textured,
    TexturedNorm,
    TexturedNormMat,
    TexturedEmissive,
}

#[derive(Clone)]
pub struct MaterialData {
    pub factors: MaterialFactors,
    pub lighting: bool,

    pub texture: Option<Rc<wgpu::Texture>>,
    pub normal: Option<Rc<wgpu::Texture>>,
    pub metallic_roughness: Option<Rc<wgpu::Texture>>,
    pub ao: Option<Rc<wgpu::Texture>>,
    pub emissive: Option<Rc<wgpu::Texture>>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialFactors {
    pub diffuse: [f32; 4],
    pub metal: f32,
    pub rough: f32,
    pub emissive: [f32; 3],
    pub extra_emissive: [f32; 3],
}

pub struct Material {
    pub factors: MaterialFactors,

    kind: MaterialKind,
    factors_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        data: &MaterialData,
    ) -> Self {
        if data.lighting {
            if let Some(ref texture) = data.texture {
                if let Some(ref emissive) = data.emissive {
                    let normal = data.normal.as_ref().expect("textured model without normal map");
                    let metallic_roughness = data.metallic_roughness.as_ref().expect("textured model without metallic roughness map");
                    let ao = data.ao.as_ref().expect("textured model without ao map");
                    Material::textured_emissive(
                        device, mesh_pass,
                        data.factors.into(),
                        &texture,
                        normal,
                        metallic_roughness,
                        ao,
                        &emissive
                    )
                } else if let Some(ref normal) = data.normal {
                    if let Some(metallic_roughness) = data.metallic_roughness.as_ref() {
                        let ao = data.ao.as_ref().expect("textured model without ao map");
                        Material::textured_norm_mat(
                            device, mesh_pass,
                            data.factors.into(),
                            &texture,
                            &normal,
                            metallic_roughness,
                            ao,
                        )
                    } else {
                        Material::textured_norm(
                            device, mesh_pass,
                            data.factors.into(),
                            &texture,
                            &normal,
                        )
                    }
                } else {
                    Material::textured(
                        device, mesh_pass,
                        data.factors.into(),
                        &texture,
                    )
                }
            } else {
                Material::untextured(
                    device, mesh_pass,
                    data.factors.into(),
                )
            }
        } else {
            if let Some(ref texture) = data.texture {
                Material::textured_unlit(
                    device, mesh_pass,
                    data.factors.into(),
                    &texture,
                )
            } else {
                Material::untextured(
                    device, mesh_pass,
                    data.factors.into(),
                )
            }
        }
    }

    pub fn upload_factors_to_gpu(
        &self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(self.factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &factors_buf, 0, &self.factors_buf, 0,
            std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
        );
    }

    fn textured_unlit(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
        texture: &wgpu::Texture,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_unlit.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        Material {
            factors_buf,
            kind: MaterialKind::TexturedUnlit,
            bind_group,
            factors,
        }
    }

    fn textured(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
        texture: &wgpu::Texture,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        Material {
            factors_buf,
            kind: MaterialKind::Textured,
            bind_group,
            factors,
        }
    }

    fn textured_norm(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
        texture: &wgpu::Texture,
        normal_texture: &wgpu::Texture,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());
        let normal_map_view = normal_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_norm.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
            ],
        });

        Material {
            factors_buf,
            kind: MaterialKind::TexturedNorm,
            bind_group,
            factors,
        }
    }

    fn textured_norm_mat(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
        texture: &wgpu::Texture,
        normal_texture: &wgpu::Texture,
        metallic_roughness_texture: &wgpu::Texture,
        ao_texture: &wgpu::Texture,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());
        let normal_map_view = normal_texture.create_view(&Default::default());
        let metallic_roughness_map_view = metallic_roughness_texture.create_view(&Default::default());
        let ao_map_view = ao_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_norm_mat.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&ao_map_view),
                },
            ],
        });

        Material {
            factors_buf,
            kind: MaterialKind::TexturedNormMat,
            bind_group,
            factors,
        }
    }

    fn textured_emissive(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
        texture: &wgpu::Texture,
        normal_texture: &wgpu::Texture,
        metallic_roughness_texture: &wgpu::Texture,
        ao_texture: &wgpu::Texture,
        emissive_texture: &wgpu::Texture,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&Default::default());
        let normal_map_view = normal_texture.create_view(&Default::default());
        let metallic_roughness_map_view = metallic_roughness_texture.create_view(&Default::default());
        let ao_map_view = ao_texture.create_view(&Default::default());
        let emissive_map_view = emissive_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_emissive.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&ao_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&emissive_map_view),
                },
            ],
        });

        Material {
            factors,
            kind: MaterialKind::TexturedEmissive,
            factors_buf,
            bind_group,
        }
    }

    fn untextured(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        factors: MaterialFactors,
    ) -> Self {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[MaterialFactorsUpload::from(factors)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.untextured.part_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(factors_buf.as_entire_buffer_binding()),
                },
            ],
        });

        Material {
            factors,
            kind: MaterialKind::Untextured,
            factors_buf,
            bind_group,
        }
    }

    pub fn kind(&self) -> MaterialKind { self.kind }
    pub fn factors_buf(&self) -> &wgpu::Buffer { &self.factors_buf }
    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialFactorsUpload {
    pub diffuse: [f32; 4],
    pub metal: f32,
    pub pad0: [u32; 3],
    pub rough: f32,
    pub pad1: [u32; 3],
    pub emissive: [f32; 3],
    pub pad2: [u32; 1],
    pub extra_emissive: [f32; 3],
    pub pad3: [u32; 1],
}

unsafe impl Pod for MaterialFactorsUpload { }
unsafe impl Zeroable for MaterialFactorsUpload { }

impl Default for MaterialFactors {
    fn default() -> Self {
        MaterialFactors {
            diffuse: [1.0, 1.0, 1.0, 1.0],
            metal: 1.0,
            rough: 1.0,
            emissive: [1.0, 1.0, 1.0],
            extra_emissive: [0.0, 0.0, 0.0],
        }
    }
}

impl From<MaterialFactors> for MaterialFactorsUpload {
    fn from(v: MaterialFactors) -> Self {
        MaterialFactorsUpload {
            diffuse: v.diffuse,
            metal: v.metal,
            rough: v.rough,
            emissive: v.emissive,
            extra_emissive: v.extra_emissive,

            pad0: [0; 3],
            pad1: [0; 3],
            pad2: [0; 1],
            pad3: [0; 1],
        }
    }
}

