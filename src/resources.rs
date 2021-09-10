use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct ResourceLoader<'d, 'q, 'r> {
    device: &'d mut wgpu::Device,
    queue: &'q mut wgpu::Queue,
    resources: &'r mut Resources,
}

impl<'d, 'q, 'r> ResourceLoader<'d, 'q, 'r> {
    pub fn new(
        device: &'d mut wgpu::Device,
        queue: &'q mut wgpu::Queue,
        resources: &'r mut Resources,
    ) -> Self {
        ResourceLoader {
            device, queue, resources,
        }
    }

    pub fn load_texture(
        &mut self,
        path: impl AsRef<Path>,
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        self.resources.load_texture(self.device, self.queue, path, srgb)
    }

    pub fn texture_from_bytes(
        &mut self,
        name: impl AsRef<Path>,
        texture_bytes: &[u8],
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        let ResourceLoader { device, queue, resources } = self;
        resources.textures.entry(name.as_ref().into())
            .or_insert_with(|| {
                Rc::new(Resources::texture_from_bytes(
                    device, queue, srgb, texture_bytes,
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
        queue: &mut wgpu::Queue,
        path: impl AsRef<Path>,
        srgb: bool,
    ) -> Rc<wgpu::Texture> {
        self.textures.entry(path.as_ref().into())
            .or_insert_with(|| Rc::new(Self::texture_from_path(device, queue, path, srgb)))
            .clone()
    }

    fn texture_from_path(
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        texture_path: impl AsRef<Path>,
        srgb: bool,
    ) -> wgpu::Texture {
        let img = match image::open(texture_path).unwrap() {
            image::DynamicImage::ImageRgba8(img) => img,
            img @ _ => img.to_rgba(),
        };

        Self::texture_to_gpu(device, queue, srgb, img)
    }

    fn texture_from_bytes(
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        srgb: bool,
        texture_bytes: &[u8],
    ) -> wgpu::Texture {
        // Create the texture
        let img = match image::load_from_memory(texture_bytes).unwrap() {
            image::DynamicImage::ImageRgba8(img) => img,
            img @ _ => img.to_rgba(),
        };

        Self::texture_to_gpu(device, queue, srgb, img)
    }

    fn texture_to_gpu(
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        srgb: bool,
        img: image::RgbaImage,
    ) -> wgpu::Texture {
        let size = img.width();
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&texels),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size),
                rows_per_image: std::num::NonZeroU32::new(size),
            },
            texture_extent,
        );

        texture
    }
}

