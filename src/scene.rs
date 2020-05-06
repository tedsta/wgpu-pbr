use slotmap::DenseSlotMap;

use super::Camera;
use super::mesh::{Mesh, PointLightData, SpotLightData};

pub type MeshId = slotmap::DefaultKey;
pub type PointLightId = slotmap::DefaultKey;
pub type SpotLightId = slotmap::DefaultKey;

pub struct Scene {
    pub camera: Camera,
    pub meshes: DenseSlotMap<MeshId, Mesh>,
    pub point_lights: DenseSlotMap<PointLightId, PointLightData>,
    pub spot_lights: DenseSlotMap<SpotLightId, SpotLightData>,
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

    pub fn add_point_light(&mut self) -> PointLightId {
        self.point_lights.insert(PointLightData::zero())
    }

    pub fn add_spot_light(&mut self) -> SpotLightId {
        self.spot_lights.insert(SpotLightData::zero())
    }
}
