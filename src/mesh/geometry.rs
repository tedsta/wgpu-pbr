use ultraviolet::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub norm: [f32; 3],
    pub tang: [f32; 4],
    pub tex_coord: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex { }
unsafe impl bytemuck::Zeroable for Vertex { }

#[derive(Clone)]
pub struct MeshPartGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshPartGeometry {
    pub fn bounding_box(&self) -> ([f32; 3], [f32; 3]) {
        let mut mins = self.vertices[0].pos;
        let mut maxes = self.vertices[0].pos;
        for vertex in &self.vertices[1..] {
            mins[0] = f32::min(mins[0], vertex.pos[0]);
            mins[1] = f32::min(mins[1], vertex.pos[1]);
            mins[2] = f32::min(mins[2], vertex.pos[2]);

            maxes[0] = f32::max(maxes[0], vertex.pos[0]);
            maxes[1] = f32::max(maxes[1], vertex.pos[1]);
            maxes[2] = f32::max(maxes[2], vertex.pos[2]);
        }

        (mins, maxes)
    }

    pub fn transform(&mut self, transform: Mat4) {
        for vertex in &mut self.vertices {
            let position = Vec3::new(vertex.pos[0], vertex.pos[1], vertex.pos[2]);
            let transformed_position = transform.transform_point3(position);
            vertex.pos = [
                transformed_position.x,
                transformed_position.y,
                transformed_position.z,
            ];
        }
    }
}

