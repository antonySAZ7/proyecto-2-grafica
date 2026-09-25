use crate::color::Color;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub enum TextureKind {
    Checker { scale: f64 },
    Grass,
    Stone,
    Water,
    Wood,
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub name: &'static str,
    pub texture: TextureKind,
    pub albedo: Color,
    pub accent: Color,
    pub specular: f64,
    pub transparency: f64,
    pub reflectivity: f64,
}

impl Material {
    pub const fn new(
        name: &'static str,
        texture: TextureKind,
        albedo: Color,
        accent: Color,
        specular: f64,
        transparency: f64,
        reflectivity: f64,
    ) -> Self {
        Self {
            name,
            texture,
            albedo,
            accent,
            specular,
            transparency,
            reflectivity,
        }
    }

    pub fn sample(self, u: f64, v: f64, point: Vec3) -> Color {
        match self.texture {
            TextureKind::Checker { scale } => {
                if checker(u, v, scale) {
                    self.albedo
                } else {
                    self.accent
                }
            }
            TextureKind::Grass => {
                let stripe = ((u * 18.0).floor() as i32 + (v * 10.0).floor() as i32) & 1 == 0;
                if stripe { self.albedo } else { self.accent }
            }
            TextureKind::Stone => {
                let crack =
                    ((point.x * 9.0).sin() + (point.y * 13.0).cos() + (point.z * 7.0).sin()).abs();
                if crack > 1.55 {
                    self.accent
                } else {
                    self.albedo * (0.78 + 0.18 * crack)
                }
            }
            TextureKind::Water => {
                let ripple = ((point.x * 8.0).sin() * (point.z * 10.0).cos()).abs();
                self.albedo * (0.75 + ripple * 0.2) + self.accent * (ripple * 0.25)
            }
            TextureKind::Wood => {
                let rings = ((u * 16.0 + (v * 2.0).sin() * 0.6).floor() as i32) & 1 == 0;
                if rings { self.albedo } else { self.accent }
            }
        }
    }
}

fn checker(u: f64, v: f64, scale: f64) -> bool {
    ((u * scale).floor() as i32 + (v * scale).floor() as i32) & 1 == 0
}
