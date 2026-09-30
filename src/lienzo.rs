use crate::algebra::Tinte;
use raylib::prelude::{Color, Image};

/// Buffer RGBA8 donde cae el render antes de subirse a la textura de la ventana.
pub struct Lienzo {
    pub ancho: usize,
    pub alto: usize,
    pub bytes: Vec<u8>,
}

impl Lienzo {
    pub fn nuevo(ancho: usize, alto: usize) -> Self {
        Lienzo { ancho, alto, bytes: vec![255; ancho * alto * 4] }
    }

    pub fn guardar_png(&self, ruta: &str) {
        let mut imagen = Image::gen_image_color(self.ancho as i32, self.alto as i32, Color::BLACK);
        for (i, p) in self.bytes.chunks_exact(4).enumerate() {
            let (x, y) = (i % self.ancho, i / self.ancho);
            imagen.draw_pixel(x as i32, y as i32, Color::new(p[0], p[1], p[2], 255));
        }
        imagen.export_image(ruta);
    }
}

/// De color lineal (puede pasarse de 1) a los 8 bits que van a pantalla.
#[inline]
pub fn revelar(color: Tinte) -> [u8; 4] {
    let canal = |v: f32| (v.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0 + 0.5) as u8;
    [canal(color.x), canal(color.y), canal(color.z), 255]
}
