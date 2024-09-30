use nalgebra_glm::Vec3;
use crate::Material;
use crate::ray_intersect::{RayIntersect, Intersect};


pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    // Constructor for Cube, now accepts a reference to Material
    pub fn new(min: Vec3, max: Vec3, material: &Material) -> Self {
        Cube {
            min,
            max,
            material: material.clone(),  // Clone the material to own it
        }
    }

    fn calculate_normal(&self, hit_point: Vec3) -> Vec3 {
        if (hit_point.x - self.min.x).abs() < 1e-4 {
            return Vec3::new(-1.0, 0.0, 0.0);
        }
        if (hit_point.x - self.max.x).abs() < 1e-4 {
            return Vec3::new(1.0, 0.0, 0.0);
        }
        if (hit_point.y - self.min.y).abs() < 1e-4 {
            return Vec3::new(0.0, -1.0, 0.0);
        }
        if (hit_point.y - self.max.y).abs() < 1e-4 {
            return Vec3::new(0.0, 1.0, 0.0);
        }
        if (hit_point.z - self.min.z).abs() < 1e-4 {
            return Vec3::new(0.0, 0.0, -1.0);
        }
        Vec3::new(0.0, 0.0, 1.0)
    }
}


impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_dir: &Vec3) -> Intersect {
        let mut tmin = (self.min.x - ray_origin.x) / ray_dir.x;
        let mut tmax = (self.max.x - ray_origin.x) / ray_dir.x;

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - ray_origin.y) / ray_dir.y;
        let mut tymax = (self.max.y - ray_origin.y) / ray_dir.y;

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if tmin > tymax || tymin > tmax {
            return Intersect::empty();
        }

        tmin = tmin.max(tymin);
        tmax = tmax.min(tymax);

        let mut tzmin = (self.min.z - ray_origin.z) / ray_dir.z;
        let mut tzmax = (self.max.z - ray_origin.z) / ray_dir.z;

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if tmin > tzmax || tzmin > tmax {
            return Intersect::empty();
        }

        tmin = tmin.max(tzmin);
        tmax = tmax.min(tzmax);

        if tmin < 0.0 && tmax < 0.0 {
            return Intersect::empty();
        }

        let intersection_point = ray_origin + ray_dir * tmin;

        Intersect {
            point: intersection_point,
            distance: tmin,
            normal: self.calculate_normal(intersection_point),
            material: self.material.clone(),
            is_intersecting: true,
        }
    }
}