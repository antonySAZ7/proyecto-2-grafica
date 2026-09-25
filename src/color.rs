use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn ppm_pixel(color: Color) -> String {
    let (r, g, b) = rgb8(color);

    format!("{r} {g} {b}\n")
}

pub fn minifb_pixel(color: Color) -> u32 {
    let (r, g, b) = rgb8(color);

    u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
}

fn rgb8(color: Color) -> (u8, u8, u8) {
    let r = (255.999 * color.x.clamp(0.0, 0.999)) as u32;
    let g = (255.999 * color.y.clamp(0.0, 0.999)) as u32;
    let b = (255.999 * color.z.clamp(0.0, 0.999)) as u32;

    (r as u8, g as u8, b as u8)
}
