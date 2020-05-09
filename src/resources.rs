use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct ResourceLoader<'d, 'c, 'r> {
    device: &'d mut wgpu::Device,
    encoder: &'c mut wgpu::CommandEncoder,
    resources: &'r mut Resources,
}

impl<'d, 'c, 'r> ResourceLoader<'d, 'c, 'r> {
    pub fn new(
        device: &'d mut wgpu::Device,
        encoder: &'c mut wgpu::CommandEncoder,
        resources: &'r mut Resources,
    ) -> Self {
        ResourceLoader {
            device, encoder, resources,
        }
    }

    pub fn load_texture(
        &mut self,
        path: impl AsRef<Path>,
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        self.resources.load_texture(self.device, self.encoder, path, srgb)
    }

    pub fn texture_from_bytes(
        &mut self,
        name: impl AsRef<Path>,
        texture_bytes: &[u8],
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        let ResourceLoader { device, encoder, resources } = self;
        resources.textures.entry(name.as_ref().into())
            .or_insert_with(|| {
                Rc::new(Resources::texture_from_bytes(
                    device, encoder, srgb, texture_bytes,
                ))
            })
            .clone()
    }
}

pub struct Resources {
    textures: HashMap<PathBuf, Rc<wgpu::Texture>>,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: impl AsRef<Path>,
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        self.textures.entry(path.as_ref().into())
            .or_insert_with(|| Rc::new(Self::texture_from_path(device, encoder, path, srgb)))
            .clone()
    }

    fn texture_from_path(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture_path: impl AsRef<Path>,
        srgb: bool,
    ) -> wgpu::Texture {
        let img = match image::open(texture_path).unwrap() {
            image::DynamicImage::ImageRgba8(img) => img,
            img @ _ => img.to_rgba(),
        };

        Self::texture_to_gpu(device, encoder, srgb, img)
    }

    fn texture_from_bytes(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        srgb: bool,
        texture_bytes: &[u8],
    ) -> wgpu::Texture {
        // Create the texture
        let img = match image::load_from_memory(texture_bytes).unwrap() {
            image::DynamicImage::ImageRgba8(img) => img,
            img @ _ => img.to_rgba(),
        };

        Self::texture_to_gpu(device, encoder, srgb, img)
    }

    fn texture_to_gpu(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        srgb: bool,
        img: image::RgbaImage,
    ) -> wgpu::Texture {
        let size = img.width();
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        let texels = img.into_raw();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if srgb {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let mut temp_buf = device.create_buffer_mapped(&wgpu::BufferDescriptor {
            label: Some("Resources::texture_to_gpu::temp_buf"),
            size: texels.len() as u64,
            usage: wgpu::BufferUsage::COPY_SRC,
        });
        temp_buf.data().copy_from_slice(&texels);

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf.finish(),
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: size,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: 0,
                },
            },
            texture_extent,
        );

        texture
    }
}

