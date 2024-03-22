use crate::color::Color;
use crate::float_eq::ApproxEq;
use crate::intersections::{IntersectionDetails, Intersections};
use crate::lights::PointLight;
use crate::point::Point;
use crate::primitives::{Primitive, PrimitiveShape};
use crate::ray::Ray;
use crate::vector::Vector3;

pub struct World {
    objects: Vec<PrimitiveShape>,
    light_sources: Vec<PointLight>,
    recursive_depth: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(8),
            light_sources: Vec::with_capacity(4),
            recursive_depth: 4,
        }
    }

    pub fn add_object(mut self, object: PrimitiveShape) -> Self {
        self.objects.push(object);
        self
    }

    pub fn add_light_source(mut self, source: PointLight) -> Self {
        self.light_sources.push(source);
        self
    }

    pub fn with_objects(mut self, objects: Vec<PrimitiveShape>) -> Self {
        self.objects = objects;
        self
    }

    pub fn with_light_sources(mut self, sources: Vec<PointLight>) -> Self {
        self.light_sources = sources;
        self
    }

    pub fn intersect_objects(&self, ray: &Ray) -> Intersections {
        let mut intersections = Intersections::new();

        self.objects
            .iter()
            .for_each(|object| intersections.merge(object.intersect(ray)));

        // TODO: move sort to intersections
        intersections.sort()
    }

    pub fn shade_hit(&self, details: &IntersectionDetails, recursive_depth: usize) -> Color {
        self.light_sources
            .iter()
            .fold(Color::default(), |acc, light| {
                let is_shadowed = self.shadow_cast(details.over_point);

                let color = details.object.material().phong_reflection(
                    *light,
                    details.object.clone(),
                    details.over_point,
                    details.eye_vector,
                    details.normal_vector,
                    is_shadowed,
                );

                let reflected_color = self.reflect_color(details, recursive_depth);

                acc + color + reflected_color
            })
    }

    pub fn reflect_color(&self, details: &IntersectionDetails, recursive_depth: usize) -> Color {
        if details.object.material().reflective.approx_eq_low(&0.0) || recursive_depth == 0 {
            Color::black()
        } else {
            let reflected_ray = Ray::new(details.over_point, details.reflection_vector);

            let color = self.color_at_recursive(&reflected_ray, recursive_depth - 1);

            color * details.object.material().reflective
        }
    }

    pub fn refracted_color(&self, details: &IntersectionDetails, recursive_depth: usize) -> Color {
        if details.object.material().transparency.approx_eq_low(&0.0) || recursive_depth == 0 {
            Color::black()
        } else {
            Color::white()
        }
    }

    pub fn color_at(&self, ray: &Ray) -> Color {
        self.color_at_recursive(ray, self.recursive_depth)
    }

    fn color_at_recursive(&self, ray: &Ray, recursive_depth: usize) -> Color {
        let intersections = self.intersect_objects(&ray);

        match intersections.hit() {
            Some(hit) => {
                let comps = IntersectionDetails::from(hit, &ray);
                self.shade_hit(&comps, recursive_depth)
            }
            None => Color::black(),
        }
    }

    pub fn shadow_cast(&self, point: Point) -> bool {
        for light in &self.light_sources {
            let v = light.position - point;
            let distance = v.magnitude();
            let direction = v.normalize();

            let ray = Ray::new(point, Vector3::new(direction.x, direction.y, direction.z));
            let intersections = self.intersect_objects(&ray);

            if let Some(hit) = intersections.hit() {
                if hit.time < distance {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod world_tests {
    use crate::color::Color;
    use crate::intersections::{Intersection, IntersectionDetails, Intersections};
    use crate::lights::PointLight;
    use crate::material::Material;
    use crate::point::Point;
    use crate::primitives::PrimitiveShape::{PlaneShape, SphereShape};
    use crate::primitives::{Plane, Primitive, Sphere};
    use crate::ray::Ray;
    use crate::transformations::Transformable;
    use crate::vector::Vector3;
    use crate::world::World;

    fn simulated_world() -> World {
        let sphere_a = Sphere::default().scale(0.5, 0.5, 0.5).transform();

        let sphere_b = Sphere::default().apply_material(
            Material::default()
                .color(Color::new(0.8, 1.0, 0.6))
                .specular(0.2)
                .diffuse(0.7),
        );

        let light_source =
            PointLight::new(Color::new(1.0, 1.0, 1.0), Point::new(-10.0, 10.0, -10.0));

        let world = World::default()
            .add_object(SphereShape(sphere_a.clone()))
            .add_object(SphereShape(sphere_b.clone()))
            .add_light_source(light_source);

        world
    }

    #[test]
    fn world_creation() {
        let world = World::default();

        assert_eq!(0, world.objects.len());
        assert_eq!(0, world.light_sources.len());
    }

    #[test]
    fn world_with_objects() {
        let world = World::default()
            .add_object(SphereShape(
                Sphere::default().scale(0.5, 0.5, 0.5).transform(),
            ))
            .add_object(SphereShape(
                Sphere::default().apply_material(
                    Material::default()
                        .color(Color::new(0.8, 1.0, 0.6))
                        .specular(0.2)
                        .diffuse(0.7),
                ),
            ))
            .add_light_source(PointLight::new(
                Color::new(1.0, 1.0, 1.0),
                Point::new(-10.0, 10.0, -10.0),
            ));

        let expected_sphere = SphereShape(Sphere::default().scale(0.5, 0.5, 0.5).transform());

        assert_eq!(expected_sphere, world.objects[0]);

        assert_eq!(2, world.objects.len());
        assert_eq!(1, world.light_sources.len());
    }

    #[test]
    fn world_with_objects_intersections() {
        let sphere_a = Sphere::default().scale(0.5, 0.5, 0.5).transform();

        let sphere_b = Sphere::default().apply_material(
            Material::default()
                .color(Color::new(0.8, 1.0, 0.6))
                .specular(0.2)
                .diffuse(0.7),
        );

        let world = simulated_world();

        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));

        let expected_intersections = Intersections::new().with(vec![
            Intersection::new(4.0, SphereShape(sphere_b.clone())),
            Intersection::new(4.5, SphereShape(sphere_a.clone())),
            Intersection::new(5.5, SphereShape(sphere_a.clone())),
            Intersection::new(6.0, SphereShape(sphere_b.clone())),
        ]);

        assert_eq!(expected_intersections, world.intersect_objects(&ray));
    }

    #[test]
    fn world_shade_intersection() {
        let world = simulated_world();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));

        let intersection = Intersection::new(4.0, world.objects[1].clone());
        let intersection_details = IntersectionDetails::from(&intersection, &ray);

        let expected_color = Color::new(0.38066, 0.47583, 0.2855);

        assert_eq!(expected_color, world.shade_hit(&intersection_details, 0));
    }

    #[test]
    fn world_shade_intersection_from_inside() {
        let world = simulated_world().with_light_sources(vec![PointLight::new(
            Color::new(1.0, 1.0, 1.0),
            Point::new(0.0, 0.25, 0.0),
        )]);
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
        let sphere_a = Sphere::default().scale(0.5, 0.5, 0.5).transform();

        let intersection = Intersection::new(0.5, SphereShape(sphere_a));
        let intersection_details = IntersectionDetails::from(&intersection, &ray);

        let expected_color = Color::new(0.90498, 0.90498, 0.90498);

        assert_eq!(expected_color, world.shade_hit(&intersection_details, 0));
    }

    #[test]
    fn world_color_at_ray_miss() {
        let world = simulated_world();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 1.0, 0.0));
        let expected_color = Color::new(0.0, 0.0, 0.0);

        assert_eq!(expected_color, world.color_at_recursive(&ray, 0));
    }

    #[test]
    fn world_color_at_ray_hits() {
        let world = simulated_world();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, 1.0));
        let expected_color = Color::new(0.38066, 0.47583, 0.2855);

        assert_eq!(expected_color, world.color_at_recursive(&ray, 0));
    }

    #[test]
    fn world_color_behind_the_ray() {
        let outer = SphereShape(
            Sphere::default()
                .apply_material(Material::default().diffuse(0.7).specular(0.2).ambient(1.0)),
        );

        let inner = SphereShape(Sphere::default().apply_material(Material::default().ambient(1.0)));

        let world = simulated_world().with_objects(vec![outer, inner]);

        let ray = Ray::new(Point::new(0.0, 0.0, 0.75), Vector3::new(0.0, 0.0, -1.0));
        let expected_color = Color::new(1.0, 1.0, 1.0);

        assert_eq!(expected_color, world.color_at_recursive(&ray, 0));
    }

    #[test]
    fn world_there_are_no_shadow() {
        let world = simulated_world();
        let point = Point::new(0.0, 10.0, 0.0);

        let expected_shadow = false;

        assert_eq!(expected_shadow, world.shadow_cast(point));
    }

    #[test]
    fn world_there_are_shadow_between_sphere_and_light() {
        let world = simulated_world();
        let point = Point::new(10.0, -10.0, 10.0);

        let expected_shadow = true;

        assert_eq!(expected_shadow, world.shadow_cast(point));
    }

    #[test]
    fn world_there_are_no_shadow_point_behind_light() {
        let world = simulated_world();
        let point = Point::new(-20.0, 20.0, -20.0);

        let expected_shadow = false;

        assert_eq!(expected_shadow, world.shadow_cast(point));
    }

    #[test]
    fn world_there_are_no_shadow_between_sphere_and_light() {
        let world = simulated_world();
        let point = Point::new(-2.0, 2.0, -2.0);

        let expected_shadow = false;

        assert_eq!(expected_shadow, world.shadow_cast(point));
    }

    #[test]
    fn world_intersection_in_shadow() {
        let sphere_a = SphereShape(Sphere::default());
        let sphere_b = SphereShape(Sphere::default().translate(0.0, 0.0, 10.0).transform());
        let light_point = PointLight::new(Color::new(1.0, 1.0, 1.0), Point::new(0.0, 0.0, -10.0));

        let world = World::default()
            .with_objects(vec![sphere_a.clone(), sphere_b.clone()])
            .with_light_sources(vec![light_point]);

        let ray = Ray::new(Point::new(0.0, 0.0, 5.0), Vector3::new(0.0, 0.0, 1.0));
        let intersection = Intersection::new(4.0, sphere_b.clone());
        let intersection_details = IntersectionDetails::from(&intersection, &ray);
        let expected_color = Color::new(0.1, 0.1, 0.1);

        assert_eq!(expected_color, world.shade_hit(&intersection_details, 0));
    }

    #[test]
    fn world_reflected_color_for_nonreflective() {
        let world = simulated_world();
        let ray = Ray::new(Point::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0));

        let sphere = Sphere::default()
            .scale(0.5, 0.5, 0.5)
            .transform()
            .apply_material(Material::default().ambient(1.0));

        let intersection = Intersection::new(1.0, SphereShape(sphere));
        let intersection_details = IntersectionDetails::from(&intersection, &ray);

        let expected_color = Color::black();

        assert_eq!(
            expected_color,
            world.reflect_color(&intersection_details, 0)
        );
    }

    #[test]
    fn world_reflected_color_for_reflective() {
        let world = simulated_world();
        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector3::new(0.0, -f64::sqrt(2.0) / 2.0, f64::sqrt(2.0) / 2.0),
        );

        let plane = Plane::default()
            .translate(0.0, -1.0, 0.0)
            .transform()
            .apply_material(Material::default().reflective(0.5));

        let intersection = Intersection::new(f64::sqrt(2.0), PlaneShape(plane));
        let intersection_details = IntersectionDetails::from(&intersection, &ray);

        let expected_color = Color::new(0.19032, 0.2379, 0.14274);

        assert_eq!(
            expected_color,
            world.reflect_color(&intersection_details, 1)
        );
    }

    #[test]
    fn world_reflected_color_for_reflective_shade_hit() {
        let world = simulated_world();
        let ray = Ray::new(
            Point::new(0.0, 0.0, -3.0),
            Vector3::new(0.0, -f64::sqrt(2.0) / 2.0, f64::sqrt(2.0) / 2.0),
        );

        let plane = Plane::default()
            .translate(0.0, -1.0, 0.0)
            .transform()
            .apply_material(Material::default().reflective(0.5));

        let intersection = Intersection::new(f64::sqrt(2.0), PlaneShape(plane));
        let intersection_details = IntersectionDetails::from(&intersection, &ray);

        let expected_color = Color::new(0.87677, 0.92436, 0.82918);

        assert_eq!(expected_color, world.shade_hit(&intersection_details, 1));
    }

    #[test]
    fn world_refracted_color_with_opaque_surface() {
        let world = simulated_world();
        let ray = Ray::new(Point::new(0.0, 0.0, -5.0), Vector3::new(0.0, 0.0, -1.0));

        let sphere = SphereShape(
            Sphere::default()
                .scale(0.5, 0.5, 0.5)
                .transform()
                .apply_material(Material::default()),
        );

        let intersection_a = Intersection::new(4.0, sphere.clone());
        let intersection_b = Intersection::new(4.0, sphere.clone());
        let intersections = Intersections::new().with(vec![intersection_a.clone(), intersection_b]);

        let intersection_details =
            IntersectionDetails::from_many(&intersection_a, &intersections, &ray);
        let expected_color = Color::black();

        assert_eq!(
            expected_color,
            world.refracted_color(&intersection_details, 0)
        );
    }
}
