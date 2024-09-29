use nalgebra_glm::Vec3;
use crate::material::Material;
use crate::cube::Cube;

// Cambiamos `Intersect` para que contenga el objeto `Cube`
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub is_intersecting: bool,
    pub material: Material,
    pub object: Cube, // Cambiado a `Cube` en lugar de `&Cube`
}

impl Intersect {
    pub fn new(point: Vec3, normal: Vec3, distance: f32, material: Material, object: Cube) -> Self {
        Intersect {
            point,
            normal,
            distance,
            is_intersecting: true,
            material,
            object, // Guardamos el objeto
        }
    }

    pub fn empty() -> Self {
        Intersect {
            point: Vec3::zeros(),
            normal: Vec3::zeros(),
            distance: 0.0,
            is_intersecting: false,
            material: Material::black(),
            object: Cube::new(Vec3::zeros(), Vec3::zeros(), Material::black()), // Ahora está bien
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
