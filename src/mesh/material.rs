use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialFactors {
    pub diffuse: [f32; 4],
    pub metal: f32,
    pub rough: f32,
    pub emissive: [f32; 3],
    pub extra_emissive: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MaterialFactorsUpload {
    pub diffuse: [f32; 4],
    pub metal: f32,
    pub pad0: [u32; 3],
    pub rough: f32,
    pub pad1: [u32; 3],
    pub emissive: [f32; 3],
    pub pad2: [u32; 1],
    pub extra_emissive: [f32; 3],
    pub pad3: [u32; 1],
}

unsafe impl Pod for MaterialFactorsUpload { }
unsafe impl Zeroable for MaterialFactorsUpload { }

impl Default for MaterialFactors {
    fn default() -> Self {
        MaterialFactors {
            diffuse: [1.0, 1.0, 1.0, 1.0],
            metal: 1.0,
            rough: 1.0,
            emissive: [1.0, 1.0, 1.0],
            extra_emissive: [0.0, 0.0, 0.0],
        }
    }
}

impl From<MaterialFactors> for MaterialFactorsUpload {
    fn from(v: MaterialFactors) -> Self {
        MaterialFactorsUpload {
            diffuse: v.diffuse,
            metal: v.metal,
            rough: v.rough,
            emissive: v.emissive,
            extra_emissive: v.extra_emissive,

            pad0: [0; 3],
            pad1: [0; 3],
            pad2: [0; 1],
            pad3: [0; 1],
        }
    }
}

