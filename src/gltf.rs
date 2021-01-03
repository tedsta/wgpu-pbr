use std::io;
use std::path::Path;
use std::rc::Rc;

use ultraviolet::{Bivec3, Mat4, Rotor3, Vec3};

use crate::resources::ResourceLoader;
use super::compute_tangents::compute_tangents;
use super::mesh::{Vertex, MeshPartData, MeshPartGeometry, MaterialData, MaterialFactors};

/// Load a single mesh by its node's name from a glTF file. If a mesh with the specified name
/// doesn't exist, `None` is returned. On success, a tuple of the mesh parts and the node's
/// transorm are returned.
pub fn load_gltf_single_mesh(
    resources: &mut ResourceLoader,
    path: impl AsRef<Path>,
    mesh_name: &str,
) -> Result<Option<(Vec<MeshPartData>, gltf::scene::Transform)>, GltfLoadError> {
    let base_path = path.as_ref().parent().expect("gltf base path");

    let file = std::fs::File::open(&path)?;
    let gltf = gltf::Gltf::from_reader(&file)?;

    let gltf_buffers = GltfBuffers::load_from_gltf(base_path, &gltf)?;

    for node in gltf.nodes() {
        if let Some(ref mesh) = node.mesh() {
            if node.name() != Some(mesh_name) { continue; }

            let mesh_parts = load_gltf_mesh(
                resources,
                &gltf,
                &gltf_buffers,
                &mesh,
                &path,
                base_path,
            )?;

            return Ok(Some((mesh_parts, node.transform())));
        }
    }

    Ok(None)
}

pub fn load_gltf(
    resources: &mut ResourceLoader,
    path: impl AsRef<Path>,
) -> Result<Vec<MeshPartData>, GltfLoadError> {
    let base_path = path.as_ref().parent().expect("gltf base path");

    let file = std::fs::File::open(&path)?;
    let gltf = gltf::Gltf::from_reader(&file)?;

    let gltf_buffers = GltfBuffers::load_from_gltf(base_path, &gltf)?;

    let mut mesh_parts: Vec<MeshPartData> = Vec::new();

    for node in gltf.nodes() {
        if let Some(ref mesh) = node.mesh() {
            let mut node_mesh_parts = load_gltf_mesh(
                resources,
                &gltf,
                &gltf_buffers,
                &mesh,
                &path,
                base_path,
            )?;

            let (part_pos, part_rot, part_scale) = node.transform().decomposed();

            let transform =
		Mat4::from_translation(Vec3::new(part_pos[0], part_pos[1], part_pos[2])) *
		    Mat4::from_nonuniform_scale(
                        Vec3::new(part_scale[0], part_scale[1], part_scale[2])
                    ) *
                    Rotor3::new(part_rot[3], Bivec3::new(-part_rot[2], part_rot[1], -part_rot[0]))
                        .into_matrix().into_homogeneous();

            for mesh_part in &mut node_mesh_parts {
                mesh_part.geometry.transform(transform);
            }

            mesh_parts.extend(node_mesh_parts);
        }
    }

    Ok(mesh_parts)
}

pub fn load_gltf_from_reader(
    resources: &mut ResourceLoader,
    path: impl AsRef<Path>,
    r: impl std::io::Read + std::io::Seek,
) -> Result<Vec<MeshPartData>, GltfLoadError> {
    let base_path = path.as_ref().parent().expect("gltf base path");

    let gltf = gltf::Gltf::from_reader(r)?;

    let gltf_buffers = GltfBuffers::load_from_gltf(base_path, &gltf)?;

    let mut mesh_parts: Vec<MeshPartData> = Vec::new();

    for node in gltf.nodes() {
        if let Some(ref mesh) = node.mesh() {
            mesh_parts.extend(load_gltf_mesh(
                resources,
                &gltf,
                &gltf_buffers,
                &mesh,
                &path,
                base_path,
            )?);
        }
    }

    Ok(mesh_parts)
}

////////////////////////////////////////

pub struct GltfBuffers {
    pub uri_buffers: Vec<Option<Vec<u8>>>,
}

impl GltfBuffers {
    pub fn load_from_gltf(base_path: impl AsRef<Path>, gltf: &gltf::Gltf) -> Result<Self, GltfLoadError> {
        use std::io::Read;
        use gltf::buffer::Source;

        let mut buffers = vec![];
        for (_index, buffer) in gltf.buffers().enumerate() {
            let data = match buffer.source() {
                Source::Uri(uri) => {
                    if uri.starts_with("data:") {
                        unimplemented!();
                    } else {
                        let mut file = std::fs::File::open(base_path.as_ref().join(uri))?;
                        let mut data: Vec<u8> = Vec::with_capacity(file.metadata()?.len() as usize);
                        file.read_to_end(&mut data)?;

                        assert!(data.len() >= buffer.length());

                        Some(data)
                    }
                }
                Source::Bin => {
                    None
                }
            };

            buffers.push(data);
        }
        Ok(GltfBuffers {
            uri_buffers: buffers,
        })
    }

    /// Obtain the contents of a loaded buffer.
    pub fn buffer<'a>(&'a self, gltf: &'a gltf::Gltf, buffer: &gltf::Buffer<'_>) -> Option<&'a [u8]> {
        use gltf::buffer::Source;

        match buffer.source() {
            Source::Uri(_) => {
                self.uri_buffers.get(buffer.index())
                    .map(Option::as_ref).flatten()
                    .map(Vec::as_slice)
            }
            Source::Bin => {
                gltf.blob.as_ref().map(Vec::as_slice)
            }
        }
    }

    /// Obtain the contents of a loaded buffer view.
    #[allow(unused)]
    pub fn view<'a>(&'a self, gltf: &'a gltf::Gltf, view: &gltf::buffer::View<'_>) -> Option<&'a [u8]> {
        self.buffer(gltf, &view.buffer()).map(|data| {
            let begin = view.offset();
            let end = begin + view.length();
            &data[begin..end]
        })
    }
}

////////////////////////////////////////

pub fn load_gltf_mesh(
    resources: &mut ResourceLoader,
    gltf: &gltf::Gltf,
    buffers: &GltfBuffers,
    mesh: &gltf::Mesh<'_>,
    path: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
) -> Result<Vec<MeshPartData>, GltfLoadError> {
    let mut mesh_parts: Vec<MeshPartData> = Vec::new();

    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buf_id| buffers.buffer(gltf, &buf_id));

        let indices = reader
            .read_indices()
            .ok_or(GltfLoadError::Message(format!("Mesh primitive does not contain indices")))?
            .into_u32()
            .collect::<Vec<u32>>();

        let positions = reader
            .read_positions()
            .ok_or(GltfLoadError::Message(format!("Primitive does not have positions")))?;
        let normals = reader
            .read_normals()
            .ok_or(GltfLoadError::Message(format!("Primitive does not have normals")))?;
        let uvs = reader
            .read_tex_coords(0)
            .ok_or(GltfLoadError::Message(format!("Primitive does not have tex coords")))?
            .into_f32();
        let tangents = reader.read_tangents();

        let geometry =
            if let Some(tangents) = tangents {
                let vertices = positions
                    .zip(normals.zip(tangents.zip(uvs)))
                    .map(|(pos, (norm, (tang, uv)))| Vertex {
                        pos: pos.into(),
                        norm: norm.into(),
                        tang: tang.into(),
                        tex_coord: uv.into(),
                    })
                    .collect::<Vec<_>>();

                MeshPartGeometry {
                    vertices,
                    indices,
                }
            } else {
                let vertices = positions
                    .zip(normals.zip(uvs))
                    .map(|(pos, (norm, uv))| Vertex {
                        pos: pos.into(),
                        norm: norm.into(),
                        tang: [0.0, 0.0, 0.0, 0.0],
                        tex_coord: uv.into(),
                    })
                    .collect::<Vec<_>>();
                let mut geometry = MeshPartGeometry {
                    vertices,
                    indices,
                };
                compute_tangents(&mut geometry);

                geometry
            };

        let material = primitive.material();

        let pbr_met_rough = material.pbr_metallic_roughness();

        let albedo = pbr_met_rough.base_color_texture().map(|t| {
            load_gltf_texture(
                resources,
                gltf, buffers,
                &path,
                &base_dir,
                t.texture(),
                true,
            )
        });

        let metallic_roughness = pbr_met_rough.metallic_roughness_texture().map(|t| {
            load_gltf_texture(
                resources,
                gltf, buffers,
                &path,
                &base_dir,
                t.texture(),
                false,
            )
        });
            
        let normal = material.normal_texture().map(|t| {
            load_gltf_texture(
                resources,
                gltf, buffers,
                &path,
                &base_dir,
                t.texture(),
                false,
            )
        });

        let ao = material.occlusion_texture().map(|t| {
            load_gltf_texture(
                resources,
                gltf, buffers,
                &path,
                &base_dir,
                t.texture(),
                false,
            )
        });

        let emissive = material.emissive_texture().map(|emissive_info| {
            load_gltf_texture(resources, gltf, buffers, &path, &base_dir, emissive_info.texture(), true)
        });

        mesh_parts.push(MeshPartData {
            geometry,
            material: MaterialData {
                factors: MaterialFactors {
                    diffuse: pbr_met_rough.base_color_factor(),
                    metal: pbr_met_rough.metallic_factor(),
                    rough: pbr_met_rough.roughness_factor(),
                    emissive: material.emissive_factor(),
                    extra_emissive: [0.0, 0.0, 0.0],
                },
                lighting: true,
                texture: albedo,
                normal: normal,
                metallic_roughness: metallic_roughness,
                ao: ao,
                emissive: emissive,
            },
        })
    }

    Ok(mesh_parts)
}

fn load_gltf_texture(
    resources: &mut ResourceLoader,
    gltf: &gltf::Gltf,
    buffers: &GltfBuffers,
    path: impl AsRef<Path>,
    base_dir: impl AsRef<Path>,
    texture: gltf::Texture<'_>,
    srgba: bool,
) -> Rc<wgpu::Texture> {
    match texture.source().source() {
        gltf::image::Source::View { ref view, mime_type: _ } => {
            let texture_name = base_dir.as_ref().join(path).join(format!("texture{}", texture.index()));
            let bytes = buffers.view(gltf, view).expect("texture view bytes");
            resources.texture_from_bytes(texture_name, bytes, srgba)
        }
        gltf::image::Source::Uri { uri, .. } => {
            resources.load_texture(base_dir.as_ref().join(uri), srgba)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum GltfLoadError {
    Io(std::io::Error),
    Gltf(gltf::Error),
    Message(String),
}

impl std::error::Error for GltfLoadError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            GltfLoadError::Io(ref err) => Some(err),
            GltfLoadError::Gltf(ref err) => Some(err),
            GltfLoadError::Message(_) => None,
        }
    }
}

impl std::fmt::Display for GltfLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GltfLoadError::Io(ref err) => write!(f, "GLTF load IO error: {}", err),
            GltfLoadError::Gltf(ref err) => write!(f, "GLTF load parse error: {}", err),
            GltfLoadError::Message(ref err) => write!(f, "GLTF load error: {}", err),
        }
    }
}

impl From<std::io::Error> for GltfLoadError {
    fn from(err: io::Error) -> GltfLoadError {
        GltfLoadError::Io(err)
    }
}

impl From<gltf::Error> for GltfLoadError {
    fn from(err: gltf::Error) -> GltfLoadError {
        GltfLoadError::Gltf(err)
    }
}
