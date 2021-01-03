use ultraviolet::{self, Mat4, Vec3, Vec4};

pub struct Camera {
    pub proj: Mat4,

    position: Vec3,
    view: Mat4,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let proj = ultraviolet::projection::rh_yup::perspective_gl(
            f32::to_radians(45.0), aspect_ratio, 0.001, 1000.0,
        );

        Camera {
            proj,

            position: Vec3::zero(),
            view: Mat4::identity(),
        }
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        self.proj = ultraviolet::projection::rh_yup::perspective_gl(
            f32::to_radians(45.0), aspect_ratio, 0.001, 1000.0,
        );
    }

    pub fn look_at(
        &mut self,
        eye: Vec3,
        target: Vec3,
        up: Vec3,
    ) {
        self.position = eye;
        self.view = Mat4::look_at(eye, target, up);
    }

    pub fn total_matrix(&self) -> Mat4 {
        self.proj * self.view
    }

    pub fn position(&self) -> &Vec3 {
        &self.position
    }

    pub fn project_world_to_screen(&self, viewport: Vec4, world: Vec3) -> Option<Vec3> {
        let screen = (self.proj * self.view) * world.into_homogeneous_point();

        if screen.w != 0.0 {
            let mut screen = screen.xyz() * (1.0 / screen.w);

            screen.x = (screen.x + 1.0) * 0.5 * (viewport.z as f32) + (viewport.x as f32);
            // Screen Origin is Top Left    (Mouse Origin is Top Left)
            screen.y = (screen.y + 1.0) * 0.5 * (viewport.w as f32) + (viewport.y as f32);
            // Screen Origin is Bottom Left (Mouse Origin is Top Left)
            //screen.y = (1.0 - screen.y) * 0.5 * (viewport.w as f32) + (viewport.y as f32);

            // This is only correct when glDepthRangef(0.0f, 1.0f)
            screen.z = (screen.z + 1.0) * 0.5;

            Some(screen)
        } else {
            None
        }
    }

    pub fn project_screen_to_world(&self, screen: Vec3, viewport: Vec4) -> Option<Vec3> {
        let mut view_projection = self.proj * self.view;

        // TODO verify invertable?
        view_projection.inverse();

        let world = Vec4::new(
            (screen.x - (viewport.x as f32)) / (viewport.z as f32) * 2.0 - 1.0,
            (1.0 - (screen.y - (viewport.y as f32)) / (viewport.w as f32)) * 2.0 - 1.0,
            screen.z,
            1.0
        );
        let world = view_projection * world;

        if world.w != 0.0 {
            Some(world.xyz() * (1.0 / world.w))
        } else {
            None
        }
    }
}

