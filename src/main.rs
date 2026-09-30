mod algebra;
mod boveda;
mod camara;
mod lienzo;
mod luces;
mod materiales;
mod mundo;
mod pintor;
mod ruido;
mod terreno;
mod texturas;
mod trazador;

use camara::CamaraOrbital;
use lienzo::Lienzo;
use luces::LuzDelCielo;
use materiales::Bodega;
use pintor::Cuadrilla;
use raylib::prelude::*;
use trazador::Escena;

const ANCHO_VENTANA: i32 = 1100;
const ALTO_VENTANA: i32 = 700;
const MUESTRAS_FOTO: u32 = 16;

/// `diorama --foto salida.png` renderiza un cuadro y sale, sin abrir ventana.
fn modo_foto(escena: &Escena, camara: &CamaraOrbital, cuadrilla: &Cuadrilla, ruta: &str) {
    let mut lienzo = Lienzo::nuevo(ANCHO_VENTANA as usize, ALTO_VENTANA as usize);
    // varias pasadas para que salga con antialiasing
    for _ in 0..MUESTRAS_FOTO {
        cuadrilla.pintar(escena, camara, &mut lienzo);
    }
    lienzo.guardar_png(ruta);
}

/// Busca `--bandera valor` en los argumentos.
fn argumento<'a>(args: &'a [String], bandera: &str) -> Option<&'a String> {
    args.iter().position(|a| a == bandera).and_then(|i| args.get(i + 1))
}

fn numero(args: &[String], bandera: &str, por_defecto: f32) -> f32 {
    argumento(args, bandera).and_then(|v| v.parse().ok()).unwrap_or(por_defecto)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let foto = argumento(&args, "--foto");
    let hora = numero(&args, "--hora", 10.0);

    let semilla = numero(&args, "--semilla", 7.0) as u32;
    let (mundo, _relieve) = terreno::esculpir_isla(semilla);
    let bodega = Bodega::surtir();
    let escena = Escena {
        faroles: luces::encender_faroles(&mundo, &bodega),
        mundo,
        bodega,
        boveda: boveda::Boveda::abrir(),
        cielo: LuzDelCielo::a_las(hora),
        reloj: 0.0,
    };
    let mut camara = CamaraOrbital::nueva(escena.mundo.centro(), numero(&args, "--zoom", 50.0));
    camara.giro = numero(&args, "--giro", camara.giro);
    camara.cabeceo = numero(&args, "--cabeceo", camara.cabeceo);

    let cuadrilla = Cuadrilla::reunir();

    if let Some(ruta) = foto {
        let inicio = std::time::Instant::now();
        modo_foto(&escena, &camara, &cuadrilla, ruta);
        println!("{} muestras en {:.0} ms con {} hilos", MUESTRAS_FOTO, inicio.elapsed().as_secs_f32() * 1000.0, cuadrilla.hilos);
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

        lienzo.borrar();
        cuadrilla.pintar(&escena, &camara, &mut lienzo);
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
