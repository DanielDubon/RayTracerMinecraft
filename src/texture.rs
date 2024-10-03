use crate::color::Color;

#[derive(Debug, Clone)] // Añade Debug y Clone aquí
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Texture {
    pub fn load(path: &str) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();
        let pixels = img.pixels()
            .map(|p| Color::new(p[0], p[1], p[2]))
            .collect();

        Ok(Self { width, height, pixels })
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        self.pixels[(y * self.width + x) as usize]
    }
}
