use std::sync::Mutex;
use std::thread;

use crate::camara::CamaraOrbital;
use crate::lienzo::{Lienzo, Revelador};
use crate::trazador::Escena;

/// filas que agarra un hilo cada vez que pide trabajo
const FILAS_POR_TANDA: usize = 4;

/// Reparte el render entre todos los núcleos.
pub struct Cuadrilla {
    pub hilos: usize,
    revelador: Revelador,
}

/// Desplazamiento dentro del pixel para la muestra número `n` (secuencia de Halton).
/// La primera muestra cae justo en el centro.
fn desfase(n: u32) -> (f32, f32) {
    let halton = |mut i: u32, base: u32| {
        let (mut f, mut r) = (1.0_f32, 0.0_f32);
        while i > 0 {
            f /= base as f32;
            r += f * (i % base) as f32;
            i /= base;
        }
        r
    };
    if n == 0 { (0.5, 0.5) } else { (halton(n, 2), halton(n, 3)) }
}

impl Cuadrilla {
    pub fn reunir() -> Self {
        let hilos = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        Cuadrilla { hilos, revelador: Revelador::nuevo() }
    }

    /// Traza una muestra por pixel y la suma a lo acumulado en el lienzo.
    ///
    /// El lienzo se parte en tandas de pocas filas y los hilos las van sacando de una
    /// cola compartida. Así, si a un hilo le toca una zona pesada (agua, vidrio), los
    /// demás siguen avanzando con el resto en vez de quedarse esperando.
    pub fn pintar(&self, escena: &Escena, camara: &CamaraOrbital, lienzo: &mut Lienzo) {
        let ancho = lienzo.ancho;
        let visor = camara.visor(ancho, lienzo.alto);
        let (jx, jy) = desfase(lienzo.muestras);
        let peso = 1.0 / (lienzo.muestras + 1) as f32;

        let tandas = lienzo
            .bytes
            .chunks_mut(ancho * 4 * FILAS_POR_TANDA)
            .zip(lienzo.acumulado.chunks_mut(ancho * FILAS_POR_TANDA))
            .enumerate();
        let cola = Mutex::new(tandas);

        thread::scope(|s| {
            for _ in 0..self.hilos {
                s.spawn(|| {
                    loop {
                        // el candado se suelta apenas se saca la tanda
                        let tanda = cola.lock().unwrap().next();
                        let Some((n, (bytes, acumulado))) = tanda else { break };

                        let fila_inicial = n * FILAS_POR_TANDA;
                        for (i, (pixel, suma)) in bytes.chunks_exact_mut(4).zip(acumulado.iter_mut()).enumerate() {
                            let x = (i % ancho) as f32;
                            let y = (fila_inicial + i / ancho) as f32;
                            let dir = visor.direccion(x + jx, y + jy);

                            *suma += escena.color_de(visor.ojo, dir);
                            pixel.copy_from_slice(&self.revelador.revelar(*suma * peso));
                        }
                    }
                });
            }
        });

        lienzo.muestras += 1;
    }
}
