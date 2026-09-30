mod algebra;
mod camara;
mod mundo;

use algebra::Vec3;
use camara::CamaraOrbital;
use mundo::Mundo;
use raylib::prelude::*;

const ANCHO_VENTANA: i32 = 1100;
const ALTO_VENTANA: i32 = 700;

fn mundo_de_prueba() -> Mundo {
    let mut m = Mundo::vacio(16, 8, 16);
    for x in 0..16 {
        for z in 0..16 {
            m.poner(x, 0, z, 1);
        }
    }
    for y in 1..4 {
        m.poner(4, y, 4, 2);
        m.poner(10, y, 7, 2);
    }
    m.poner(10, 4, 7, 3);
    m.poner(7, 1, 11, 3);
    m
}

fn pintar(mundo: &Mundo, camara: &CamaraOrbital, lienzo: &mut [u8], ancho: usize, alto: usize) {
    let visor = camara.visor(ancho, alto);
    let luz = Vec3::new(0.5, 0.8, 0.3).unitario();

    for y in 0..alto {
        for x in 0..ancho {
            let dir = visor.direccion(x as f32 + 0.5, y as f32 + 0.5);
            let color = match mundo.recorrer(visor.ojo, dir, mundo::AIRE) {
                Some(golpe) => {
                    let base = match golpe.bloque {
                        1 => Vec3::new(0.4, 0.7, 0.3),
                        2 => Vec3::new(0.6, 0.45, 0.3),
                        _ => Vec3::new(0.8, 0.8, 0.85),
                    };
                    base * (0.25 + 0.75 * golpe.normal().punto(luz).max(0.0))
                }
                None => Vec3::new(0.5, 0.7, 0.95),
            };
            let i = (y * ancho + x) * 4;
            lienzo[i] = (color.x.clamp(0.0, 1.0) * 255.0) as u8;
            lienzo[i + 1] = (color.y.clamp(0.0, 1.0) * 255.0) as u8;
            lienzo[i + 2] = (color.z.clamp(0.0, 1.0) * 255.0) as u8;
            lienzo[i + 3] = 255;
        }
    }
}

fn main() {
    let (mut rl, hilo) = raylib::init()
        .size(ANCHO_VENTANA, ALTO_VENTANA)
        .title("Diorama - Proyecto 2")
        .build();

    let (ancho, alto) = (ANCHO_VENTANA as usize / 2, ALTO_VENTANA as usize / 2);
    let mut lienzo = vec![0_u8; ancho * alto * 4];
    let imagen = Image::gen_image_color(ancho as i32, alto as i32, Color::BLACK);
    let mut pantalla = rl.load_texture_from_image(&hilo, &imagen).expect("no se pudo crear la textura");

    let mundo = mundo_de_prueba();
    let mut camara = CamaraOrbital::nueva(mundo.centro(), 28.0);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        if rl.is_key_down(KeyboardKey::KEY_A) {
            camara.girar(-1.5 * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            camara.girar(1.5 * dt, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_W) {
            camara.girar(0.0, 1.0 * dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            camara.girar(0.0, -1.0 * dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            camara.acercar(1.0 - dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            camara.acercar(1.0 + dt);
        }

        pintar(&mundo, &camara, &mut lienzo, ancho, alto);
        pantalla.update_texture(&lienzo).expect("lienzo de tamaño distinto a la textura");

        let mut d = rl.begin_drawing(&hilo);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &pantalla,
            Rectangle::new(0.0, 0.0, ancho as f32, alto as f32),
            Rectangle::new(0.0, 0.0, ANCHO_VENTANA as f32, ALTO_VENTANA as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        d.draw_fps(10, 10);
    }
}
