use slotmap::DenseSlotMap;

use super::{Camera, PointLight, SpotLight};
use super::mesh::Mesh;

pub type MeshId = slotmap::DefaultKey;
pub type PointLightId = slotmap::DefaultKey;
pub type SpotLightId = slotmap::DefaultKey;

pub struct Scene {
    pub camera: Camera,

    pub(crate) meshes: DenseSlotMap<MeshId, Mesh>,
    pub(crate) point_lights: DenseSlotMap<PointLightId, PointLight>,
    pub(crate) spot_lights: DenseSlotMap<SpotLightId, SpotLight>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            camera,
            meshes: DenseSlotMap::new(),
            point_lights: DenseSlotMap::new(),
            spot_lights: DenseSlotMap::new(),
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshId {
        self.meshes.insert(mesh)
    }

    pub fn remove_mesh(&mut self, id: MeshId) {
        self.meshes.remove(id);
    }

    pub fn mesh(&mut self, id: MeshId) -> &mut Mesh {
        &mut self.meshes[id]
    }

    pub fn mesh_mut(&mut self, id: MeshId) -> &mut Mesh {
        &mut self.meshes[id]
    }

    ////////////////////////////////////

    pub fn add_point_light(&mut self, point_light: PointLight) -> PointLightId {
        self.point_lights.insert(point_light)
    }

    pub fn remove_point_light(&mut self, id: PointLightId) {
        self.point_lights.remove(id);
    }

    pub fn point_light(&mut self, id: PointLightId) -> &mut PointLight {
        &mut self.point_lights[id]
    }

    ////////////////////////////////////

    pub fn add_spot_light(&mut self, spot_light: SpotLight) -> SpotLightId {
        self.spot_lights.insert(spot_light)
    }

    pub fn remove_spot_light(&mut self, id: SpotLightId) {
        self.spot_lights.remove(id);
    }

    pub fn spot_light(&mut self, id: SpotLightId) -> &mut SpotLight {
        &mut self.spot_lights[id]
    }
}
