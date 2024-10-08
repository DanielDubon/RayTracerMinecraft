use nalgebra_glm::Vec3;
use crate::color::Color;

#[derive(Clone, Copy)]
pub struct Light {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,  // Radio de influencia de la luz
}

impl Light {
    pub fn new(position: Vec3, color: Color, intensity: f32, radius: f32) -> Self {
        Light { position, color, intensity, radius }
    }
}