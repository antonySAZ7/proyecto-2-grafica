mod camera;
mod color;
mod material;
mod ray;
mod scene;
mod vec3;

use std::fs::{File, create_dir_all};
use std::io::{self, BufWriter, Write};
use std::{env, process};

use camera::Camera;
use color::{Color, minifb_pixel, ppm_pixel};
use material::{Material, TextureKind};
use minifb::{Key, Window, WindowOptions};
use ray::Ray;
use scene::{Cube, Scene};
use vec3::Vec3;

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 520;

fn main() {
    println!("Proyecto 2: Diorama con Raytracing");

    let show_preview = parse_show_preview();

    match render("renders/fase3_diorama.ppm", show_preview) {
        Ok(()) if show_preview => {
            println!("Ventana cerrada. Render guardado en renders/fase3_diorama.ppm");
        }
        Ok(()) => println!("Render guardado en renders/fase3_diorama.ppm"),
        Err(error) => eprintln!("No se pudo renderizar la imagen: {error}"),
    }
}

fn parse_show_preview() -> bool {
    let mut show_preview = true;

    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--no-window" => show_preview = false,
            "--help" | "-h" => {
                println!("Uso: cargo run -- [--no-window]");
                println!("  --no-window   Renderiza solo el PPM, sin abrir la ventana.");
                process::exit(0);
            }
            unknown => {
                eprintln!("Argumento desconocido: {unknown}");
                eprintln!("Uso: cargo run -- [--no-window]");
                process::exit(2);
            }
        }
    }

    show_preview
}

fn render(path: &str, show_preview: bool) -> io::Result<()> {
    let image_height = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
    let camera = Camera::look_at(
        Vec3::new(4.2, 3.1, 3.8),
        Vec3::new(0.0, -0.55, -3.6),
        Vec3::new(0.0, 1.0, 0.0),
        46.0,
        ASPECT_RATIO,
    );
    let scene = build_scene();
    let material_names = scene
        .materials
        .iter()
        .map(|material| material.name)
        .collect::<Vec<_>>()
        .join(", ");

    println!("Materiales cargados: {material_names}");
    println!("Renderizando {IMAGE_WIDTH}x{image_height}...");

    let pixels = render_pixels(IMAGE_WIDTH, image_height, camera, &scene);
    save_ppm(path, IMAGE_WIDTH, image_height, &pixels)?;
    if show_preview {
        show_window(IMAGE_WIDTH, image_height, &pixels)?;
    }

    Ok(())
}

fn render_pixels(width: usize, height: usize, camera: Camera, scene: &Scene) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let sample_y = height - 1 - y;
        for x in 0..width {
            let u = x as f64 / (width - 1) as f64;
            let v = sample_y as f64 / (height - 1) as f64;
            let ray = camera.ray_at(u, v);
            pixels.push(ray_color(ray, scene));
        }
    }

    pixels
}

fn save_ppm(path: &str, width: usize, height: usize, pixels: &[Color]) -> io::Result<()> {
    create_dir_all("renders")?;
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "P3")?;
    writeln!(writer, "{width} {height}")?;
    writeln!(writer, "255")?;

    for pixel in pixels {
        writer.write_all(ppm_pixel(*pixel).as_bytes())?;
    }

    writer.flush()
}

fn show_window(width: usize, height: usize, pixels: &[Color]) -> io::Result<()> {
    let buffer = pixels
        .iter()
        .map(|pixel| minifb_pixel(*pixel))
        .collect::<Vec<_>>();
    let mut window = Window::new(
        "Proyecto 2 - Diorama con Raytracing | Esc para cerrar",
        width,
        height,
        WindowOptions::default(),
    )
    .map_err(|error| io::Error::other(error.to_string()))?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width, height)
            .map_err(|error| io::Error::other(error.to_string()))?;
    }

    Ok(())
}

fn build_scene() -> Scene {
    const GRASS: usize = 0;
    const STONE: usize = 1;
    const WOOD: usize = 2;
    const PATH: usize = 3;
    const WATER: usize = 4;

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
            "arcilla",
            TextureKind::Checker { scale: 6.0 },
            Vec3::new(0.82, 0.74, 0.58),
            Vec3::new(0.36, 0.29, 0.22),
            0.2,
            0.0,
            0.05,
        ),
        Material::new(
            "agua",
            TextureKind::Water,
            Vec3::new(0.08, 0.38, 0.62),
            Vec3::new(0.52, 0.84, 0.95),
            0.45,
            0.35,
            0.2,
        ),
    ];

    let mut cubes = vec![
        Cube::new(
            Vec3::new(-3.2, -1.45, -6.4),
            Vec3::new(3.2, -1.15, -1.1),
            GRASS,
        ),
        Cube::new(
            Vec3::new(-2.9, -1.62, -5.95),
            Vec3::new(2.9, -1.45, -1.55),
            STONE,
        ),
        Cube::new(
            Vec3::new(-0.55, -1.14, -6.1),
            Vec3::new(0.55, -1.02, -1.35),
            WATER,
        ),
        Cube::new(
            Vec3::new(-0.72, -0.98, -4.0),
            Vec3::new(0.72, -0.82, -3.62),
            WOOD,
        ),
        Cube::new(
            Vec3::new(-0.18, -0.98, -4.2),
            Vec3::new(0.18, -0.82, -3.42),
            WOOD,
        ),
        Cube::new(
            Vec3::new(-2.7, -1.09, -2.9),
            Vec3::new(-0.95, -0.96, -2.45),
            PATH,
        ),
        Cube::new(
            Vec3::new(-1.32, -1.09, -3.85),
            Vec3::new(-0.88, -0.96, -2.5),
            PATH,
        ),
    ];

    add_house(&mut cubes, STONE, WOOD, PATH);
    add_tree(&mut cubes, WOOD, GRASS);
    add_rocks(&mut cubes, STONE);

    Scene::new(materials, cubes)
}

fn add_house(cubes: &mut Vec<Cube>, stone: usize, wood: usize, roof: usize) {
    cubes.extend([
        Cube::new(
            Vec3::new(1.05, -1.15, -4.95),
            Vec3::new(2.25, -0.2, -3.75),
            wood,
        ),
        Cube::new(
            Vec3::new(1.18, -1.08, -3.73),
            Vec3::new(2.12, -0.28, -3.63),
            stone,
        ),
        Cube::new(
            Vec3::new(1.35, -1.15, -3.61),
            Vec3::new(1.72, -0.55, -3.5),
            roof,
        ),
        Cube::new(
            Vec3::new(0.92, -0.24, -5.07),
            Vec3::new(2.38, 0.0, -3.63),
            roof,
        ),
        Cube::new(
            Vec3::new(1.12, 0.0, -4.88),
            Vec3::new(2.18, 0.2, -3.82),
            roof,
        ),
    ]);
}

fn add_tree(cubes: &mut Vec<Cube>, wood: usize, leaves: usize) {
    cubes.extend([
        Cube::new(
            Vec3::new(-2.35, -1.15, -4.85),
            Vec3::new(-2.02, -0.2, -4.52),
            wood,
        ),
        Cube::new(
            Vec3::new(-2.75, -0.42, -5.24),
            Vec3::new(-1.62, 0.22, -4.12),
            leaves,
        ),
        Cube::new(
            Vec3::new(-2.55, 0.12, -5.03),
            Vec3::new(-1.82, 0.58, -4.32),
            leaves,
        ),
    ]);
}

fn add_rocks(cubes: &mut Vec<Cube>, stone: usize) {
    cubes.extend([
        Cube::new(
            Vec3::new(-2.65, -1.15, -5.78),
            Vec3::new(-2.05, -0.72, -5.2),
            stone,
        ),
        Cube::new(
            Vec3::new(2.35, -1.15, -2.35),
            Vec3::new(2.82, -0.78, -1.9),
            stone,
        ),
        Cube::new(
            Vec3::new(0.9, -1.15, -5.78),
            Vec3::new(1.38, -0.84, -5.32),
            stone,
        ),
    ]);
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
