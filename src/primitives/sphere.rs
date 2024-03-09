use crate::intersections::{Intersectable, Intersection, Intersections};
use crate::matrices::{Matrix, Matrix4};
use crate::point::Point;
use crate::primitives::Shape;
use crate::ray::Ray;
use crate::transformations::Transformable;
use crate::vector::Vector3;

#[derive(Debug, PartialEq, Clone)]
pub struct Sphere {
    pub origin: Point,
    pub radius: f64,
    transformation: Matrix4,
}

impl Sphere {
    pub fn new(origin: Point, radius: f64) -> Self {
        Self {
            origin,
            radius,
            transformation: Matrix4::identity(),
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Intersections {
        let ray = ray.transform(&self.transformation.invert());
        let sphere_to_ray = ray.origin - Point::default();
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * ray.direction.dot_point(&sphere_to_ray);
        let c = sphere_to_ray.dot(&sphere_to_ray) - 1.0;

        let discriminant = b.powi(2) - (4.0 * a * c);

        if discriminant < 0.0 {
            Intersections::new()
        } else {
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

            Intersections::new().with(vec![
                Intersection::new(t1, &self),
                Intersection::new(t2, &self),
            ])
        }
    }
}

impl Shape for Sphere {
    fn normal(&self, world: Point) -> Vector3 {
        let object_point = self.transformation.invert() * world;
        let object_normal = object_point - Point::default();
        let world_normal = self.transformation.invert().transpose() * object_normal;
        Vector3::new(world_normal.x, world_normal.y, world_normal.z).normalize()
    }
}

impl Transformable for Sphere {
    fn transform(self, transformation: &Matrix4) -> Sphere {
        Self {
            transformation: transformation.clone() * self.transformation,
            ..self
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            origin: Point::default(),
            radius: 1.0,
            transformation: Matrix4::identity(),
        }
    }
}

#[cfg(test)]
mod sphere_tests {
    use crate::float_eq::ApproxEq;
    use crate::intersections::{Intersectable, Intersection, Intersections};
    use crate::matrices::{Matrix, Matrix4};
    use crate::point::Point;
    use crate::primitives::sphere::Sphere;
    use crate::primitives::Shape;
    use crate::ray::Ray;
    use crate::transformations::{Over, Transform, Transformable};
    use crate::vector::Vector3;
    use std::f64::consts::PI;

    #[test]
    fn sphere_creation() {
        let sphere = Sphere::new(Point::new(0.0, 0.0, 0.0), 1.0);
        let expected_origin = Point::default();
        let expected_radius = 1.0;

        assert_eq!(expected_origin, sphere.origin);
        assert_eq!(expected_radius, sphere.radius);
    }

    #[test]
    fn sphere_creation_default() {
        let sphere = Sphere::default();
        let expected_origin = Point::default();
        let expected_radius = 1.0;

        assert_eq!(expected_origin, sphere.origin);
        assert_eq!(expected_radius, sphere.radius);
    }

    #[test]
    fn ray_intersect_sphere_at_two_points() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![
            Intersection::new(4.0, &sphere),
            Intersection::new(6.0, &sphere),
        ]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn ray_intersect_sphere_at_one_point() {
        let ray = Ray::new(Point::new(0.0, 1.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![
            Intersection::new(5.0, &sphere),
            Intersection::new(5.0, &sphere),
        ]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn ray_originated_inside_intersects_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![
            Intersection::new(-1.0, &sphere),
            Intersection::new(1.0, &sphere),
        ]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn ray_behind_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![
            Intersection::new(-6.0, &sphere),
            Intersection::new(-4.0, &sphere),
        ]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn sphere_transformation_matrix() {
        let sphere = Sphere::default();
        let expected_sphere_matrix = Matrix4::identity();
        assert_eq!(expected_sphere_matrix, sphere.transformation);
    }

    #[test]
    fn sphere_translation_matrix() {
        let sphere = Sphere::default().translate(2.0, 3.0, 4.0).transform();
        let expected_sphere_transformation = Transform::Translate(2.0, 3.0, 4.0).transformation();
        assert_eq!(expected_sphere_transformation, sphere.transformation);
    }

    #[test]
    fn ray_intersect_scaled_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default().scale(2.0, 2.0, 2.0).transform();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![
            Intersection::new(3.0, &sphere),
            Intersection::new(7.0, &sphere),
        ]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn ray_intersect_translated_sphere() {
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::default().translate(5.0, 0.0, 0.0).transform();
        let intersects = sphere.intersect(&ray);
        let expected_intersects = Intersections::new().with(vec![]);

        assert_eq!(expected_intersects, intersects);
    }

    #[test]
    fn sphere_normal_on_x() {
        let sphere = Sphere::default();
        let normal_vector = sphere.normal(Point::new(1.0, 0.0, 0.0));
        let expected_vector = Vector3::new(1.0, 0.0, 0.0);

        assert_eq!(expected_vector, normal_vector);
    }

    #[test]
    fn sphere_normal_on_y() {
        let sphere = Sphere::default();
        let normal_vector = sphere.normal(Point::new(0.0, 1.0, 0.0));
        let expected_vector = Vector3::new(0.0, 1.0, 0.0);

        assert_eq!(expected_vector, normal_vector);
    }

    #[test]
    fn sphere_normal_on_z() {
        let sphere = Sphere::default();
        let normal_vector = sphere.normal(Point::new(0.0, 0.0, 1.0));
        let expected_vector = Vector3::new(0.0, 0.0, 1.0);

        assert_eq!(expected_vector, normal_vector);
    }

    #[test]
    fn sphere_normal_nonaxial() {
        let sphere = Sphere::default();
        let normal_vector = sphere.normal(Point::new(
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
        ));
        let expected_vector = Vector3::new(
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
        );

        assert_eq!(expected_vector, normal_vector);
    }

    #[test]
    fn sphere_normal_is_normalized() {
        let sphere = Sphere::default();
        let normal_vector = sphere.normal(Point::new(
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
        ));
        let expected_vector = Vector3::new(
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
            f64::sqrt(3.0) / 3.0,
        );

        assert_eq!(expected_vector, normal_vector.normalize());
    }

    #[test]
    fn sphere_translated_normal() {
        let sphere = Sphere::default().translate(0.0, 1.0, 0.0).transform();
        let normal_vector = sphere.normal(Point::new(0.0, 1.70711, -0.70711));
        let expected_vector = Vector3::new(0.0, 0.70711, -0.70711);

        assert_eq!(expected_vector, normal_vector);
    }

    #[test]
    fn sphere_transformed_normal() {
        let sphere = Sphere::default()
            .rotate(Over::Z, PI / 5.0)
            .scale(1.0, 0.5, 1.0)
            .transform();
        let normal_vector =
            sphere.normal(Point::new(0.0, f64::sqrt(2.0) / 2.0, -f64::sqrt(2.0) / 2.0));
        let expected_vector = Vector3::new(0.0, 0.97014, -0.24254);

        assert_eq!(expected_vector, normal_vector);
    }
}
