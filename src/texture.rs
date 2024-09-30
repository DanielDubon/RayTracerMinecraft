use crate::color::Color;

#[derive(Debug, Clone)] // Añade Debug y Clone aquí
pub struct Texture {
    pub data: Vec<u8>, // Datos de la textura
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(image_path: &str) -> Texture {
        let img = image::open(image_path).expect("Failed to load texture");
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        let data = img.into_raw(); // Convierte la imagen a un vector de bytes
        Texture { data, width, height }
    }

    pub fn get_color(&self, u: f32, v: f32) -> Color {
        // Convertir coordenadas UV a índices de píxel
        let x = (u * self.width as f32) as usize % self.width as usize;
        let y = (v * self.height as f32) as usize % self.height as usize;
        let index = (y * self.width as usize + x) * 4; // 4 para RGBA
        Color::new(
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
        )
    }
}
