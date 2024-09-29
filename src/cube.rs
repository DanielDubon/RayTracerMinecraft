use nalgebra_glm::Vec3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

#[derive(Debug, Clone)] // Agregado para que Cube implemente Debug
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub top_material: Material,       // Material para la cara superior
    pub side_material: Material,      // Material para las caras laterales
    pub visible_faces: Vec<String>,   // Caras visibles
}

impl Cube {
    // Método para crear un cubo con un solo material para todas las caras
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube {
            min,
            max,
            top_material: material.clone(),
            side_material: material,
            visible_faces: vec!["top".to_string(), "left_right".to_string(), "front_back".to_string()],
        }
    }

    // Método para crear un cubo con materiales separados para la parte superior y los lados
    pub fn new_with_faces(min: Vec3, max: Vec3, top_material: Material, side_material: Material, visible_faces: Vec<String>) -> Self {
        Cube {
            min,
            max,
            top_material,
            side_material,
            visible_faces,
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let inv_dir = Vec3::new(1.0, 1.0, 1.0).component_div(ray_direction);
        let t1 = (self.min - ray_origin).component_mul(&inv_dir);
        let t2 = (self.max - ray_origin).component_mul(&inv_dir);

        let tmin = t1[0].min(t2[0]).max(t1[1].min(t2[1])).max(t1[2].min(t2[2]));
        let tmax = t1[0].max(t2[0]).min(t1[1].max(t2[1])).min(t1[2].max(t2[2]));

        if tmax < tmin || tmin < 0.0 {
            return Intersect::empty();
        }

        let point = ray_origin + ray_direction * tmin;
        let normal = if tmin == t1[0] { Vec3::new(-1.0, 0.0, 0.0) }
                    else if tmin == t2[0] { Vec3::new(1.0, 0.0, 0.0) }
                    else if tmin == t1[1] { Vec3::new(0.0, -1.0, 0.0) }
                    else if tmin == t2[1] { Vec3::new(0.0, 1.0, 0.0) }
                    else if tmin == t1[2] { Vec3::new(0.0, 0.0, -1.0) }
                    else { Vec3::new(0.0, 0.0, 1.0) };

        // Usa `self.clone()` para pasar el objeto
        Intersect::new(point, normal, tmin, self.top_material.clone(), self.clone()) 
    }
}
