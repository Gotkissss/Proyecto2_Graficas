use crate::algebra::Vec3;
use raylib::prelude::Image;

/// Imagen ya decodificada y guardada en floats, para no depender de raylib
/// (ni de su hilo) mientras se trazan los rayos.
pub struct Textura {
    ancho: usize,
    alto: usize,
    pixeles: Vec<[f32; 4]>,
}

impl Textura {
    /// `es_color` indica si hay que pasar de sRGB a lineal. Los mapas normales
    /// guardan direcciones, no colores, así que esos se leen tal cual.
    pub fn abrir(ruta: &str, es_color: bool) -> Textura {
        let imagen = Image::load_image(ruta)
            .unwrap_or_else(|_| panic!("no encontré la textura {ruta} (hay que correr desde la raíz del repo)"));
        let ancho = imagen.width as usize;
        let alto = imagen.height as usize;

        let pixeles = imagen
            .get_image_data()
            .iter()
            .map(|c| {
                let canal = |v: u8| {
                    let f = v as f32 / 255.0;
                    if es_color { f.powf(2.2) } else { f }
                };
                [canal(c.r), canal(c.g), canal(c.b), c.a as f32 / 255.0]
            })
            .collect();

        Textura { ancho, alto, pixeles }
    }

    /// Vecino más cercano, que es lo que le da el look pixelado a los bloques.
    #[inline]
    pub fn texel(&self, u: f32, v: f32) -> [f32; 4] {
        let x = ((u * self.ancho as f32) as usize).min(self.ancho - 1);
        let y = ((v * self.alto as f32) as usize).min(self.alto - 1);
        self.pixeles[y * self.ancho + x]
    }

    /// Bilineal, para el cielo (ahí sí se notarían los pixeles).
    pub fn suave(&self, u: f32, v: f32) -> Vec3 {
        let fx = (u * self.ancho as f32 - 0.5).clamp(0.0, (self.ancho - 1) as f32);
        let fy = (v * self.alto as f32 - 0.5).clamp(0.0, (self.alto - 1) as f32);
        let (x0, y0) = (fx as usize, fy as usize);
        let (x1, y1) = ((x0 + 1).min(self.ancho - 1), (y0 + 1).min(self.alto - 1));
        let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);

        let leer = |x: usize, y: usize| {
            let p = self.pixeles[y * self.ancho + x];
            Vec3::new(p[0], p[1], p[2])
        };
        let arriba = leer(x0, y0).mezclar(leer(x1, y0), tx);
        let abajo = leer(x0, y1).mezclar(leer(x1, y1), tx);
        arriba.mezclar(abajo, ty)
    }
}
