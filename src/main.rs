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
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use ray::Ray;
use scene::{Cube, Scene};
use vec3::Vec3;

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 460;
const MAX_RAY_DEPTH: usize = 3;
const RAY_BIAS: f64 = 1e-4;
const LIGHT_DIRECTION: Vec3 = Vec3::new(-0.35, 0.82, 0.28);
const CAMERA_TARGET: Vec3 = Vec3::new(0.0, -0.58, -3.7);
const CAMERA_HEIGHT: f64 = 3.1;
const INITIAL_CAMERA_ANGLE: f64 = 0.53;
const INITIAL_CAMERA_DISTANCE: f64 = 5.15;
const MIN_CAMERA_DISTANCE: f64 = 3.0;
const MAX_CAMERA_DISTANCE: f64 = 8.5;
const CAMERA_ROTATION_STEP: f64 = 0.18;
const CAMERA_ZOOM_STEP: f64 = 0.45;

#[derive(Clone, Copy, Debug)]
struct CameraOrbit {
    angle: f64,
    distance: f64,
}

impl CameraOrbit {
    const fn new(angle: f64, distance: f64) -> Self {
        Self { angle, distance }
    }

    fn reset(&mut self) {
        self.angle = INITIAL_CAMERA_ANGLE;
        self.distance = INITIAL_CAMERA_DISTANCE;
    }

    fn rotate(&mut self, delta: f64) {
        self.angle += delta;
    }

    fn zoom(&mut self, delta: f64) {
        self.distance = (self.distance + delta).clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    }

    fn camera(self) -> Camera {
        let origin = Vec3::new(
            CAMERA_TARGET.x + self.distance * self.angle.sin(),
            CAMERA_HEIGHT,
            CAMERA_TARGET.z + self.distance * self.angle.cos(),
        );

        Camera::look_at(
            origin,
            CAMERA_TARGET,
            Vec3::new(0.0, 1.0, 0.0),
            46.0,
            ASPECT_RATIO,
        )
    }
}

fn main() {
    println!("Proyecto 2: Diorama nocturno de gatos con Raytracing");

    let options = parse_options();

    let result = if let Some(frame_count) = options.frames {
        render_frames(frame_count)
    } else {
        render("renders/final_gatos_nocturnos.ppm", options.show_preview)
    };

    match result {
        Ok(()) if options.show_preview && options.frames.is_none() => {
            println!("Ventana cerrada. Render guardado en renders/final_gatos_nocturnos.ppm");
        }
        Ok(()) if options.frames.is_some() => println!("Frames guardados en frames/"),
        Ok(()) => println!("Render guardado en renders/final_gatos_nocturnos.ppm"),
        Err(error) => eprintln!("No se pudo renderizar la imagen: {error}"),
    }
}

#[derive(Clone, Copy, Debug)]
struct RunOptions {
    show_preview: bool,
    frames: Option<usize>,
}

fn parse_options() -> RunOptions {
    let mut show_preview = true;
    let mut frames = None;
    let mut arguments = env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--no-window" => show_preview = false,
            "--frames" => {
                let Some(value) = arguments.next() else {
                    eprintln!("Falta el numero de frames despues de --frames.");
                    process::exit(2);
                };
                let parsed = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Numero de frames invalido: {value}");
                    process::exit(2);
                });
                frames = Some(parsed.max(1));
                show_preview = false;
            }
            "--help" | "-h" => {
                println!("Uso: cargo run -- [--no-window] [--frames N]");
                println!("  --no-window   Renderiza solo el PPM, sin abrir la ventana.");
                println!("  --frames N    Renderiza N frames orbitando para armar un video.");
                process::exit(0);
            }
            unknown => {
                eprintln!("Argumento desconocido: {unknown}");
                eprintln!("Uso: cargo run -- [--no-window] [--frames N]");
                process::exit(2);
            }
        }
    }

    RunOptions {
        show_preview,
        frames,
    }
}

fn render(path: &str, show_preview: bool) -> io::Result<()> {
    let image_height = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
    let camera_orbit = CameraOrbit::new(INITIAL_CAMERA_ANGLE, INITIAL_CAMERA_DISTANCE);
    let scene = build_scene();
    let material_names = scene
        .materials
        .iter()
        .map(|material| material.name)
        .collect::<Vec<_>>()
        .join(", ");

    println!("Materiales cargados: {material_names}");
    println!("Renderizando {IMAGE_WIDTH}x{image_height}...");

    let pixels = render_pixels(IMAGE_WIDTH, image_height, camera_orbit.camera(), &scene);
    save_ppm(path, IMAGE_WIDTH, image_height, &pixels)?;
    if show_preview {
        show_window(IMAGE_WIDTH, image_height, &scene, pixels, camera_orbit)?;
    }

    Ok(())
}

fn render_frames(frame_count: usize) -> io::Result<()> {
    let width = 420;
    let height = (width as f64 / ASPECT_RATIO) as usize;
    let scene = build_scene();

    create_dir_all("frames")?;
    println!("Renderizando {frame_count} frames {width}x{height}...");

    for frame in 0..frame_count {
        let progress = frame as f64 / frame_count as f64;
        let angle = INITIAL_CAMERA_ANGLE + progress * std::f64::consts::TAU;
        let zoom_wave = (progress * std::f64::consts::TAU).sin();
        let distance = INITIAL_CAMERA_DISTANCE + zoom_wave * 0.9;
        let camera = CameraOrbit::new(angle, distance).camera();
        let pixels = render_pixels(width, height, camera, &scene);
        let path = format!("frames/frame_{frame:04}.ppm");
        save_ppm(&path, width, height, &pixels)?;
        println!("Frame {}/{}: {}", frame + 1, frame_count, path);
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
            pixels.push(ray_color(ray, scene, MAX_RAY_DEPTH));
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

fn show_window(
    width: usize,
    height: usize,
    scene: &Scene,
    mut pixels: Vec<Color>,
    mut camera_orbit: CameraOrbit,
) -> io::Result<()> {
    let mut buffer = to_window_buffer(&pixels);
    let mut window = Window::new(
        "Proyecto 2 - A/D gira | W/S zoom | R reinicia | Esc cierra",
        width,
        height,
        WindowOptions::default(),
    )
    .map_err(|error| io::Error::other(error.to_string()))?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width, height)
            .map_err(|error| io::Error::other(error.to_string()))?;

        if update_camera_from_input(&window, &mut camera_orbit) {
            println!(
                "Renderizando vista: angulo {:.2}, zoom {:.2}",
                camera_orbit.angle, camera_orbit.distance
            );
            pixels = render_pixels(width, height, camera_orbit.camera(), scene);
            buffer = to_window_buffer(&pixels);
        }
    }

    Ok(())
}

fn to_window_buffer(pixels: &[Color]) -> Vec<u32> {
    pixels.iter().map(|pixel| minifb_pixel(*pixel)).collect()
}

fn update_camera_from_input(window: &Window, camera_orbit: &mut CameraOrbit) -> bool {
    let mut changed = false;

    if window.is_key_pressed(Key::A, KeyRepeat::Yes)
        || window.is_key_pressed(Key::Left, KeyRepeat::Yes)
    {
        camera_orbit.rotate(-CAMERA_ROTATION_STEP);
        changed = true;
    }

    if window.is_key_pressed(Key::D, KeyRepeat::Yes)
        || window.is_key_pressed(Key::Right, KeyRepeat::Yes)
    {
        camera_orbit.rotate(CAMERA_ROTATION_STEP);
        changed = true;
    }

    if window.is_key_pressed(Key::W, KeyRepeat::Yes)
        || window.is_key_pressed(Key::Up, KeyRepeat::Yes)
    {
        camera_orbit.zoom(-CAMERA_ZOOM_STEP);
        changed = true;
    }

    if window.is_key_pressed(Key::S, KeyRepeat::Yes)
        || window.is_key_pressed(Key::Down, KeyRepeat::Yes)
    {
        camera_orbit.zoom(CAMERA_ZOOM_STEP);
        changed = true;
    }

    if window.is_key_pressed(Key::R, KeyRepeat::No) {
        camera_orbit.reset();
        changed = true;
    }

    changed
}

fn build_scene() -> Scene {
    const NIGHT_GRASS: usize = 0;
    const STONE: usize = 1;
    const LAVENDER_WOOD: usize = 2;
    const PINK_PATH: usize = 3;
    const WATER: usize = 4;
    const CAT_FUR: usize = 5;
    const CAT_ACCENT: usize = 6;
    const GLOW: usize = 7;
    const MOON: usize = 8;
    const BLANKET: usize = 9;

    let materials = vec![
        Material::new(
            "pasto nocturno",
            TextureKind::Grass,
            Vec3::new(0.07, 0.23, 0.24),
            Vec3::new(0.12, 0.36, 0.34),
            0.15,
            0.0,
            0.05,
            1.0,
            0.0,
        ),
        Material::new(
            "piedra azulada",
            TextureKind::Stone,
            Vec3::new(0.30, 0.35, 0.48),
            Vec3::new(0.13, 0.15, 0.24),
            0.25,
            0.0,
            0.08,
            1.0,
            0.0,
        ),
        Material::new(
            "madera lavanda",
            TextureKind::Wood,
            Vec3::new(0.45, 0.32, 0.62),
            Vec3::new(0.26, 0.18, 0.38),
            0.18,
            0.0,
            0.04,
            1.0,
            0.0,
        ),
        Material::new(
            "sendero rosado",
            TextureKind::Checker { scale: 6.0 },
            Vec3::new(0.82, 0.55, 0.74),
            Vec3::new(0.42, 0.30, 0.58),
            0.2,
            0.0,
            0.05,
            1.0,
            0.0,
        ),
        Material::new(
            "agua lunar",
            TextureKind::Water,
            Vec3::new(0.05, 0.18, 0.38),
            Vec3::new(0.47, 0.78, 0.98),
            0.45,
            0.55,
            0.34,
            1.33,
            0.0,
        ),
        Material::new(
            "pelaje violeta",
            TextureKind::CatFur,
            Vec3::new(0.20, 0.13, 0.34),
            Vec3::new(0.36, 0.24, 0.52),
            0.18,
            0.0,
            0.12,
            1.0,
            0.0,
        ),
        Material::new(
            "patitas crema",
            TextureKind::Checker { scale: 4.0 },
            Vec3::new(0.98, 0.78, 0.86),
            Vec3::new(0.80, 0.58, 0.75),
            0.26,
            0.0,
            0.06,
            1.0,
            0.0,
        ),
        Material::new(
            "ojos estrellas",
            TextureKind::Glow,
            Vec3::new(0.95, 0.95, 0.48),
            Vec3::new(0.65, 1.0, 0.95),
            0.85,
            0.0,
            0.25,
            1.0,
            1.25,
        ),
        Material::new(
            "luna",
            TextureKind::Glow,
            Vec3::new(0.96, 0.88, 0.58),
            Vec3::new(1.0, 0.96, 0.78),
            0.5,
            0.0,
            0.18,
            1.0,
            0.9,
        ),
        Material::new(
            "mantita",
            TextureKind::Checker { scale: 8.0 },
            Vec3::new(0.88, 0.42, 0.72),
            Vec3::new(0.30, 0.22, 0.52),
            0.22,
            0.0,
            0.05,
            1.0,
            0.0,
        ),
    ];

    let mut cubes = vec![
        Cube::new(
            Vec3::new(-3.2, -1.45, -6.4),
            Vec3::new(3.2, -1.15, -1.1),
            NIGHT_GRASS,
        ),
        Cube::new(
            Vec3::new(-2.9, -1.62, -5.95),
            Vec3::new(2.9, -1.45, -1.55),
            STONE,
        ),
        Cube::new(
            Vec3::new(-0.48, -1.14, -6.05),
            Vec3::new(0.62, -1.02, -1.6),
            WATER,
        ),
        Cube::new(
            Vec3::new(-0.88, -0.98, -4.05),
            Vec3::new(0.92, -0.82, -3.65),
            LAVENDER_WOOD,
        ),
        Cube::new(
            Vec3::new(-0.18, -0.98, -4.2),
            Vec3::new(0.18, -0.82, -3.42),
            LAVENDER_WOOD,
        ),
        Cube::new(
            Vec3::new(-2.7, -1.09, -2.9),
            Vec3::new(-0.95, -0.96, -2.45),
            PINK_PATH,
        ),
        Cube::new(
            Vec3::new(-1.32, -1.09, -3.85),
            Vec3::new(-0.88, -0.96, -2.5),
            PINK_PATH,
        ),
        Cube::new(
            Vec3::new(0.9, -1.08, -3.05),
            Vec3::new(2.15, -0.96, -2.18),
            BLANKET,
        ),
    ];

    add_cat(&mut cubes, -1.65, -3.05, 0.95, CAT_FUR, CAT_ACCENT, GLOW);
    add_cat(&mut cubes, 1.45, -3.0, 0.72, CAT_ACCENT, CAT_FUR, GLOW);
    add_cat_house(&mut cubes, STONE, LAVENDER_WOOD, PINK_PATH, GLOW);
    add_moon_and_stars(&mut cubes, MOON, GLOW);
    add_rocks(&mut cubes, STONE);

    Scene::new(materials, cubes)
}

fn add_cat(
    cubes: &mut Vec<Cube>,
    x: f64,
    z: f64,
    scale: f64,
    fur: usize,
    accent: usize,
    glow: usize,
) {
    let y = -1.15;
    cubes.extend([
        Cube::new(
            Vec3::new(x - 0.42 * scale, y, z - 0.38 * scale),
            Vec3::new(x + 0.42 * scale, y + 0.48 * scale, z + 0.26 * scale),
            fur,
        ),
        Cube::new(
            Vec3::new(x - 0.34 * scale, y + 0.38 * scale, z + 0.12 * scale),
            Vec3::new(x + 0.34 * scale, y + 0.92 * scale, z + 0.66 * scale),
            fur,
        ),
        Cube::new(
            Vec3::new(x - 0.35 * scale, y + 0.84 * scale, z + 0.19 * scale),
            Vec3::new(x - 0.12 * scale, y + 1.12 * scale, z + 0.42 * scale),
            fur,
        ),
        Cube::new(
            Vec3::new(x + 0.12 * scale, y + 0.84 * scale, z + 0.19 * scale),
            Vec3::new(x + 0.35 * scale, y + 1.12 * scale, z + 0.42 * scale),
            fur,
        ),
        Cube::new(
            Vec3::new(x - 0.20 * scale, y + 0.55 * scale, z + 0.67 * scale),
            Vec3::new(x - 0.08 * scale, y + 0.68 * scale, z + 0.72 * scale),
            glow,
        ),
        Cube::new(
            Vec3::new(x + 0.08 * scale, y + 0.55 * scale, z + 0.67 * scale),
            Vec3::new(x + 0.20 * scale, y + 0.68 * scale, z + 0.72 * scale),
            glow,
        ),
        Cube::new(
            Vec3::new(x - 0.07 * scale, y + 0.43 * scale, z + 0.68 * scale),
            Vec3::new(x + 0.07 * scale, y + 0.51 * scale, z + 0.73 * scale),
            accent,
        ),
        Cube::new(
            Vec3::new(x - 0.32 * scale, y, z - 0.34 * scale),
            Vec3::new(x - 0.15 * scale, y + 0.14 * scale, z + 0.34 * scale),
            accent,
        ),
        Cube::new(
            Vec3::new(x + 0.15 * scale, y, z - 0.34 * scale),
            Vec3::new(x + 0.32 * scale, y + 0.14 * scale, z + 0.34 * scale),
            accent,
        ),
        Cube::new(
            Vec3::new(x + 0.40 * scale, y + 0.12 * scale, z - 0.38 * scale),
            Vec3::new(x + 0.58 * scale, y + 0.30 * scale, z + 0.36 * scale),
            fur,
        ),
        Cube::new(
            Vec3::new(x + 0.52 * scale, y + 0.26 * scale, z + 0.18 * scale),
            Vec3::new(x + 0.70 * scale, y + 0.46 * scale, z + 0.52 * scale),
            fur,
        ),
    ]);
}

fn add_cat_house(cubes: &mut Vec<Cube>, stone: usize, wood: usize, roof: usize, glow: usize) {
    cubes.extend([
        Cube::new(
            Vec3::new(-2.7, -1.15, -5.45),
            Vec3::new(-1.35, -0.18, -4.15),
            wood,
        ),
        Cube::new(
            Vec3::new(-2.45, -1.1, -4.13),
            Vec3::new(-1.62, -0.35, -4.02),
            stone,
        ),
        Cube::new(
            Vec3::new(-2.15, -1.1, -4.0),
            Vec3::new(-1.92, -0.55, -3.92),
            roof,
        ),
        Cube::new(
            Vec3::new(-2.84, -0.24, -5.58),
            Vec3::new(-1.2, 0.03, -4.02),
            roof,
        ),
        Cube::new(
            Vec3::new(-2.58, 0.03, -5.35),
            Vec3::new(-1.46, 0.25, -4.25),
            roof,
        ),
        Cube::new(
            Vec3::new(-2.58, -0.02, -3.96),
            Vec3::new(-2.35, 0.20, -3.90),
            glow,
        ),
        Cube::new(
            Vec3::new(-1.62, -0.02, -3.96),
            Vec3::new(-1.39, 0.20, -3.90),
            glow,
        ),
    ]);
}

fn add_moon_and_stars(cubes: &mut Vec<Cube>, moon: usize, glow: usize) {
    cubes.extend([
        Cube::new(
            Vec3::new(2.10, 0.72, -6.35),
            Vec3::new(2.82, 1.44, -6.25),
            moon,
        ),
        Cube::new(
            Vec3::new(2.48, 1.02, -6.22),
            Vec3::new(2.92, 1.52, -6.12),
            glow,
        ),
        Cube::new(
            Vec3::new(-2.85, 0.7, -6.2),
            Vec3::new(-2.66, 0.9, -6.1),
            glow,
        ),
        Cube::new(
            Vec3::new(-0.42, 0.95, -6.28),
            Vec3::new(-0.25, 1.12, -6.18),
            glow,
        ),
        Cube::new(
            Vec3::new(0.75, 0.45, -6.18),
            Vec3::new(0.92, 0.62, -6.08),
            glow,
        ),
        Cube::new(
            Vec3::new(1.58, 0.2, -6.12),
            Vec3::new(1.72, 0.34, -6.02),
            glow,
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

fn ray_color(ray: Ray, scene: &Scene, depth: usize) -> Color {
    if depth == 0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    if let Some(hit) = scene.hit(ray, 0.001, f64::INFINITY) {
        let material = scene.materials[hit.material_index];
        let texture_color = material.sample(hit.u, hit.v, hit.point);
        let direct_color = shade_hit(ray, scene, hit.point, hit.normal, material, texture_color);
        let reflected_color = reflected_color(ray, scene, hit.point, hit.normal, material, depth);
        let refracted_color = refracted_color(ray, scene, &hit, material, depth);

        let surface_weight =
            (1.0 - material.reflectivity - material.transparency * 0.6).clamp(0.15, 1.0);

        return direct_color * surface_weight
            + reflected_color * material.reflectivity
            + refracted_color * material.transparency;
    }

    skybox_color(ray)
}

fn shade_hit(
    ray: Ray,
    scene: &Scene,
    point: Vec3,
    normal: Vec3,
    material: Material,
    texture_color: Color,
) -> Color {
    let light_direction = LIGHT_DIRECTION.unit();
    let diffuse = normal.dot(light_direction).max(0.0);
    let shadow = shadow_factor(scene, point, normal, light_direction);
    let ambient = 0.25;
    let lambert = ambient + diffuse * shadow * 0.72;
    let view_direction = -ray.direction.unit();
    let half_vector = (light_direction + view_direction).unit();
    let specular = normal
        .dot(half_vector)
        .max(0.0)
        .powf(24.0 + material.specular * 64.0)
        * material.specular
        * shadow;

    texture_color * lambert
        + Vec3::new(0.76, 0.84, 1.0) * specular
        + texture_color * material.emission
}

fn shadow_factor(scene: &Scene, point: Vec3, normal: Vec3, light_direction: Vec3) -> f64 {
    let shadow_ray = Ray::new(point + normal * RAY_BIAS, light_direction);

    if scene.hit(shadow_ray, 0.001, f64::INFINITY).is_some() {
        0.35
    } else {
        1.0
    }
}

fn reflected_color(
    ray: Ray,
    scene: &Scene,
    point: Vec3,
    normal: Vec3,
    material: Material,
    depth: usize,
) -> Color {
    if material.reflectivity <= 0.0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    let direction = ray.direction.unit().reflect(normal).unit();
    let reflected_ray = Ray::new(point + normal * RAY_BIAS, direction);

    ray_color(reflected_ray, scene, depth - 1)
}

fn refracted_color(
    ray: Ray,
    scene: &Scene,
    hit: &scene::HitRecord,
    material: Material,
    depth: usize,
) -> Color {
    if material.transparency <= 0.0 {
        return Vec3::new(0.0, 0.0, 0.0);
    }

    let unit_direction = ray.direction.unit();
    let eta_ratio = if hit.front_face {
        1.0 / material.refractive_index
    } else {
        material.refractive_index
    };
    let direction = unit_direction
        .refract(hit.normal, eta_ratio)
        .unwrap_or_else(|| unit_direction.reflect(hit.normal));
    let refracted_ray = Ray::new(hit.point + direction * RAY_BIAS, direction.unit());

    ray_color(refracted_ray, scene, depth - 1)
}

fn skybox_color(ray: Ray) -> Color {
    let unit_direction = ray.direction.unit();
    let blend = 0.5 * (unit_direction.y + 1.0);
    let horizon = Vec3::new(0.08, 0.12, 0.24);
    let zenith = Vec3::new(0.02, 0.03, 0.12);
    let moon_direction = Vec3::new(0.45, 0.55, -0.7).unit();
    let moon = unit_direction.dot(moon_direction).max(0.0).powf(360.0);
    let moon_halo = unit_direction.dot(moon_direction).max(0.0).powf(18.0);
    let star_seed = (unit_direction.x * 91.7).sin()
        * (unit_direction.y * 47.3).cos()
        * (unit_direction.z * 63.1).sin();
    let stars = star_seed.abs().powf(34.0) * blend.max(0.0);

    (1.0 - blend) * horizon
        + blend * zenith
        + Vec3::new(0.96, 0.9, 0.66) * (moon * 1.4)
        + Vec3::new(0.50, 0.60, 0.95) * (moon_halo * 0.25)
        + Vec3::new(0.92, 0.95, 1.0) * (stars * 1.2)
}
