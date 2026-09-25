use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn ppm_pixel(color: Color) -> String {
    let r = (255.999 * color.x.clamp(0.0, 0.999)) as u32;
    let g = (255.999 * color.y.clamp(0.0, 0.999)) as u32;
    let b = (255.999 * color.z.clamp(0.0, 0.999)) as u32;

    format!("{r} {g} {b}\n")
}
