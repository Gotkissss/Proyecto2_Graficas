mod algebra;
mod camara;
mod lienzo;
mod materiales;
mod mundo;
mod pintor;
mod texturas;
mod trazador;

use camara::CamaraOrbital;
use lienzo::Lienzo;
use materiales::{Bloque, Bodega};
use mundo::Mundo;
use raylib::prelude::*;
use trazador::Escena;

const ANCHO_VENTANA: i32 = 1100;
const ALTO_VENTANA: i32 = 700;

fn mundo_de_prueba() -> Mundo {
    let mut m = Mundo::vacio(16, 8, 16);
    for x in 0..16 {
        for z in 0..16 {
            m.poner(x, 0, z, Bloque::Piedra.id());
            m.poner(x, 1, z, if (x + z) % 5 == 0 { Bloque::Arena.id() } else { Bloque::Pasto.id() });
        }
    }
    for y in 2..5 {
        m.poner(4, y, 4, Bloque::Tronco.id());
        m.poner(10, y, 7, Bloque::Ladrillo.id());
    }
    m.poner(4, 5, 4, Bloque::Hojas.id());
    m.poner(10, 5, 7, Bloque::PiedraLuz.id());
    m.poner(7, 2, 11, Bloque::Oro.id());
    m.poner(8, 2, 11, Bloque::Adoquin.id());
    m.poner(9, 2, 11, Bloque::Tablones.id());
    m
}

/// `diorama --foto salida.png` renderiza un cuadro y sale, sin abrir ventana.
fn modo_foto(escena: &Escena, camara: &CamaraOrbital, ruta: &str) {
    let mut lienzo = Lienzo::nuevo(ANCHO_VENTANA as usize, ALTO_VENTANA as usize);
    pintor::pintar(escena, camara, &mut lienzo);
    lienzo.guardar_png(ruta);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let foto = args.iter().position(|a| a == "--foto").and_then(|i| args.get(i + 1));

    let escena = Escena { mundo: mundo_de_prueba(), bodega: Bodega::surtir() };
    let mut camara = CamaraOrbital::nueva(escena.mundo.centro(), 28.0);

    if let Some(ruta) = foto {
        modo_foto(&escena, &camara, ruta);
        return;
    }

    let (mut rl, hilo) = raylib::init()
        .size(ANCHO_VENTANA, ALTO_VENTANA)
        .title("Diorama - Proyecto 2")
        .build();

    let mut lienzo = Lienzo::nuevo(ANCHO_VENTANA as usize / 2, ALTO_VENTANA as usize / 2);
    let imagen = Image::gen_image_color(lienzo.ancho as i32, lienzo.alto as i32, Color::BLACK);
    let mut pantalla = rl.load_texture_from_image(&hilo, &imagen).expect("no se pudo crear la textura");

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

        pintor::pintar(&escena, &camara, &mut lienzo);
        pantalla.update_texture(&lienzo.bytes).expect("lienzo de tamaño distinto a la textura");

        let mut d = rl.begin_drawing(&hilo);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            &pantalla,
            Rectangle::new(0.0, 0.0, lienzo.ancho as f32, lienzo.alto as f32),
            Rectangle::new(0.0, 0.0, ANCHO_VENTANA as f32, ALTO_VENTANA as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        d.draw_fps(10, 10);
    }
}
