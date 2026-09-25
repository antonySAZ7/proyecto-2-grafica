mod camera;
mod color;
mod material;
mod ray;
mod scene;
mod vec3;

use std::fs::{File, create_dir_all};
use std::io::{self, BufWriter, Write};

use camera::Camera;
use color::{Color, ppm_pixel};
use material::{Material, TextureKind};
use ray::Ray;
use scene::{Cube, Scene};
use vec3::Vec3;

fn main() {
    println!("Proyecto 2: Diorama con Raytracing");

    match render_ppm("renders/fase2_cubos.ppm") {
        Ok(()) => println!("Fase 2 lista: renders/fase2_cubos.ppm"),
        Err(error) => eprintln!("No se pudo renderizar la imagen: {error}"),
    }
}

fn render_ppm(path: &str) -> io::Result<()> {
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as usize;
    let camera = Camera::new(aspect_ratio);
    let scene = build_scene();
    let material_names = scene
        .materials
        .iter()
        .map(|material| material.name)
        .collect::<Vec<_>>()
        .join(", ");

    println!("Materiales cargados: {material_names}");

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
            writer.write_all(ppm_pixel(ray_color(ray, &scene)).as_bytes())?;
        }
    }

    writer.flush()
}

fn build_scene() -> Scene {
    let materials = vec![
        Material::new(
            "pasto",
            TextureKind::Grass,
            Vec3::new(0.18, 0.58, 0.18),
            Vec3::new(0.11, 0.38, 0.12),
            0.15,
            0.0,
            0.05,
        ),
        Material::new(
            "piedra",
            TextureKind::Stone,
            Vec3::new(0.48, 0.49, 0.48),
            Vec3::new(0.22, 0.23, 0.24),
            0.25,
            0.0,
            0.08,
        ),
        Material::new(
            "madera",
            TextureKind::Wood,
            Vec3::new(0.55, 0.31, 0.13),
            Vec3::new(0.32, 0.16, 0.06),
            0.18,
            0.0,
            0.04,
        ),
        Material::new(
            "ajedrez",
            TextureKind::Checker { scale: 6.0 },
            Vec3::new(0.82, 0.74, 0.58),
            Vec3::new(0.36, 0.29, 0.22),
            0.2,
            0.0,
            0.05,
        ),
    ];

    let cubes = vec![
        Cube::new(Vec3::new(-3.0, -1.35, -5.8), Vec3::new(3.0, -1.15, -1.6), 0),
        Cube::new(
            Vec3::new(-0.9, -1.15, -3.6),
            Vec3::new(-0.1, -0.35, -2.8),
            2,
        ),
        Cube::new(
            Vec3::new(0.15, -1.15, -3.65),
            Vec3::new(0.95, -0.35, -2.85),
            1,
        ),
        Cube::new(
            Vec3::new(1.05, -1.15, -4.35),
            Vec3::new(1.75, -0.45, -3.65),
            3,
        ),
        Cube::new(
            Vec3::new(-1.6, -1.15, -4.45),
            Vec3::new(-0.9, -0.45, -3.75),
            1,
        ),
    ];

    Scene::new(materials, cubes)
}

fn ray_color(ray: Ray, scene: &Scene) -> Color {
    if let Some(hit) = scene.hit(ray, 0.001, f64::INFINITY) {
        let material = scene.materials[hit.material_index];
        let texture_color = material.sample(hit.u, hit.v, hit.point);
        let light_direction = Vec3::new(-0.55, 0.8, 0.35).unit();
        let diffuse = hit.normal.dot(light_direction).max(0.0);
        let ambient = 0.22;
        let light = ambient + diffuse * (0.78 + material.specular * 0.12);
        let finish = 1.0 - material.transparency * 0.25 + material.reflectivity * 0.1;

        return texture_color * (light * finish);
    }

    let unit_direction = (ray.at(1.0) - ray.origin).unit();
    let blend = 0.5 * (unit_direction.y + 1.0);

    (1.0 - blend) * Vec3::new(1.0, 1.0, 1.0) + blend * Vec3::new(0.45, 0.68, 1.0)
}
