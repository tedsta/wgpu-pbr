use std::path::Path;

use super::resources::ResourceLoader;
use super::compute_tangents::compute_tangents;
use super::mesh::{Vertex, MeshPartData, MeshPartGeometry, MaterialData, MaterialFactors};

pub fn load_obj(
    resources: &mut ResourceLoader,
    path: impl AsRef<Path>,
) -> Vec<MeshPartData> {
    let triangulate = true;
    let (models, materials) = tobj::load_obj(path.as_ref(), triangulate).expect("load obj");

    let mut parts = Vec::new();
    for model in models.iter() {
        let mesh = &model.mesh;
        // Normals and texture coordinates are also loaded, but not printed in this example
        assert!(mesh.positions.len() % 3 == 0);
        assert!(mesh.texcoords.len() % 2 == 0);
        assert!(mesh.texcoords.len() / 2 == mesh.positions.len() / 3);
        assert!(mesh.normals.len() == mesh.positions.len());

        let mut vertices = Vec::new();
        for v in 0..mesh.positions.len() / 3 {
            vertices.push(Vertex {
                pos: [
                    mesh.positions[3 * v],
                    mesh.positions[3 * v + 1],
                    mesh.positions[3 * v + 2],
                ],
                norm: [
                    mesh.normals[3 * v],
                    mesh.normals[3 * v + 1],
                    mesh.normals[3 * v + 2],
                ],
                tang: [0.0, 0.0, 0.0, 0.0],
                tex_coord: [mesh.texcoords[2 * v], 1.0 - mesh.texcoords[2 * v + 1]],
            });
        }

        let mut geometry = MeshPartGeometry { vertices, indices: mesh.indices.clone() };
        compute_tangents(&mut geometry);

        let path_prefix = path.as_ref().parent()
            .map(|p| p.to_owned())
            .unwrap_or("".into());

        let (
            diffuse, texture_path, normal_path,
            metallic_roughness_path, ao_path, emissive_path,
        ) = if let Some(material_id) = mesh.material_id {
            let diffuse_texture = materials[material_id].diffuse_texture.clone();
            let texture_path = if diffuse_texture.len() > 0 {
                Some(path_prefix.join(diffuse_texture))
            } else {
                None
            };

            let normal_path = materials[material_id].unknown_param.get("norm")
                .or(materials[material_id].unknown_param.get("map_Bump"))
                .map(|p| path_prefix.join(p));

            let metallic_roughness_path =
                materials[material_id].unknown_param.get("metallic_roughness").or(
                    materials[material_id].unknown_param.get("metal_rough")
                ).map(|p| path_prefix.join(p));

            let ao_path =
                materials[material_id].unknown_param.get("ao").map(|p| path_prefix.join(p));

            let emissive_path =
                materials[material_id].unknown_param.get("emissive").map(|p| path_prefix.join(p));
            (
                [
                    materials[material_id].diffuse[0],
                    materials[material_id].diffuse[1],
                    materials[material_id].diffuse[2],
                    1.0,
                ],
                texture_path,
                normal_path,
                metallic_roughness_path,
                ao_path,
                emissive_path,
            )
        } else {
            ([1.0, 1.0, 1.0, 1.0], None, None, None, None, None)
        };

        parts.push(MeshPartData {
            geometry,
            material: MaterialData {
                factors: MaterialFactors {
                    diffuse: diffuse,
                    metal: 1.0,
                    rough: 1.0,
                    emissive: [1.0, 1.0, 1.0],
                    extra_emissive: [0.0, 0.0, 0.0],
                },
                lighting: true,
                texture: texture_path.map(|p| resources.load_texture(p, true)),
                normal: normal_path.map(|p| resources.load_texture(p, false)),
                metallic_roughness: metallic_roughness_path.map(|p| resources.load_texture(p, false)),
                ao: ao_path.map(|p| resources.load_texture(p, false)),
                emissive: emissive_path.map(|p| resources.load_texture(p, true)),
            },
        });
    }

    parts
}

