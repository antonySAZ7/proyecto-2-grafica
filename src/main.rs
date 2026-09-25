mod camera;
mod color;
mod ray;
mod vec3;

use std::fs::{File, create_dir_all};
use std::io::{self, BufWriter, Write};

use camera::Camera;
use color::{Color, ppm_pixel};
use ray::Ray;
use vec3::Vec3;

fn main() {
    println!("Proyecto 2: Diorama con Raytracing");

    match render_ppm("renders/fase1_cielo.ppm") {
        Ok(()) => println!("Fase 1 lista: renders/fase1_cielo.ppm"),
        Err(error) => eprintln!("No se pudo renderizar la imagen: {error}"),
    }
}

fn render_ppm(path: &str) -> io::Result<()> {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as usize;
    let camera = Camera::new(aspect_ratio);

    create_dir_all("renders")?;
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "P3")?;
    writeln!(writer, "{image_width} {image_height}")?;
    writeln!(writer, "255")?;

    for y in (0..image_height).rev() {
        for x in 0..image_width {
            let u = x as f64 / (image_width - 1) as f64;
            let v = y as f64 / (image_height - 1) as f64;
            let ray = camera.ray_at(u, v);
            writer.write_all(ppm_pixel(ray_color(ray)).as_bytes())?;
        }
    }

    writer.flush()
}

fn ray_color(ray: Ray) -> Color {
    let unit_direction = (ray.at(1.0) - ray.origin).unit();
    let blend = 0.5 * (unit_direction.y + 1.0);

    (1.0 - blend) * Vec3::new(1.0, 1.0, 1.0) + blend * Vec3::new(0.45, 0.68, 1.0)
}
