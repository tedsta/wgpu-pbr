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
    /// Create a new mesh from its "parts" (aka primitives). A `MeshPart` has a triangle mesh and a
    /// single material. A multi-material `Mesh` must be composed of multiple `MeshPart`s.
    pub fn from_parts(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
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
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
            ],
        });

        let mut parts = Vec::new();
        for part_data in mesh_parts {
            parts.push(MeshPart::new(
                device, mesh_pass,
                &part_data,
            ));
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

