use std::fmt;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    // Constructor to initialize the color using r, g, b values
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    // Function to create a color from a hex value
    pub const fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Color { r, g, b }
    }

    pub const fn black() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }

    // Function to return the color as a hex value
    pub fn to_hex(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn intensity(&self) -> f32 {
        (self.r as f32 + self.g as f32 + self.b as f32) / (3.0 * 255.0)
    }

    pub fn lerp(a: &Color, b: &Color, t: f32) -> Color {
        Color {
            r: (a.r as f32 * (1.0 - t) + b.r as f32 * t) as u8,
            g: (a.g as f32 * (1.0 - t) + b.g as f32 * t) as u8,
            b: (a.b as f32 * (1.0 - t) + b.b as f32 * t) as u8,
        }
    }

    pub fn mul(&self, other: &Color) -> Color {
        Color {
            r: ((self.r as f32 * other.r as f32) / 255.0) as u8,
            g: ((self.g as f32 * other.g as f32) / 255.0) as u8,
            b: ((self.b as f32 * other.b as f32) / 255.0) as u8,
        }
    }

    pub fn mul_scalar(&self, scalar: f32) -> Color {
        Color {
            r: (self.r as f32 * scalar).min(255.0) as u8,
            g: (self.g as f32 * scalar).min(255.0) as u8,
            b: (self.b as f32 * scalar).min(255.0) as u8,
        }
    }

    pub fn add(&self, other: &Color) -> Color {
        Color {
            r: (self.r as u16 + other.r as u16).min(255) as u8,
            g: (self.g as u16 + other.g as u16).min(255) as u8,
            b: (self.b as u16 + other.b as u16).min(255) as u8,
        }
    }
}

// Implement addition for Color
use std::ops::Add;

impl Add for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color {
            r: (self.r as u16 + other.r as u16).min(255) as u8,
            g: (self.g as u16 + other.g as u16).min(255) as u8,
            b: (self.b as u16 + other.b as u16).min(255) as u8,
        }
    }
}

impl Add<&Color> for Color {
    type Output = Color;

    fn add(self, other: &Color) -> Color {
        self.add(other)
    }
}

// Implement multiplication by a constant for Color
use std::ops::Mul;

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, scalar: f32) -> Color {
        Color {
            r: (self.r as f32 * scalar).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * scalar).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * scalar).clamp(0.0, 255.0) as u8,
        }
    }
}

// Implement display formatting for Color
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Color(r: {}, g: {}, b: {})", self.r, self.g, self.b)
    }
}
