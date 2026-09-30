//! Ruido hecho a mano (value noise) para no depender de ninguna librería.

/// Número "al azar" en [0, 1) que siempre sale igual para la misma celda y semilla.
pub fn azar(x: i32, y: i32, semilla: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ semilla.wrapping_mul(2_246_822_519);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^= n >> 16;
    (n & 0x00FF_FFFF) as f32 / 0x0100_0000 as f32
}

/// Ruido suave: valores al azar en las esquinas de cada celda, interpolados.
pub fn ruido(x: f32, y: f32, semilla: u32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - ix as f32, y - iy as f32);
    // curva suave para que no se noten las costuras entre celdas
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));

    let a = azar(ix, iy, semilla);
    let b = azar(ix + 1, iy, semilla);
    let c = azar(ix, iy + 1, semilla);
    let d = azar(ix + 1, iy + 1, semilla);
    let arriba = a + (b - a) * sx;
    let abajo = c + (d - c) * sx;
    arriba + (abajo - arriba) * sy
}

/// Varias capas de ruido, cada una con el doble de detalle y la mitad de peso.
/// Devuelve algo entre 0 y 1.
pub fn fractal(x: f32, y: f32, octavas: u32, semilla: u32) -> f32 {
    let (mut total, mut peso, mut suma_pesos) = (0.0, 1.0, 0.0);
    let (mut px, mut py) = (x, y);
    for o in 0..octavas {
        total += ruido(px, py, semilla.wrapping_add(o * 101)) * peso;
        suma_pesos += peso;
        peso *= 0.5;
        px = px * 2.0 + 17.3;
        py = py * 2.0 - 9.1;
    }
    total / suma_pesos
}
