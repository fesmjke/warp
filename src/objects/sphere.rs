use crate::hit::{Hit, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn from(center: Vec3, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit: &mut Hit) -> bool {
        let oc: Vec3 = ray.origin() - self.center;
        let a = ray.direction().length_squared();
        let b = Vec3::dot(&oc, &ray.direction());
        let c = oc.length_squared() - (self.radius * self.radius);
        let discriminant = b * b - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrt_discriminant = discriminant.sqrt();

        let mut root = (-b - sqrt_discriminant) / a;

        if root <= t_min || t_max <= root {
            root = (-b + sqrt_discriminant) / a;
            if root <= t_min || t_max <= root {
                return false;
            }
        }

        hit.t = root;
        hit.point = ray.at(hit.t);
        let outward_normal = (hit.point - self.center) / self.radius;
        hit.set_front_face(ray, &outward_normal);
        hit.material = self.material.clone();

        true
    }
}
