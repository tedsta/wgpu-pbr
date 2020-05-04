pub use camera::Camera;
pub use mesh::{Mesh, MeshPass, PointLightData, SpotLightData};
pub use renderer::Renderer;
pub use scene::{Scene, MeshId};
pub use mesh::{MeshPartData, MeshPartGeometry};
pub use self::gltf::GltfLoadError;
pub use resources::{ResourceLoader, Resources};

mod camera;
mod compute_tangents;
mod gltf;
mod mesh;
mod obj;
mod renderer;
mod scene;
mod resources;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
