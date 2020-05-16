#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub pos: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SpotLight {
    pub pos: [f32; 3],
    pub angle: f32,
    pub color: [f32; 3],
    pub range: f32,
    pub dir: [f32; 3],
    pub smoothness: f32,
    pub intensity: f32,
}

