use std::rc::Rc;

use super::{
    geometry::MeshPartGeometry,
    material::{MaterialFactors, MaterialFactorsUpload},
    mesh_pass::MeshPass,
};

#[derive(Copy, Clone)]
pub enum MeshPartKind {
    Untextured,
    TexturedUnlit,
    Textured,
    TexturedNorm,
    TexturedEmissive,
}

pub struct MeshPartData {
    pub geometry: MeshPartGeometry,
    pub material_factors: MaterialFactors,
    pub texture: Option<Rc<wgpu::Texture>>,
    pub normal: Option<Rc<wgpu::Texture>>,
    pub metallic_roughness: Option<Rc<wgpu::Texture>>,
    pub ao: Option<Rc<wgpu::Texture>>,
    pub emissive: Option<Rc<wgpu::Texture>>,
}


pub struct MeshPart {
    kind: MeshPartKind,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    mat_factors_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    pub mat_factors: MaterialFactors,
}

impl MeshPart {
    pub fn update_material(
        &self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(self.mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
        );
        encoder.copy_buffer_to_buffer(
            &mat_factors_buf, 0, &self.mat_factors_buf, 0,
            std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
        );
    }

    pub fn textured_unlit(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        geometry: &MeshPartGeometry,
        mat_factors: MaterialFactors,
        texture: &wgpu::Texture,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.vertices),
            wgpu::BufferUsage::VERTEX
        );
        let index_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.indices),
            wgpu::BufferUsage::INDEX
        );

        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let texture_view = texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_unlit.part_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mat_factors_buf,
                        range: 0 .. std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        MeshPart {
            kind: MeshPartKind::TexturedUnlit,
            vertex_buf,
            index_buf,
            index_count: geometry.indices.len(),
            mat_factors_buf,
            bind_group,
            mat_factors,
        }
    }

    pub fn textured(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        geometry: &MeshPartGeometry,
        mat_factors: MaterialFactors,
        texture: &wgpu::Texture,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.vertices),
            wgpu::BufferUsage::VERTEX
        );
        let index_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.indices),
            wgpu::BufferUsage::INDEX
        );

        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let texture_view = texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured.part_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mat_factors_buf,
                        range: 0 .. std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        MeshPart {
            kind: MeshPartKind::Textured,
            vertex_buf,
            index_buf,
            index_count: geometry.indices.len(),
            mat_factors_buf,
            bind_group,
            mat_factors,
        }
    }

    pub fn textured_norm(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        geometry: &MeshPartGeometry,
        mat_factors: MaterialFactors,
        texture: &wgpu::Texture,
        normal_texture: &wgpu::Texture,
        metallic_roughness_texture: &wgpu::Texture,
        ao_texture: &wgpu::Texture,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.vertices),
            wgpu::BufferUsage::VERTEX
        );
        let index_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.indices),
            wgpu::BufferUsage::INDEX
        );

        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let texture_view = texture.create_default_view();
        let normal_map_view = normal_texture.create_default_view();
        let metallic_roughness_map_view = metallic_roughness_texture.create_default_view();
        let ao_map_view = ao_texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_norm.part_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mat_factors_buf,
                        range: 0 .. std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_map_view),
                },
                wgpu::Binding {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&ao_map_view),
                },
            ],
        });

        MeshPart {
            kind: MeshPartKind::TexturedNorm,
            vertex_buf,
            index_buf,
            index_count: geometry.indices.len(),
            mat_factors_buf,
            bind_group,
            mat_factors,
        }
    }

    pub fn textured_emissive(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        geometry: &MeshPartGeometry,
        mat_factors: MaterialFactors,
        texture: &wgpu::Texture,
        normal_texture: &wgpu::Texture,
        metallic_roughness_texture: &wgpu::Texture,
        ao_texture: &wgpu::Texture,
        emissive_texture: &wgpu::Texture,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.vertices),
            wgpu::BufferUsage::VERTEX
        );
        let index_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.indices),
            wgpu::BufferUsage::INDEX
        );

        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let texture_view = texture.create_default_view();
        let normal_map_view = normal_texture.create_default_view();
        let metallic_roughness_map_view = metallic_roughness_texture.create_default_view();
        let ao_map_view = ao_texture.create_default_view();
        let emissive_map_view = emissive_texture.create_default_view();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.textured_emissive.part_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mat_factors_buf,
                        range: 0 .. std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
                wgpu::Binding {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_map_view),
                },
                wgpu::Binding {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&ao_map_view),
                },
                wgpu::Binding {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&emissive_map_view),
                },
            ],
        });

        MeshPart {
            kind: MeshPartKind::TexturedEmissive,
            vertex_buf,
            index_buf,
            index_count: geometry.indices.len(),
            mat_factors_buf,
            bind_group,
            mat_factors,
        }
    }

    pub fn untextured(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        geometry: &MeshPartGeometry,
        mat_factors: MaterialFactors,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.vertices),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&geometry.indices),
            wgpu::BufferUsage::INDEX,
        );

        let mat_factors_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&[MaterialFactorsUpload::from(mat_factors)]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.untextured.part_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &mat_factors_buf,
                        range: 0 .. std::mem::size_of::<MaterialFactorsUpload>() as wgpu::BufferAddress,
                    },
                },
            ],
        });

        MeshPart {
            kind: MeshPartKind::Untextured,
            vertex_buf,
            index_buf,
            index_count: geometry.indices.len(),
            mat_factors_buf,
            bind_group,
            mat_factors,
        }
    }

    pub fn kind(&self) -> MeshPartKind { self.kind }
    pub fn vertex_buf(&self) -> &wgpu::Buffer { &self.vertex_buf }
    pub fn index_buf(&self) -> &wgpu::Buffer { &self.index_buf }
    pub fn index_count(&self) -> usize { self.index_count }
    pub fn mat_factors_buf(&self) -> &wgpu::Buffer { &self.mat_factors_buf }
    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }

}

