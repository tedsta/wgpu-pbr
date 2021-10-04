use wgpu::util::DeviceExt;

use super::{
    geometry::MeshPartGeometry,
    material::{Material, MaterialData},
    mesh_pass::MeshPass,
};

#[derive(Clone)]
pub struct MeshPartData {
    pub geometry: MeshPartGeometry,
    pub material: MaterialData,
}

pub struct MeshPart {
    pub material: Material,

    // Geometry
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
}

impl MeshPart {
    pub fn new(
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
        data: &MeshPartData,
    ) -> Self {
        let material = Material::new(device, mesh_pass, &data.material);

        // Create the vertex and index buffers
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data.geometry.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data.geometry.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        MeshPart {
            material,
            vertex_buf, index_buf,
            index_count: data.geometry.indices.len(),
        }
    }

    pub fn vertex_buf(&self) -> &wgpu::Buffer { &self.vertex_buf }
    pub fn index_buf(&self) -> &wgpu::Buffer { &self.index_buf }
    pub fn index_count(&self) -> usize { self.index_count }
}

pub fn mesh_parts_bbox(parts: &[MeshPartData]) -> ([f32; 3], [f32; 3]) {
    let (mut mins, mut maxes) = parts[0].geometry.bounding_box();

    for part in &parts[1..] {
        let (mut part_mins, mut part_maxes) = parts[0].geometry.bounding_box();

        mins[0] = f32::min(mins[0], part_mins[0]);
        mins[1] = f32::min(mins[1], part_mins[1]);
        mins[2] = f32::min(mins[2], part_mins[2]);

        maxes[0] = f32::max(maxes[0], part_maxes[0]);
        maxes[1] = f32::max(maxes[1], part_maxes[1]);
        maxes[2] = f32::max(maxes[2], part_maxes[2]);
    }

    (mins, maxes)
}

