use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn look_at(
        origin: Vec3,
        target: Vec3,
        up: Vec3,
        vertical_fov_degrees: f64,
        aspect_ratio: f64,
    ) -> Self {
        let theta = vertical_fov_degrees.to_radians();
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = aspect_ratio * viewport_height;

        let forward = (target - origin).unit();
        let right = forward.cross(up).unit();
        let camera_up = right.cross(forward);
        let horizontal = right * viewport_width;
        let vertical = camera_up * viewport_height;
        let lower_left_corner = origin + forward - horizontal / 2.0 - vertical / 2.0;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn ray_at(self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        )
    }
}
