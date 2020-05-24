use ultraviolet::{Mat4, Rotor3, Vec3};

use super::{
    mesh_part::{MeshPart, MeshPartData},
    mesh_pass::MeshPass,
};

pub struct Mesh {
    pub position: Vec3,
    pub rotation: Rotor3,
    pub scale: Vec3,
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
        let transform = Mat4::identity();
        let transform_ref: &[f32; 16] = transform.as_array();
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
            position: Vec3::zero(),
            rotation: Rotor3::identity(),
            scale: Vec3::broadcast(1.0),

            parts,
            bind_group,
            uniform_buf,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
    pub fn uniform_buf(&self) -> &wgpu::Buffer { &self.uniform_buf }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(self.position) *
            Mat4::from_nonuniform_scale(self.scale.into()) *
            self.rotation.into_matrix().into_homogeneous()
    }
}

