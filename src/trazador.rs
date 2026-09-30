use crate::algebra::{Tinte, Vec3};
use crate::materiales::Bodega;
use crate::mundo::{AIRE, Golpe, Mundo};

pub struct Escena {
    pub mundo: Mundo,
    pub bodega: Bodega,
}

/// Coordenadas de textura dentro de la cara golpeada.
/// En las caras laterales la v va invertida para que la textura no salga de cabeza.
#[inline]
pub fn coordenadas_de_cara(punto: Vec3, eje: usize) -> (f32, f32) {
    let frac = |v: f32| v - v.floor();
    match eje {
        0 => (frac(punto.z), 1.0 - frac(punto.y)),
        1 => (frac(punto.x), frac(punto.z)),
        _ => (frac(punto.x), 1.0 - frac(punto.y)),
    }
}

impl Escena {
    pub fn color_de(&self, origen: Vec3, dir: Vec3) -> Tinte {
        let Some(golpe) = self.mundo.recorrer(origen, dir, AIRE) else {
            return Tinte::new(0.45, 0.65, 0.95);
        };
        self.sombrear(&golpe, origen, dir)
    }

    fn sombrear(&self, golpe: &Golpe, origen: Vec3, dir: Vec3) -> Tinte {
        let material = self.bodega.material(golpe.bloque);
        let punto = origen + dir * golpe.distancia;
        let normal = golpe.normal();

        let (u, v) = coordenadas_de_cara(punto, golpe.eje);
        let cual = if golpe.eje != 1 {
            material.lado
        } else if golpe.signo > 0.0 {
            material.arriba
        } else {
            material.abajo
        };
        let texel = self.bodega.texturas[cual].texel(u, v);
        let base = Tinte::new(texel[0], texel[1], texel[2]);

        let hacia_luz = Vec3::new(0.45, 0.8, 0.3).unitario();
        let difusa = normal.punto(hacia_luz).max(0.0);
        base * (material.albedo * (0.25 + 0.9 * difusa))
    }
}
