mod algebra;
mod boveda;
mod camara;
mod lienzo;
mod luces;
mod materiales;
mod mundo;
mod obras;
mod pintor;
mod ruido;
mod terreno;
mod texturas;
mod trazador;

use boveda::Boveda;
use camara::CamaraOrbital;
use lienzo::Lienzo;
use luces::{Farol, LuzDelCielo};
use materiales::Bodega;
use mundo::Mundo;
use pintor::Cuadrilla;
use raylib::prelude::*;
use trazador::Escena;

const ANCHO_VENTANA: i32 = 1100;
const ALTO_VENTANA: i32 = 700;
const MUESTRAS_FOTO: u32 = 24;
/// cuando la imagen quieta ya juntó estas muestras no vale la pena seguir
const MUESTRAS_MAX: u32 = 48;

const HORAS_POR_SEGUNDO: f32 = 0.55;
const GIRO_AUTOMATICO: f32 = 0.22;

/// Lo que el usuario va cambiando con el teclado y el mouse.
struct Mando {
    camara: CamaraOrbital,
    hora: f32,
    semilla: u32,
    gira_sola: bool,
    corre_el_dia: bool,
    /// entre cuánto se divide la resolución mientras la escena se mueve
    reduccion: usize,
    con_ayuda: bool,
}

fn levantar_isla(semilla: u32, bodega: &Bodega) -> (Mundo, Vec<Farol>) {
    let (mut mundo, mut relieve) = terreno::esculpir_isla(semilla);
    obras::poblar(&mut mundo, &mut relieve, semilla);
    mundo.medir_vacios();
    let faroles = luces::encender_faroles(&mundo, bodega);
    (mundo, faroles)
}

/// Busca `--bandera valor` en los argumentos.
fn argumento<'a>(args: &'a [String], bandera: &str) -> Option<&'a String> {
    args.iter().position(|a| a == bandera).and_then(|i| args.get(i + 1))
}

fn numero(args: &[String], bandera: &str, por_defecto: f32) -> f32 {
    argumento(args, bandera).and_then(|v| v.parse().ok()).unwrap_or(por_defecto)
}

/// `diorama --foto salida.png` renderiza un cuadro y sale, sin abrir ventana.
fn modo_foto(escena: &Escena, camara: &CamaraOrbital, cuadrilla: &Cuadrilla, ruta: &str) {
    let mut lienzo = Lienzo::nuevo(ANCHO_VENTANA as usize, ALTO_VENTANA as usize);
    let inicio = std::time::Instant::now();
    for _ in 0..MUESTRAS_FOTO {
        cuadrilla.pintar(escena, camara, &mut lienzo);
    }
    let ms = inicio.elapsed().as_secs_f32() * 1000.0;
    println!("{MUESTRAS_FOTO} muestras en {ms:.0} ms ({} hilos, {} faroles)", cuadrilla.hilos, escena.faroles.len());
    lienzo.guardar_png(ruta);
}

/// `diorama --medir` da una vuelta completa a la isla sin ventana y dice cuánto tarda
/// cada cuadro. Sirve para comparar antes y después de tocar el trazador.
fn modo_medicion(escena: &mut Escena, mando: &mut Mando, cuadrilla: &Cuadrilla) {
    const CUADROS: u32 = 60;
    for reduccion in [2, 1] {
        let mut lienzo = Lienzo::nuevo(ANCHO_VENTANA as usize / reduccion, ALTO_VENTANA as usize / reduccion);
        let inicio = std::time::Instant::now();
        for _ in 0..CUADROS {
            mando.camara.girar(std::f32::consts::TAU / CUADROS as f32, 0.0);
            lienzo.borrar();
            cuadrilla.pintar(escena, &mando.camara, &mut lienzo);
        }
        let ms = inicio.elapsed().as_secs_f32() * 1000.0 / CUADROS as f32;
        println!("{}x{}: {ms:.1} ms por cuadro ({:.0} fps)", lienzo.ancho, lienzo.alto, 1000.0 / ms);
    }
}

fn leer_controles(rl: &RaylibHandle, mando: &mut Mando, dt: f32) {
    use KeyboardKey::*;

    let camara = &mut mando.camara;
    let sostenida = |a: KeyboardKey, b: KeyboardKey| rl.is_key_down(a) || rl.is_key_down(b);

    if sostenida(KEY_A, KEY_LEFT) {
        camara.girar(-1.4 * dt, 0.0);
    }
    if sostenida(KEY_D, KEY_RIGHT) {
        camara.girar(1.4 * dt, 0.0);
    }
    if sostenida(KEY_W, KEY_UP) {
        camara.girar(0.0, 0.9 * dt);
    }
    if sostenida(KEY_S, KEY_DOWN) {
        camara.girar(0.0, -0.9 * dt);
    }
    if rl.is_key_down(KEY_Q) {
        camara.acercar(1.0 - 1.2 * dt);
    }
    if rl.is_key_down(KEY_E) {
        camara.acercar(1.0 + 1.2 * dt);
    }

    let rueda = rl.get_mouse_wheel_move();
    if rueda != 0.0 {
        camara.acercar(1.0 - rueda * 0.08);
    }
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
        let arrastre = rl.get_mouse_delta();
        camara.girar(-arrastre.x * 0.006, arrastre.y * 0.006);
    }
    if mando.gira_sola {
        camara.girar(GIRO_AUTOMATICO * dt, 0.0);
    }

    if rl.is_key_down(KEY_Z) {
        mando.hora -= 4.0 * dt;
    }
    if rl.is_key_down(KEY_X) {
        mando.hora += 4.0 * dt;
    }
    if mando.corre_el_dia {
        mando.hora += HORAS_POR_SEGUNDO * dt;
    }
    mando.hora = mando.hora.rem_euclid(24.0);

    if rl.is_key_pressed(KEY_SPACE) {
        mando.gira_sola = !mando.gira_sola;
    }
    if rl.is_key_pressed(KEY_T) {
        mando.corre_el_dia = !mando.corre_el_dia;
    }
    if rl.is_key_pressed(KEY_H) {
        mando.con_ayuda = !mando.con_ayuda;
    }
    if rl.is_key_pressed(KEY_R) {
        mando.semilla = mando.semilla.wrapping_mul(1_664_525).wrapping_add(1_013_904_223) % 100_000;
    }
    for (tecla, reduccion) in [(KEY_ONE, 1), (KEY_TWO, 2), (KEY_THREE, 3)] {
        if rl.is_key_pressed(tecla) {
            mando.reduccion = reduccion;
        }
    }
}

fn dibujar_ayuda(d: &mut RaylibDrawHandle, mando: &Mando, detalle: &str) {
    let lineas = [
        format!("{} fps   {}", d.get_fps(), detalle),
        format!("hora {:02}:{:02}   semilla {}", mando.hora as i32, (mando.hora.fract() * 60.0) as i32, mando.semilla),
        String::new(),
        "A/D  W/S  o arrastrar: girar".to_string(),
        "Q/E o rueda: acercar y alejar".to_string(),
        format!("ESPACIO: giro automatico [{}]", if mando.gira_sola { "si" } else { "no" }),
        format!("T: ciclo dia/noche [{}]   Z/X: hora", if mando.corre_el_dia { "si" } else { "no" }),
        "R: generar otra isla".to_string(),
        format!("1/2/3: calidad al moverse [1/{}]", mando.reduccion),
        "P: guardar captura   H: ocultar".to_string(),
    ];

    d.draw_rectangle(8, 8, 330, 22 * lineas.len() as i32 + 12, Color::new(0, 0, 0, 140));
    for (i, linea) in lineas.iter().enumerate() {
        d.draw_text(linea, 16, 14 + 22 * i as i32, 18, Color::RAYWHITE);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    set_trace_log(TraceLogLevel::LOG_WARNING);

    let semilla = numero(&args, "--semilla", 7.0) as u32;
    let bodega = Bodega::surtir();
    let (mundo, faroles) = levantar_isla(semilla, &bodega);

    let mut camara = CamaraOrbital::nueva(mundo.centro(), numero(&args, "--zoom", 50.0));
    camara.giro = numero(&args, "--giro", camara.giro);
    camara.cabeceo = numero(&args, "--cabeceo", camara.cabeceo);

    let mut mando = Mando {
        camara,
        hora: numero(&args, "--hora", 9.5),
        semilla,
        gira_sola: true,
        corre_el_dia: true,
        reduccion: 2,
        con_ayuda: true,
    };

    let mut escena = Escena {
        mundo,
        faroles,
        bodega,
        boveda: Boveda::abrir(),
        cielo: LuzDelCielo::a_las(mando.hora),
        reloj: 0.0,
    };
    let mut cuadrilla = Cuadrilla::reunir();
    // --hilos 1 sirve para ver cuánto ayuda el paralelismo
    cuadrilla.hilos = numero(&args, "--hilos", cuadrilla.hilos as f32).max(1.0) as usize;

    if let Some(ruta) = argumento(&args, "--foto") {
        modo_foto(&escena, &mando.camara, &cuadrilla, ruta);
        return;
    }
    if args.iter().any(|a| a == "--medir") {
        modo_medicion(&mut escena, &mut mando, &cuadrilla);
        return;
    }

    let (mut rl, hilo) = raylib::init()
        .size(ANCHO_VENTANA, ALTO_VENTANA)
        .title("Diorama - isla flotante")
        .build();
    rl.set_target_fps(60);

    let (ancho, alto) = (ANCHO_VENTANA as usize, ALTO_VENTANA as usize);

    // Dos lienzos: el borrador (resolución reducida) se usa mientras algo se mueve,
    // y el fino (resolución completa) va juntando muestras cuando la escena está quieta.
    let mut fino = Lienzo::nuevo(ancho, alto);
    let mut borrador = Lienzo::nuevo(ancho / mando.reduccion, alto / mando.reduccion);

    let crear_textura = |rl: &mut RaylibHandle, lienzo: &Lienzo| {
        let imagen = Image::gen_image_color(lienzo.ancho as i32, lienzo.alto as i32, Color::BLACK);
        let textura = rl.load_texture_from_image(&hilo, &imagen).expect("no se pudo crear la textura");
        textura.set_texture_filter(&hilo, TextureFilter::TEXTURE_FILTER_BILINEAR);
        textura
    };
    let mut pantalla_fina = crear_textura(&mut rl, &fino);
    let mut pantalla_borrador = crear_textura(&mut rl, &borrador);

    let mut capturas = 0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);

        let antes = (mando.camara, mando.hora, mando.semilla, mando.reduccion);
        leer_controles(&rl, &mut mando, dt);
        let se_movio = antes != (mando.camara, mando.hora, mando.semilla, mando.reduccion);

        if mando.semilla != antes.2 {
            (escena.mundo, escena.faroles) = levantar_isla(mando.semilla, &escena.bodega);
        }
        if mando.reduccion != antes.3 {
            borrador = Lienzo::nuevo(ancho / mando.reduccion, alto / mando.reduccion);
            pantalla_borrador = crear_textura(&mut rl, &borrador);
        }

        let usar_borrador = se_movio && mando.reduccion > 1;
        if se_movio {
            escena.cielo = LuzDelCielo::a_las(mando.hora);
            // el agua solo se anima mientras algo cambia; quieta deja que la imagen se afine
            escena.reloj += dt;
            fino.borrar();
        }

        if usar_borrador {
            borrador.borrar();
            cuadrilla.pintar(&escena, &mando.camara, &mut borrador);
            pantalla_borrador.update_texture(&borrador.bytes).expect("el borrador cambió de tamaño");
        } else if fino.muestras < MUESTRAS_MAX {
            cuadrilla.pintar(&escena, &mando.camara, &mut fino);
            pantalla_fina.update_texture(&fino.bytes).expect("el lienzo cambió de tamaño");
        }

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            std::fs::create_dir_all("capturas").ok();
            capturas += 1;
            let ruta = format!("capturas/diorama_{}_{capturas}.png", mando.semilla);
            if usar_borrador { borrador.guardar_png(&ruta) } else { fino.guardar_png(&ruta) }
        }

        let (visible, mostrado) = if usar_borrador { (&pantalla_borrador, &borrador) } else { (&pantalla_fina, &fino) };
        let detalle = format!("{}x{}  {} muestras", mostrado.ancho, mostrado.alto, mostrado.muestras);

        let mut d = rl.begin_drawing(&hilo);
        d.clear_background(Color::BLACK);
        d.draw_texture_pro(
            visible,
            Rectangle::new(0.0, 0.0, mostrado.ancho as f32, mostrado.alto as f32),
            Rectangle::new(0.0, 0.0, ANCHO_VENTANA as f32, ALTO_VENTANA as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        if mando.con_ayuda {
            dibujar_ayuda(&mut d, &mando, &detalle);
        }
    }
}
