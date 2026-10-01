use crate::color::Color;
use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub enum TextureKind {
    CatFur,
    Checker { scale: f64 },
    Glow,
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
    pub refractive_index: f64,
    pub emission: f64,
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
        refractive_index: f64,
        emission: f64,
    ) -> Self {
        Self {
            name,
            texture,
            albedo,
            accent,
            specular,
            transparency,
            reflectivity,
            refractive_index,
            emission,
        }
    }

    pub fn sample(self, u: f64, v: f64, point: Vec3) -> Color {
        match self.texture {
            TextureKind::CatFur => {
                let stripe = ((point.x * 12.0).sin() + (point.z * 9.0).cos()).abs();
                if stripe > 1.35 {
                    self.accent
                } else {
                    self.albedo * (0.72 + stripe * 0.12)
                }
            }
            TextureKind::Checker { scale } => {
                if checker(u, v, scale) {
                    self.albedo
                } else {
                    self.accent
                }
            }
            TextureKind::Glow => {
                let shimmer = ((u * 8.0).sin() * (v * 8.0).cos()).abs();
                self.albedo * (0.82 + shimmer * 0.18) + self.accent * (shimmer * 0.25)
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
