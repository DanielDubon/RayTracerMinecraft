use crate::color::Color;

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,            // Color of the material (e.g., green for grass)
    pub shininess: f32,          // Specular shininess
    pub properties: [f32; 4],    // Material properties: [diffuse, specular, reflectivity, transparency]
    pub refractive_index: f32,   // Refractive index, useful for materials like glass or water
}

  impl Material {
    pub const fn new(color: Color, shininess: f32, properties: [f32; 4], refractive_index: f32) -> Self {
        Material {
            color,
            shininess,
            properties,
            refractive_index,
        }
    }


    // Method to create a black material with default values
    pub fn black() -> Self {
        Material {
            color: Color::new(0, 0, 0),    // Use integer values for Color
            shininess: 0.0,                 // Default shininess
            properties: [0.0, 0.0, 0.0, 0.0], // Default properties (all set to 0)
            refractive_index: 1.0,          // Default refractive index (e.g., for air)
        }
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
