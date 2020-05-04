use cgmath::{self, SquareMatrix};

pub struct Camera {
    pub proj: cgmath::Matrix4<f32>,

    position: cgmath::Point3<f32>,
    view: cgmath::Matrix4<f32>,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let proj = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 0.001, 1000.0);

        Camera {
            proj,

            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            view: cgmath::Matrix4::identity(),
        }
    }

    pub fn resize(&mut self, aspect_ratio: f32) {
        self.proj = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 0.001, 1000.0);
    }

    pub fn look_at(
        &mut self,
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
    ) {
        self.position = eye;
        self.view = cgmath::Matrix4::look_at(eye, target, up);
    }

    pub fn total_matrix(&self) -> cgmath::Matrix4<f32> {
        self.proj * self.view
    }

    pub fn position(&self) -> &cgmath::Point3<f32> {
        &self.position
    }

    pub fn project_world_to_screen(
        &self,
        viewport: cgmath::Vector4<i32>,
        world: cgmath::Vector3<f32>,
    ) -> Option<cgmath::Vector3<f32>> {
        let screen = (self.proj * self.view) * world.extend(1.0);

        if screen.w != 0.0 {
            let mut screen = screen.truncate() * (1.0 / screen.w);

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

    pub fn project_screen_to_world(
        &self,
        screen: cgmath::Vector3<f32>,
        viewport: cgmath::Vector4<i32>,
    ) -> Option<cgmath::Vector3<f32>> {
        let view_projection = self.proj * self.view;
        if let Some(inv_view_projection) = view_projection.invert() {
            let world = cgmath::Vector4::new(
                (screen.x - (viewport.x as f32)) / (viewport.z as f32) * 2.0 - 1.0,
                (1.0 - (screen.y - (viewport.y as f32)) / (viewport.w as f32)) * 2.0 - 1.0,
                screen.z,
                1.0
            );
            let world = inv_view_projection * world;

            if world.w != 0.0 {
                Some(world.truncate() * (1.0 / world.w))
            } else {
                None
            }
        } else {
            None
        }
    }
}

