use cgmath::{self, SquareMatrix};

use super::{
    mesh_part::{MeshPart, MeshPartData},
    mesh_pass::MeshPass,
};

pub struct Mesh {
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Matrix4<f32>,
    pub scale: cgmath::Vector3<f32>,
    pub parts: Vec<MeshPart>,

    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

impl Mesh {
    pub fn from_parts(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        lighting: bool,
        mesh_parts: &[MeshPartData],
    ) -> Mesh {
        let transform = cgmath::Matrix4::identity();
        let transform_ref: &[f32; 16] = transform.as_ref();
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(transform_ref),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.mesh_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0 .. 64, // mat4
                    },
                },
            ],
        });

        let mut parts = Vec::new();
        if lighting {
            for part_data in mesh_parts {
                if let Some(ref texture) = part_data.texture {
                    if let Some(ref emissive) = part_data.emissive {
                        let normal = part_data.normal.as_ref().expect("textured model without normal map");
                        let metallic_roughness = part_data.metallic_roughness.as_ref().expect("textured model without metallic roughness map");
                        let ao = part_data.ao.as_ref().expect("textured model without ao map");
                        parts.push(MeshPart::textured_emissive(
                            device, mesh_pass,
                            &part_data.geometry,
                            part_data.material_factors.into(),
                            &texture,
                            normal,
                            metallic_roughness,
                            ao,
                            &emissive
                        ));
                    } else if let Some(ref normal) = part_data.normal {
                        let metallic_roughness = part_data.metallic_roughness.as_ref().expect("textured model without metallic roughness map");
                        let ao = part_data.ao.as_ref().expect("textured model without ao map");
                        parts.push(MeshPart::textured_norm(
                            device, mesh_pass,
                            &part_data.geometry,
                            part_data.material_factors.into(),
                            &texture,
                            &normal,
                            metallic_roughness,
                            ao,
                        ));
                    } else {
                        parts.push(MeshPart::textured(
                            device, mesh_pass,
                            &part_data.geometry,
                            part_data.material_factors.into(),
                            &texture,
                        ));
                    }
                } else {
                    parts.push(MeshPart::untextured(
                        device, mesh_pass,
                        &part_data.geometry,
                        part_data.material_factors.into(),
                    ));
                }
            }
        } else {
            for part_data in mesh_parts {
                if let Some(ref texture) = part_data.texture {
                    parts.push(MeshPart::textured_unlit(
                        device, mesh_pass,
                        &part_data.geometry,
                        part_data.material_factors.into(),
                        &texture,
                    ));
                } else {
                    parts.push(MeshPart::untextured(
                        device, mesh_pass,
                        &part_data.geometry,
                        part_data.material_factors.into(),
                    ));
                }
            }
        }

        Mesh {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Matrix4::identity(),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),

            parts,
            bind_group,
            uniform_buf,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
    pub fn uniform_buf(&self) -> &wgpu::Buffer { &self.uniform_buf }

    pub fn transform(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            self.position.x, self.position.y, self.position.z,
        )) *
            cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
            self.rotation
    }
}

