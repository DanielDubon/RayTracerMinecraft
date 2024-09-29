use crate::color::Color;
use crate::texture::Texture; // Asegúrate de que la estructura Texture esté en texture.rs


#[derive(Debug, Clone)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<Texture>, // Añadido para manejar texturas
}


impl Material {
    pub fn new(
        diffuse: Color,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture: Option<Texture>, // Añadido como parámetro
    ) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture,
        }
    }


    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
            refractive_index: 0.0,
            texture: None, // Sin textura
        }
    }
}
