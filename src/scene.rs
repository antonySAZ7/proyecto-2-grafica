use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;

const EPSILON: f64 = 1e-6;

#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    pub t: f64,
    pub point: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub u: f64,
    pub v: f64,
    pub material_index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material_index: usize,
}

impl Cube {
    pub const fn new(min: Vec3, max: Vec3, material_index: usize) -> Self {
        Self {
            min,
            max,
            material_index,
        }
    }

    pub fn hit(self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut best: Option<HitRecord> = None;
        let faces = [
            (0, self.min.x, Vec3::new(-1.0, 0.0, 0.0)),
            (0, self.max.x, Vec3::new(1.0, 0.0, 0.0)),
            (1, self.min.y, Vec3::new(0.0, -1.0, 0.0)),
            (1, self.max.y, Vec3::new(0.0, 1.0, 0.0)),
            (2, self.min.z, Vec3::new(0.0, 0.0, -1.0)),
            (2, self.max.z, Vec3::new(0.0, 0.0, 1.0)),
        ];

        for (axis, plane, outward_normal) in faces {
            let direction = axis_value(ray.direction, axis);
            if direction.abs() < EPSILON {
                continue;
            }

            let t = (plane - axis_value(ray.origin, axis)) / direction;
            if t < t_min || t > t_max || best.is_some_and(|hit| t >= hit.t) {
                continue;
            }

            let point = ray.at(t);
            if self.contains_on_face(point, axis) {
                let (u, v) = self.face_uv(point, axis);
                let front_face = ray.direction.dot(outward_normal) < 0.0;
                let normal = if front_face {
                    outward_normal
                } else {
                    -outward_normal
                };
                best = Some(HitRecord {
                    t,
                    point,
                    normal,
                    front_face,
                    u,
                    v,
                    material_index: self.material_index,
                });
            }
        }

        best
    }

    fn contains_on_face(self, point: Vec3, axis: usize) -> bool {
        for other_axis in 0..3 {
            if other_axis == axis {
                continue;
            }

            let value = axis_value(point, other_axis);
            let min = axis_value(self.min, other_axis) - EPSILON;
            let max = axis_value(self.max, other_axis) + EPSILON;

            if value < min || value > max {
                return false;
            }
        }

        true
    }

    fn face_uv(self, point: Vec3, axis: usize) -> (f64, f64) {
        match axis {
            0 => (
                remap(point.z, self.min.z, self.max.z),
                remap(point.y, self.min.y, self.max.y),
            ),
            1 => (
                remap(point.x, self.min.x, self.max.x),
                remap(point.z, self.min.z, self.max.z),
            ),
            _ => (
                remap(point.x, self.min.x, self.max.x),
                remap(point.y, self.min.y, self.max.y),
            ),
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    pub materials: Vec<Material>,
    pub cubes: Vec<Cube>,
}

impl Scene {
    pub fn new(materials: Vec<Material>, cubes: Vec<Cube>) -> Self {
        Self { materials, cubes }
    }

    pub fn hit(&self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut best = None;

        for cube in &self.cubes {
            if let Some(hit) = cube.hit(ray, t_min, closest) {
                closest = hit.t;
                best = Some(hit);
            }
        }

        best
    }
}

fn axis_value(vector: Vec3, axis: usize) -> f64 {
    match axis {
        0 => vector.x,
        1 => vector.y,
        _ => vector.z,
    }
}

fn remap(value: f64, min: f64, max: f64) -> f64 {
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::Cube;
    use crate::ray::Ray;
    use crate::vec3::Vec3;

    #[test]
    fn ray_hits_front_face_of_cube() {
        let cube = Cube::new(Vec3::new(-0.5, -0.5, -2.0), Vec3::new(0.5, 0.5, -1.0), 0);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.hit(ray, 0.001, f64::INFINITY).expect("expected hit");

        assert!((hit.t - 1.0).abs() < 1e-10);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }
}
