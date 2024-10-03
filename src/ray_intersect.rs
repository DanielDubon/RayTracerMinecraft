use nalgebra_glm::Vec3;
use crate::material::Material;

#[derive(Debug, Clone)]
pub struct Intersect {
    pub is_intersecting: bool,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
    pub face: CubeFace,
}

#[derive(Debug, Clone)]
pub enum CubeFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Intersect {
    pub fn new() -> Self {
        Self {
            is_intersecting: false,
            point: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            distance: f32::MAX,
            material: Material::black(),
            face: CubeFace::Top,  // Default to Top, or whichever face makes sense as a default
        }
    }

    pub fn empty() -> Self {
        Self {
            is_intersecting: false,
            point: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 0.0),
            distance: f32::MAX,
            material: Material::black(),
            face: CubeFace::Top,  // or any default face you prefer
        }
    }

    pub fn texture_coords(&self) -> (f32, f32) {
        match self.face {
            CubeFace::Top | CubeFace::Bottom => {
                let u = self.point.x.fract().abs();
                let v = self.point.z.fract().abs();
                (u, v)
            },
            CubeFace::Left | CubeFace::Right => {
                let u = self.point.z.fract().abs();
                let v = self.point.y.fract().abs();
                (u, v)
            },
            CubeFace::Front | CubeFace::Back => {
                let u = self.point.x.fract().abs();
                let v = self.point.y.fract().abs();
                (u, v)
            },
        }
    }
}

pub trait RayIntersect {
  fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}


