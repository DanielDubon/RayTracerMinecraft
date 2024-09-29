use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texture;

use framebuffer::Framebuffer;
use color::Color;
use ray_intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use crate::cube::Cube;
use crate::material::Material;
use rayon::prelude::*;
use texture::Texture;

const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(68, 142, 228);

fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        // Ray is entering the object
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        // Ray is leaving the object
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        // Total internal reflection
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }

    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency)
}


pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    // Crea un búfer temporal para almacenar los colores de los píxeles
    let mut pixel_buffer = vec![0u32; (framebuffer.width * framebuffer.height) as usize];

    // Utiliza paralelización para calcular los colores
    pixel_buffer
        .par_iter_mut()  // Iterador paralelo sobre el búfer
        .enumerate()
        .for_each(|(index, pixel)| {
            let x = (index % framebuffer.width as usize) as u32;
            let y = (index / framebuffer.width as usize) as u32;

            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            // Asigna el color calculado en el buffer de píxeles
            *pixel = pixel_color.to_hex();
        });

    // Finalmente, vuelca el pixel_buffer en el framebuffer
    for (index, &pixel) in pixel_buffer.iter().enumerate() {
        let x = (index % framebuffer.width as usize) as u32;
        let y = (index / framebuffer.width as usize) as u32;
        framebuffer.set_current_color(pixel);
        framebuffer.point(x as usize, y as usize);
    }
}

fn create_cube_grid() -> Vec<Cube> {
    let mut cubes = Vec::new();

    // Cargar texturas
    let top_texture = Texture::new("assets/UP_GRASSTEXTURE.jpg");
    let side_texture = Texture::new("assets/SIDE_GRASSTEXTURE.jpg");

    // Material para la cara superior con la textura de arriba
    let top_material = Material::new(
        Color::new(0, 255, 0), // Color base (solo por defecto)
        0.0,
        [1.0, 0.0, 0.0, 0.0],
        0.0,
        Some(top_texture), // Asignar la textura de la parte superior
    );

    // Material para los lados con la textura lateral
    let side_material = Material::new(
        Color::new(0, 150, 0), // Color base (solo por defecto)
        0.0,
        [1.0, 0.0, 0.0, 0.0],
        0.0,
        Some(side_texture), // Asignar la textura lateral
    );

    let grid_size = 5; // Plataforma 5x5
    let cube_size = 0.5; // Tamaño de cada cubo

    // Calculamos el desplazamiento para centrar el grid
    let offset = (grid_size as f32 * cube_size) / 2.0;
    let y_offset = -2.0; // Desplazamos la base hacia abajo

    for x in 0..grid_size {
        for z in 0..grid_size {
            // Centramos las coordenadas en X y Z, y ajustamos en Y
            let min = Vec3::new(x as f32 * cube_size - offset, y_offset, z as f32 * cube_size - offset);
            let max = Vec3::new((x + 1) as f32 * cube_size - offset, cube_size + y_offset, (z + 1) as f32 * cube_size - offset);

            // Crea un cubo con dos materiales: uno para la parte superior y otro para los lados
            let cube = Cube::new_with_faces(min, max, top_material.clone(), side_material.clone(), vec!["top".to_string(), "left_right".to_string(), "front_back".to_string()]);
            
            // Añadir el cubo a la lista
            cubes.push(cube);
        }
    }

    cubes
}




fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Raytracer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    let cubes = create_cube_grid(); // Declara cubes como mutable



    // Combina los cubos de la base y el marco en una sola colección
   // let all_cubes = [&cubes[..], &black_cubes[..]].concat();

    let light = Light::new(
        Vec3::new(1.0, 1.0, 5.0),
        Color::new(255, 255, 255),
        1.0
    );

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 6.5),  // eye: Initial camera position
        Vec3::new(0.0, 0.0, 0.0),  // center: Point the camera is looking at (origin)
        Vec3::new(0.0, 1.0, 0.0)   // up: World up vector
    );
    let rotation_speed = PI / 50.0;

    let zoom_speed = 0.5;
    const MAX_ZOOM: f32 = 1.0;  // El valor más cercano que la cámara puede estar
    const MIN_ZOOM: f32 = 10.0; // El valor más lejano que la cámara puede estar

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Si presionas la tecla W, la cámara se acerca
        if window.is_key_down(Key::W) {
            if camera.eye.z - zoom_speed > MAX_ZOOM {
                camera.eye.z -= zoom_speed;
            } else {
                camera.eye.z = MAX_ZOOM;
            }
        }
        // Si presionas la tecla S, la cámara se aleja
        if window.is_key_down(Key::S) {
            if camera.eye.z + zoom_speed < MIN_ZOOM {
                camera.eye.z += zoom_speed;
            } else {
                camera.eye.z = MIN_ZOOM;
            }
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed); 
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // render the scene with cubes
        render(&mut framebuffer, &cubes, &camera, &light);

        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
