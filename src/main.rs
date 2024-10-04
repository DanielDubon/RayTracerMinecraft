use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;
use rayon::prelude::*;


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
use ray_intersect::{Intersect, RayIntersect, CubeFace};
use camera::Camera;
use light::Light;
use crate::cube::Cube;
use crate::material::Material;
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


fn cast_shadow(intersect: &Intersect, light: &Light, objects: &[Cube]) -> f32 {
    let light_dir = light.position - intersect.point;
    let distance_to_light = light_dir.magnitude();
    
    if distance_to_light > light.radius {
        return 1.0;
    }

    let light_dir = light_dir.normalize();
    let shadow_ray_origin = intersect.point + light_dir * 0.001;

    for object in objects {
        let shadow_intersect = object.intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < distance_to_light {
            return 0.95; // Permitimos que algo de luz pase a través de los objetos
        }
    }

    0.0
}


pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Cube],
    lights: &[Light],
    ambient_color: &Color,
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
        // Simular el color del cielo basado en la dirección del rayo y la posición del sol
        let sun_dir = lights[0].position.normalize();
        let sun_intensity = ray_direction.dot(&sun_dir).max(0.0).powf(20.0);
        let sky_color = ambient_color.mul_scalar(0.5); // Color base del cielo
        let sun_color = Color::new(255, 255, 200).mul_scalar(sun_intensity); // Color del sol
        return sky_color + sun_color;
    }


    let material_color = if !intersect.material.textures.is_empty() {
        let texture_index = match &intersect.material.color {
            color if *color == Color::new(0, 255, 0) => {
                // Esto es césped
                match intersect.face {
                    CubeFace::Top => 0, // Textura de césped para la cara superior
                    _ => 1, // Textura de tierra para las otras caras
                }
            },
            color if *color == Color::new(128, 128, 128) => {
                // Esto es piedra, usa la textura de piedra para todas las caras
                0 // Asumiendo que la textura de piedra es la primera (y única) en el vector de texturas
            },
            _ => 0, // Para otros materiales, usa la primera textura
        };
        let (u, v) = intersect.texture_coords();
        // Escala u y v para que coincidan con el tamaño de unidad de 0.5
        let scaled_u = (u * 2.0) % 1.0;
        let scaled_v = (v * 2.0) % 1.0;
        intersect.material.textures[texture_index].sample(scaled_u, scaled_v)
    } else {
        intersect.material.color
    };


    let is_glowstone = intersect.material.emission != Color::new(0, 0, 0);

    let mut final_color = Color::new(0, 0, 0);

    for light in lights {
        let light_dir = light.position - intersect.point;
        let distance_to_light = light_dir.magnitude();
        
        if distance_to_light <= light.radius {
            let light_dir = light_dir.normalize();
            let shadow_intensity = cast_shadow(&intersect, light, objects);
            if shadow_intensity < 1.0 {
                let attenuation = 1.0 / (1.0 + distance_to_light * distance_to_light / (light.radius * light.radius));
                let light_intensity = (1.0 - shadow_intensity) * light.intensity * attenuation;

                let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0);
                let diffuse = material_color.mul(&light.color).mul_scalar(intersect.material.properties[0] * diffuse_intensity * light_intensity);

                let view_dir = (ray_origin - intersect.point).normalize();
                let halfway = (light_dir + view_dir).normalize();
                let specular_intensity = halfway.dot(&intersect.normal).max(0.0).powf(intersect.material.shininess);
                let specular = light.color.mul_scalar(intersect.material.properties[1] * specular_intensity * light_intensity);

                if is_glowstone {
                    // Para el glowstone, usamos más el color de la textura y menos la iluminación calculada
                    final_color = final_color + material_color.mul_scalar(0.7) + (diffuse + specular).mul_scalar(0.3);
                } else {
                    final_color = final_color + diffuse + specular;
                }
            }
        }
    }

    // Añadimos la emisión de luz del material
    if is_glowstone {
        // Para el glowstone, mezclamos la emisión con el color de la textura
        final_color = final_color.mul_scalar(0.6) + material_color.mul_scalar(0.4);
    } else {
        final_color = final_color + intersect.material.emission;
    }

    // Añade iluminación ambiental
    let ambient = material_color.mul(ambient_color).mul_scalar(0.1);
    final_color = final_color + ambient;

    // Color reflejado
    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.properties[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, lights, ambient_color, depth + 1);
    }


    // Color refractado
    let mut refract_color = Color::black();
    let transparency = intersect.material.properties[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, lights, ambient_color, depth + 1);
    }


    // Combinación de los colores difuso, especular, reflejado, refractado y emitido
    let final_color = final_color * (1.0 - reflectivity - transparency) + 
    (reflect_color * reflectivity) + 
    (refract_color * transparency);


    final_color
}




pub fn render(framebuffer: &mut Framebuffer, objects: &[Cube], camera: &Camera, lights: &[Light], ambient_color: &Color) {
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




            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, lights, ambient_color, 0);




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

fn generate_lights_from_emissive_materials(objects: &[Cube]) -> Vec<Light> {
    let mut lights = Vec::new();
    for cube in objects {
        if cube.material.emission != Color::new(0, 0, 0) {
            let position = (cube.min + cube.max) * 0.5;
            let intensity = cube.material.emission.intensity() * 0.1;
            let radius = (cube.max - cube.min).magnitude() * 2.0;
            lights.push(Light {
                position,
                color: cube.material.emission,
                intensity,
                radius,
            });
        }
    }
    lights
}

fn generate_lights_from_emissive_objects(objects: &[Cube]) -> Vec<Light> {
    objects.iter()
        .filter(|cube| cube.material.emission != Color::new(0, 0, 0))
        .map(|cube| {
            let position = (cube.min + cube.max) * 0.5;
            let intensity = cube.material.emission.intensity() * 10.0;  // Aumentamos significativamente la intensidad
            let radius = (cube.max - cube.min).magnitude() * 10.0;  // Aumentamos aún más el radio
            Light::new(position, cube.material.emission, intensity, radius)
        })
        .collect()
}

struct DayNightCycle {
    time: f32,
    day_color: Color,
    night_color: Color,
    sun_position: Vec3,
}

impl DayNightCycle {
    fn new() -> Self {
        DayNightCycle {
            time: 0.5, // Empezamos a mitad del día
            day_color: Color::new(255, 255, 255),
            night_color: Color::new(10, 10, 50),
            sun_position: Vec3::new(0.0, 5.0, 0.0), // Posición inicial del sol
        }
    }

    fn update(&mut self, delta: f32) {
        self.time += delta;
        if self.time > 1.0 {
            self.time -= 1.0;
        }
        if self.time < 0.0 {
            self.time += 1.0;
        }

        // Actualizar la posición del sol
        let angle = self.time * 2.0 * std::f32::consts::PI;
        self.sun_position = Vec3::new(
            5.0 * angle.cos(),
            5.0 * angle.sin().abs() + 1.0, // Mantiene el sol por encima del horizonte
            5.0 * angle.sin(),
        );
    }

    fn get_current_color(&self) -> Color {
        let t = (self.time * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
        Color::lerp(&self.night_color, &self.day_color, t)
    }

    fn get_light_intensity(&self) -> f32 {
        ((self.time * std::f32::consts::PI * 2.0).sin() * 0.4 + 0.6).max(0.2)
    }
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


    let light = Light::new(
         Vec3::new(4.0, 1.0, 5.0),
        Color::new(255, 255, 255), // Luz blanca
        2.0,                       // Intensidad
        10.0                       // Radio de influencia (ajusta este valor según sea necesario)
    );


    let rubber = Material::new(
        Color::new(80, 0, 0),
        1.0,
        [0.9, 0.1, 0.0, 0.0],
        0.0,
    );


    let ivory = Material::new(
        Color::new(100, 100, 80),
        50.0,
        [0.6, 0.3, 0.6, 0.0],
        0.0,
    );


    let glass = Material::new(
        Color::new(255, 255, 255),
        1425.0,
        [0.0, 10.0, 0.5, 0.5],
        0.3,
    );


    // Define the grass top and dirt side textures
    let grass_top_texture = Texture::load("assets/UP_GRASSTEXTURE.jpg").expect("Failed to load grass top texture");
    let dirt_side_texture = Texture::load("assets/SIDE_GRASSTEXTURE.jpg").expect("Failed to load dirt side texture");


    // Define el material de césped
    let grass_texture = Texture::load("assets/UP_GRASSTEXTURE.jpg").expect("Failed to load grass texture");


    let GRASS = Material::new(
        Color::new(0, 255, 0),  // Color verde
        50.0,                   // Ajuste el brillo si es necesario
        [0.8, 0.2, 0.0, 0.0],   // Ajusta las propiedades: difuso, especular, reflectividad, transparencia
        1.0
    ).with_textures(vec![grass_top_texture, dirt_side_texture]);


    let wood_plank_texture = Texture::load("assets/wood_plank.jpg").expect("Failed to load wood plank texture");


    let WOOD: Material = Material::new(
        Color::new(170, 137, 85),   // Color marrón típico de la madera
        30.0,                       // Ajuste el brillo
        [0.7, 0.2, 0.0, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        2.0                         // Índice de refracción (ajustado a 1.0 para superficies opacas)
    ).with_textures(vec![wood_plank_texture.clone(), wood_plank_texture ]);

    let stone_texture = Texture::load("assets/stone_block.jpg").expect("Failed to load stone texture");

    let STONE: Material = Material::new(
    Color::new(128, 128, 128),  // Color gris típico de la piedra
    30.0,                       // Brillo moderado, la piedra no refleja mucha luz
    [0.7, 0.1, 0.1, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
    1.0                         // Índice de refracción para superficies opacas
).with_textures(vec![stone_texture.clone()]);  // Usa la misma textura para todas las caras
    
    let tree_plank_texture = Texture::load("assets/wood_rawplank.jpg").expect("Failed to load rawtree plank texture");

    let TREEWOOD: Material = Material::new(
        Color::new(139, 69, 19),    // Color marrón típico de la madera
        10.0,                       // Ajuste el brillo (puede ser más bajo para que la madera no se vea muy brillante)
        [0.7, 0.2, 0.0, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        1.0                         // Índice de refracción (ajustado a 1.0 para superficies opacas)
    ).with_textures(vec![tree_plank_texture.clone()]);

    let leaves_texture = Texture::load("assets/leaves_texture.jpg").expect("Failed to load leaves  texture");

    let LEAVES: Material = Material::new(
        Color::new(34, 139, 34),    // Color verde
        10.0,                       // Brillo ligeramente más bajo para las hojas
        [0.6, 0.3, 0.0, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        1.0                         // Índice de refracción para superficies opacas
    ).with_textures(vec![leaves_texture.clone()]);

    // Material para Cristal
    let GLASS: Material = Material::new(
    Color::new(0, 0, 0),  
    60.0,                      
    [0.1, 0.1, 0.1, 0.5],       // Propiedades: bajo difuso, alto especular, sin reflectividad, alta transparencia
    1.0                         // Índice de refracción típico para el vidrio
);
    
    let glowstone_texture = Texture::load("assets/glowstone_texture.jpg").expect("Failed to load glowstone texture");

    let GLOWSTONE: Material = Material::new(
        Color::new(255, 255, 200),  // Color base amarillento
        10.0,                       // Reducimos el brillo para que la textura sea más visible
        [0.9, 0.1, 0.0, 0.0],       // Aumentamos el difuso, reducimos el especular
        1.0
    ).with_textures(vec![glowstone_texture.clone()])
     .with_emission(Color::new(255, 255, 150)); // Mantenemos la emisión fuerte

    // Define los objetos que componen el portal
    let objects = [
        
        Cube { min: Vec3::new(-4.0, -0.5, -4.0), max: Vec3::new(4.0, 0.0, 4.0), material: GRASS.clone() }, // Base de cesped
       
      // Pared trasera
      Cube { min: Vec3::new(-1.5, 0.0, -1.5), max: Vec3::new(1.5, 2.0, -1.0), material: WOOD.clone() },

      // Pared izquierda
      Cube { min: Vec3::new(-1.5, 0.0, -1.5), max: Vec3::new(-1.0, 2.0, 1.5), material: WOOD.clone() },

      // Parte inferior de la pared derecha
      Cube { min: Vec3::new(1.0, 0.0, -1.5), max: Vec3::new(1.5, 0.5, 1.5), material: WOOD.clone() },

      // Parte derecha de la pared derecha
      Cube { min: Vec3::new(1.0, 0.0, -1.5), max: Vec3::new(1.5, 2.0, -0.5), material: WOOD.clone() },   

      // Parte izquierda de la pared derecha
      Cube { min: Vec3::new(1.0, 0.0, 0.5), max: Vec3::new(1.5, 2.0, 1.5), material: WOOD.clone() },

      // Parte superior de la pared derecha (arriba de la ventana)
      Cube { min: Vec3::new(1.0, 1.5, -1.5), max: Vec3::new(1.5, 2.0, 1.5), material: WOOD.clone() },

      // Cristal para la ventana
      Cube { min: Vec3::new(1.0, 0.5, -0.5), max: Vec3::new(1.5, 1.5, 0.5), material: GLASS.clone() },

      // Pared frontal izquierda (antes de la puerta)
      Cube { min: Vec3::new(-1.5, 0.0, 1.0), max: Vec3::new(-0.5, 2.0, 1.5), material: WOOD.clone() },

      // Pared frontal derecha (después de la puerta)
      Cube { min: Vec3::new(0.5, 0.0, 1.0), max: Vec3::new(1.5, 2.0, 1.5), material: WOOD.clone() },

      // Pared frontal encima de la puerta
      Cube { min: Vec3::new(-0.5, 1.0, 1.0), max: Vec3::new(0.5, 2.0, 1.5), material: WOOD.clone() },

      // Techo de la casa
      Cube { min: Vec3::new(-2.0, 2.0, -2.0), max: Vec3::new(2.0, 2.5, 2.0), material: STONE.clone() },
      Cube { min: Vec3::new(-1.5, 2.5, -1.5), max: Vec3::new(1.5, 3.0, 1.5), material: STONE.clone() },
      Cube { min: Vec3::new(-1.0, 3.0, -1.0), max: Vec3::new(1.0, 3.5, 1.0), material: STONE.clone() },
      Cube { min: Vec3::new(-0.5, 3.5, -0.5), max: Vec3::new(0.5, 4.0, 0.5), material: STONE.clone() },

      // Árbol (movido un bloque hacia adelante)
      // Tronco del árbol
      Cube { min: Vec3::new(-3.0, 0.0, 3.0), max: Vec3::new(-2.5, 0.5, 3.5), material: TREEWOOD.clone() },
      Cube { min: Vec3::new(-3.0, 0.5, 3.0), max: Vec3::new(-2.5, 1.0, 3.5), material: TREEWOOD.clone() },
      Cube { min: Vec3::new(-3.0, 1.0, 3.0), max: Vec3::new(-2.5, 1.5, 3.5), material: TREEWOOD.clone() },
      Cube { min: Vec3::new(-3.0, 1.5, 3.0), max: Vec3::new(-2.5, 2.0, 3.5), material: TREEWOOD.clone() },

      // Hojas del árbol
      Cube { min: Vec3::new(-3.5, 2.0, 2.5), max: Vec3::new(-2.0, 2.5, 4.0), material: LEAVES.clone() },
      Cube { min: Vec3::new(-3.5, 2.5, 2.5), max: Vec3::new(-2.0, 3.0, 4.0), material: LEAVES.clone() },
      Cube { min: Vec3::new(-3.0, 3.0, 3.0), max: Vec3::new(-2.5, 3.5, 3.5), material: LEAVES.clone() },

      // Bloque de piedra luminosa al lado de la casa
      Cube { min: Vec3::new(2.0, 0.0, -1.0), max: Vec3::new(2.5, 0.5, -0.5), material: GLOWSTONE.clone() },
  ];


    // Genera luces adicionales a partir de materiales emisivos
    let mut lights = vec![
        Light::new(
            Vec3::new(4.0, 1.0, 5.0),
            Color::new(255, 255, 255),
            1.0,  // Reducimos la intensidad de la luz principal
            10.0
        )
    ];

    // Añade las luces de los objetos emisivos
    lights.extend(generate_lights_from_emissive_objects(&objects));


    // Inicializa la cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 6.5),  // posición inicial de la cámara
        Vec3::new(0.0, 0.0, 0.0),  // punto al que la cámara está mirando (origen)
        Vec3::new(0.0, 1.0, 0.0)   // vector hacia arriba del mundo
    );
    let rotation_speed = PI / 50.0;
    let zoom_speed = 0.5;
    const MAX_ZOOM: f32 = 1.0;
    const MIN_ZOOM: f32 = 10.0;


    let stone_texture = Texture::load("assets/stone_block.jpg").expect("Failed to load stone texture");


    let mut day_night_cycle = DayNightCycle::new();


    while window.is_open() {
        // Escuchar entradas
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
        // Controles de órbita de la cámara
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


        if window.is_key_down(Key::Q) {
            day_night_cycle.update(-0.005); // Avanzar hacia la noche
        }
        if window.is_key_down(Key::E) {
            day_night_cycle.update(0.005);  // Avanzar hacia el día
        }


        let ambient_color = day_night_cycle.get_current_color();
        let light_intensity = day_night_cycle.get_light_intensity();


        // Actualizar la luz principal (sol)
        lights[0].position = day_night_cycle.sun_position;
        lights[0].color = ambient_color;
        lights[0].intensity = light_intensity * 2.0; // Ajusta este factor según sea necesario


        // Dibuja los objetos
        render(&mut framebuffer, &objects, &camera, &lights, &ambient_color);


        // Actualiza la ventana con el contenido del framebuffer
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();


        std::thread::sleep(frame_delay);
    }
}



