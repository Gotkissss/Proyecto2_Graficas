use raylib::prelude::*;

const ANCHO_VENTANA: i32 = 1100;
const ALTO_VENTANA: i32 = 700;

fn main() {
    let (mut rl, hilo) = raylib::init()
        .size(ANCHO_VENTANA, ALTO_VENTANA)
        .title("Diorama - Proyecto 2")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&hilo);
        d.clear_background(Color::BLACK);
        d.draw_fps(10, 10);
    }
}
