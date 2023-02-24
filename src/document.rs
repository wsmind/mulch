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
    pub option: bool,     // just a test
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            rect: egui::Rect::NOTHING,
            option: true,
        }
    }
}
