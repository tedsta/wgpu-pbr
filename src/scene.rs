use super::Camera;
use super::mesh::{Mesh, LightData, SpotLightData};

pub type MeshId = slotmap::DefaultKey;

pub struct Scene {
    pub camera: Camera,
    pub meshes: slotmap::DenseSlotMap<MeshId, Mesh>,
    pub point_lights: Vec<LightData>,
    pub spot_lights: Vec<SpotLightData>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            camera,
            meshes: slotmap::DenseSlotMap::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.insert(mesh)
    }
}
