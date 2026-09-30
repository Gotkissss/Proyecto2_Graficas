use crate::camara::CamaraOrbital;
use crate::lienzo::{Lienzo, revelar};
use crate::trazador::Escena;

pub fn pintar(escena: &Escena, camara: &CamaraOrbital, lienzo: &mut Lienzo) {
    let visor = camara.visor(lienzo.ancho, lienzo.alto);
    let ancho = lienzo.ancho;

    for (y, fila) in lienzo.bytes.chunks_exact_mut(ancho * 4).enumerate() {
        for (x, pixel) in fila.chunks_exact_mut(4).enumerate() {
            let dir = visor.direccion(x as f32 + 0.5, y as f32 + 0.5);
            let color = escena.color_de(visor.ojo, dir);
            pixel.copy_from_slice(&revelar(color));
        }
    }
}
