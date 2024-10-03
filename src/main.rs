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


    let material_color = if !intersect.material.textures.is_empty() {
        let texture_index = match intersect.face {
            CubeFace::Top => 0, // Grass texture
            _ => 1, // Dirt texture for all other faces
        };
        let (u, v) = intersect.texture_coords();
        intersect.material.textures[texture_index].sample(u, v)
    } else {
        intersect.material.color
    };


    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();


    // Calcula la intensidad de la sombra
    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);


    // Intensidad difusa
    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = material_color * intersect.material.properties[0] * diffuse_intensity * light_intensity;


    // Intensidad especular
    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.shininess);
    let specular = light.color * intersect.material.properties[1] * specular_intensity * light_intensity;


    // Color reflejado
    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.properties[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }


    // Color refractado
    let mut refract_color = Color::black();
    let transparency = intersect.material.properties[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }


    // Combinación de los colores difuso, especular, reflejado y refractado
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
        2.0                        // Incrementa la intensidad si es necesario
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


    let STONE: Material = Material::new(
        Color::new(128, 128, 128),  // Color gris típico de la piedra
        30.0,                       // Brillo moderado, la piedra no refleja mucha luz
        [0.7, 0.1, 0.1, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        1.0                         // Índice de refracción para superficies opacas
    );

    let TREEWOOD: Material = Material::new(
        Color::new(139, 69, 19),    // Color marrón típico de la madera
        30.0,                       // Ajuste el brillo (puede ser más bajo para que la madera no se vea muy brillante)
        [0.7, 0.2, 0.0, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        1.0                         // Índice de refracción (ajustado a 1.0 para superficies opacas)
    );

    let LEAVES: Material = Material::new(
        Color::new(34, 139, 34),    // Color verde típico de las hojas (#228B22)
        20.0,                       // Brillo ligeramente más bajo para las hojas
        [0.6, 0.3, 0.0, 0.0],       // Propiedades: difuso, especular, reflectividad, transparencia
        1.0                         // Índice de refracción para superficies opacas
    );

    // Material para Cristal
    let GLASS: Material = Material::new(
    Color::new(0, 0, 0),  
    30.0,                      
    [0.1, 0.1, 0.1, 0.5],       // Propiedades: bajo difuso, alto especular, sin reflectividad, alta transparencia
    1.0                         // Índice de refracción típico para el vidrio
);
    
    // Define los objetos que componen el portal
    let objects = [
        
        Cube { min: Vec3::new(-4.0, -3.0, -4.0), max: Vec3::new(4.0, -2.0, 4.0), material: GRASS.clone() }, // Base de cesped
       
      // Pared trasera (parte de atrás de la casa)
      Cube { min: Vec3::new(-2.0, -2.0, -2.0), max: Vec3::new(2.0, 0.0, -1.5), material: WOOD.clone() },

      // Pared izquierda
      Cube { min: Vec3::new(-2.0, -2.0, -2.0), max: Vec3::new(-1.5, 0.0, 2.0), material: WOOD.clone() },
   
      // Parte inferior de la pared derecha
      Cube { min: Vec3::new(1.5, -2.0, -2.0), max: Vec3::new(2.0, -1.5, 2.0), material: WOOD.clone() },

    // Parte derecha de la pared derecha
    Cube { min: Vec3::new(1.5, -2.0, -2.0), max: Vec3::new(2.0, 0.0, -0.5), material: WOOD.clone() },   

    // Parte izquierda de la pared derecha
    Cube { min: Vec3::new(1.5, -2.0, 1.5), max: Vec3::new(2.0, 0.0, 0.5), material: WOOD.clone() },

    // Parte superior de la pared derecha (arriba de la ventana)
     Cube { min: Vec3::new(1.5, -0.5, -2.0), max: Vec3::new(2.0, 0.0, 2.0), material: WOOD.clone() },

     // Cristal para la ventana
    Cube { min: Vec3::new(1.5, -2.0, 0.5), max: Vec3::new(2.0, -0.5, -0.5), material: GLASS.clone() },
    

      // Pared frontal izquierda (antes de la puerta)
      Cube { min: Vec3::new(-2.0, -2.0, 1.5), max: Vec3::new(-0.5, 0.0, 2.0), material: WOOD.clone() },
  
      // Pared frontal derecha (después de la puerta)
      Cube { min: Vec3::new(0.5, -2.0, 1.5), max: Vec3::new(2.0, 0.0, 2.0), material: WOOD.clone() },
  
      // Pared frontal encima de la puerta
      Cube { min: Vec3::new(-0.5, -1.0, 1.5), max: Vec3::new(0.5, 0.0, 2.0), material: WOOD.clone() },
  
      // Techo de la casa
      Cube { min: Vec3::new(-2.5, 0.0, -2.5), max: Vec3::new(2.5, 0.5, 2.5), material: STONE.clone() },
      Cube { min: Vec3::new(-2.0, 0.5, -2.0), max: Vec3::new(2.0, 1.0, 2.0), material: STONE.clone() },
      Cube { min: Vec3::new(-1.5, 1.0, -1.5), max: Vec3::new(1.5, 1.5, 1.5), material: STONE.clone() },
      Cube { min: Vec3::new(-1.0, 1.5, -1.0), max: Vec3::new(1.0, 2.0, 1.0), material: STONE.clone() },


     // Tronco del árbol (hecho de 4 cubos de madera apilados en la esquina superior izquierda)
     Cube { min: Vec3::new(-3.5, -2.0, 3.0), max: Vec3::new(-3.0, -1.5, 3.5), material: TREEWOOD.clone() },
     Cube { min: Vec3::new(-3.5, -1.5, 3.0), max: Vec3::new(-3.0, -1.0, 3.5), material: TREEWOOD.clone() },
     Cube { min: Vec3::new(-3.5, -1.0, 3.0), max: Vec3::new(-3.0, -0.5, 3.5), material: TREEWOOD.clone() },
     Cube { min: Vec3::new(-3.5, -0.5, 3.0), max: Vec3::new(-3.0, 0.0, 3.5), material: TREEWOOD.clone() },
 
     // Hojas del árbol (hechas de cubos, ajustadas a la nueva posición)
     Cube { min: Vec3::new(-4.0, 0.0, 2.5), max: Vec3::new(-2.5, 0.5, 4.0), material: LEAVES.clone() }, // Capa inferior de hojas
     Cube { min: Vec3::new(-3.75, 0.5, 2.75), max: Vec3::new(-2.75, 1.0, 3.75), material: LEAVES.clone() }, // Capa superior de hojas
 
  ];


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


        // Dibuja los objetos
        render(&mut framebuffer, &objects, &camera, &light);


        // Actualiza la ventana con el contenido del framebuffer
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();


        std::thread::sleep(frame_delay);
    }
}



