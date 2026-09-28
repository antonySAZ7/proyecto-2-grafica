use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub const fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub const fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub const fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn unit(self) -> Self {
        self / self.length()
    }

    pub fn reflect(self, normal: Self) -> Self {
        self - normal * (2.0 * self.dot(normal))
    }

    pub fn refract(self, normal: Self, eta_ratio: f64) -> Option<Self> {
        let cos_theta = (-self).dot(normal).min(1.0);
        let sin2_theta = 1.0 - cos_theta * cos_theta;

        if eta_ratio * eta_ratio * sin2_theta > 1.0 {
            return None;
        }

        let perpendicular = eta_ratio * (self + normal * cos_theta);
        let parallel = -normal * (1.0 - perpendicular.length_squared()).abs().sqrt();

        Some(perpendicular + parallel)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;

    #[test]
    fn normalizes_vector() {
        let unit = Vec3::new(3.0, 4.0, 0.0).unit();

        assert!((unit.length() - 1.0).abs() < 1e-10);
        assert!((unit.x - 0.6).abs() < 1e-10);
        assert!((unit.y - 0.8).abs() < 1e-10);
    }

    #[test]
    fn calculates_dot_product() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, -5.0, 6.0);

        assert_eq!(a.dot(b), 12.0);
    }

    #[test]
    fn calculates_cross_product() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);

        assert_eq!(x.cross(y), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn reflects_vector() {
        let incoming = Vec3::new(1.0, -1.0, 0.0).unit();
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let reflected = incoming.reflect(normal);

        assert!((reflected.x - incoming.x).abs() < 1e-10);
        assert!((reflected.y + incoming.y).abs() < 1e-10);
    }

    #[test]
    fn refracts_vector() {
        let incoming = Vec3::new(0.0, -1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let refracted = incoming
            .refract(normal, 1.0 / 1.33)
            .expect("normal incidence should refract");

        assert!((refracted.x - incoming.x).abs() < 1e-10);
        assert!((refracted.y - incoming.y).abs() < 1e-10);
    }
}
