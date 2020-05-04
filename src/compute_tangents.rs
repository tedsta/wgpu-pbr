use super::mesh::MeshPartGeometry;

pub fn compute_tangents(geometry: &mut MeshPartGeometry) {
    mikktspace::generate_tangents(geometry);
}

impl mikktspace::Geometry for MeshPartGeometry {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let i = self.indices[face*3 + vert] as usize;
        self.vertices[i].pos
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let i = self.indices[face*3 + vert] as usize;
        self.vertices[i].norm
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let i = self.indices[face*3 + vert] as usize;
        self.vertices[i].tex_coord
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let i = self.indices[face*3 + vert] as usize;
        self.vertices[i].tang = tangent;
    }
}

