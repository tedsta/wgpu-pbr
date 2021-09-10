use std::path::Path;

use crate::resources::{Resources, ResourceLoader};
use super::mesh::{Mesh, MeshPass, MeshPartData};
use super::obj::load_obj;
use super::gltf::{load_gltf, load_gltf_from_reader, load_gltf_single_mesh, GltfLoadError};
use super::scene::Scene;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub resources: Resources,
    pub mesh_pass: MeshPass,
}

impl Renderer {
    pub fn new(
        surface_config: &wgpu::SurfaceConfiguration,
        mut device: wgpu::Device,
        mut queue: wgpu::Queue,
    ) -> Renderer {
        let mesh_pass = MeshPass::init(surface_config, &mut device, &mut queue);

        Renderer {
            device,
            queue,
            resources: Resources::new(),
            mesh_pass,
        }
    }

    pub fn render(
        &mut self,
        render_target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
    ) {
        self.mesh_pass.render(&self.device, render_target, encoder, scene);
    }

    pub fn mesh_from_file(&mut self, path: impl AsRef<std::path::Path>, lighting: bool) -> Mesh {
        let mut mesh_parts = self.mesh_parts_from_file(path);
        for part in &mut mesh_parts {
            part.material.lighting = lighting;
        }
        self.mesh_from_parts(&mesh_parts)
    }

    pub fn mesh_from_parts(&mut self, parts: &[MeshPartData]) -> Mesh {
        Mesh::from_parts(
            &mut self.device,
            &self.mesh_pass,
            parts,
        )
    }

    pub fn mesh_parts_from_file(&mut self, path: impl AsRef<Path>) -> Vec<MeshPartData> {
        let mut resource_loader = ResourceLoader::new(
            &mut self.device, &mut self.queue, &mut self.resources,
        );

        let ext = path.as_ref().extension()
            .expect(&format!("Failed to load mesh '{}' - unknown file type.", path.as_ref().display()));
        let mesh_parts =
            if ext == "obj" {
                load_obj(&mut resource_loader, path)
            } else if ext == "glb" || ext == "gltf" {
                load_gltf(&mut resource_loader, path).expect("load gltf")
            } else {
                // TODO don't panic
                panic!("Failed to load mesh '{}' - unknown file type.", path.as_ref().display());
            };

        self.queue.submit(None);
        
        mesh_parts
    }

    pub fn gltf_mesh_parts_from_reader(
        &mut self,
        path: impl AsRef<Path>,
        reader: impl std::io::Read + std::io::Seek,
    ) -> Vec<MeshPartData> {
        let mut resource_loader = ResourceLoader::new(
            &mut self.device, &mut self.queue, &mut self.resources,
        );

        let mesh_parts =
            load_gltf_from_reader(&mut resource_loader, path, reader).expect("load gltf");

        self.queue.submit(None);
        
        mesh_parts
    }

    pub fn gltf_single_mesh_parts(
        &mut self, path: impl AsRef<Path>, mesh_name: &str,
    ) -> Result<Option<(Vec<MeshPartData>, gltf::scene::Transform)>, GltfLoadError> {
        let mut resource_loader = ResourceLoader::new(
            &mut self.device, &mut self.queue, &mut self.resources,
        );

        let maybe_mesh_parts = load_gltf_single_mesh(&mut resource_loader, path, mesh_name)?;

        self.queue.submit(None);

        Ok(maybe_mesh_parts)
    }
}

