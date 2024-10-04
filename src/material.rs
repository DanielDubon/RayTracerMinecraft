use crate::color::Color;
use crate::texture::Texture;

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub shininess: f32,
    pub properties: [f32; 4],
    pub refractive_index: f32,
    pub textures: Vec<Texture>,
    pub emission: Color,
}

impl Material {
    pub fn new(color: Color, shininess: f32, properties: [f32; 4], refractive_index: f32) -> Self {
        Material {
            color,
            shininess,
            properties,
            refractive_index,
            textures: Vec::new(),
            emission: Color::new(0, 0, 0), // Por defecto, no emite luz
        }
    }

    pub fn with_emission(mut self, emission: Color) -> Self {
        self.emission = emission;
        self
    }

    // Method to create a black material with default values
    pub fn black() -> Self {
        Material {
            color: Color::new(0, 0, 0),    // Use integer values for Color
            shininess: 0.0,                 // Default shininess
            properties: [0.0, 0.0, 0.0, 0.0], // Default properties (all set to 0)
            refractive_index: 1.0,          // Default refractive index (e.g., for air)
            textures: Vec::new(),            // Empty textures vector
            emission: Color::new(0, 0, 0),   // No emission for black material
        }
    }

    pub fn with_textures(mut self, textures: Vec<Texture>) -> Self {
        self.textures = textures;
        self
    }

    // Method to determine if the material is completely diffuse (no shininess)
    pub fn is_diffuse(&self) -> bool {
        self.properties[1] == 0.0 && self.properties[2] == 0.0
    }

    // Method to determine if the material is reflective
    pub fn is_reflective(&self) -> bool {
        self.properties[2] > 0.0
    }

    // Method to determine if the material is transparent
    pub fn is_transparent(&self) -> bool {
        self.properties[3] > 0.0
    }
}
