pub struct Document {
    pub layers: Vec<Layer>,
    pub viewport: Viewport,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            layers: vec![
                Layer {
                    name: "Plop".to_string(),
                },
                Layer {
                    name: "OtherLayer".to_string(),
                },
                Layer {
                    name: "Hello!".to_string(),
                },
            ],
            viewport: Viewport::default(),
        }
    }
}

pub struct Layer {
    pub name: String,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: "Layer".to_string(),
        }
    }
}

pub struct Viewport {
    pub rect: egui::Rect, // in points
    pub grid_enabled: bool,
    pub camera: Camera,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            rect: egui::Rect::NOTHING,
            grid_enabled: true,
            camera: Camera::default(),
        }
    }
}

pub struct Camera {
    pub position: glam::Vec3,
    pub pitch: f32,
    pub yaw: f32,

    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: (-2.0, -3.0, 1.6).into(),
            pitch: -0.5,
            yaw: -0.6,

            fovy: 1.2,
            near: 0.01,
            far: 1000.0,
        }
    }
}

impl Camera {
    pub fn translate_local_frame(&mut self, offset: glam::Vec3) {
        let (view, _) = self.compute_matrices(1.0);

        self.position += glam::Mat3::from_mat4(view).transpose() * offset;
    }

    pub fn compute_matrices(&self, aspect_ratio: f32) -> (glam::Mat4, glam::Mat4) {
        let direction = glam::Mat3::from_rotation_z(self.yaw)
            * glam::Mat3::from_rotation_x(self.pitch)
            * glam::Vec3::Y;

        let view = glam::Mat4::look_to_rh(self.position, direction, glam::Vec3::Z);

        let projection = glam::Mat4::perspective_rh(self.fovy, aspect_ratio, self.near, self.far);

        (view, projection)
    }
}
