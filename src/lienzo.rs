use crate::algebra::Tinte;
use raylib::prelude::{Color, Image};

/// Donde cae el render antes de subirse a la textura de la ventana.
///
/// Además de los bytes RGBA guarda la suma de colores de cada pixel: mientras la
/// escena no cambie se siguen sumando muestras y los bordes se van suavizando solos.
pub struct Lienzo {
    pub ancho: usize,
    pub alto: usize,
    pub bytes: Vec<u8>,
    pub acumulado: Vec<Tinte>,
    pub muestras: u32,
}

impl Lienzo {
    pub fn nuevo(ancho: usize, alto: usize) -> Self {
        Lienzo {
            ancho,
            alto,
            bytes: vec![255; ancho * alto * 4],
            acumulado: vec![Tinte::CERO; ancho * alto],
            muestras: 0,
        }
    }

    /// Hay que llamarlo cuando algo se movió: lo acumulado ya no sirve.
    pub fn borrar(&mut self) {
        self.acumulado.fill(Tinte::CERO);
        self.muestras = 0;
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

const ENTRADAS: usize = 1024;

/// Pasa de color lineal (que puede pasarse de 1) a los 8 bits de pantalla.
/// La curva gamma sale de una tabla para no llamar a `powf` tres veces por pixel.
pub struct Revelador {
    gamma: [u8; ENTRADAS],
}

impl Revelador {
    pub fn nuevo() -> Self {
        let mut gamma = [0_u8; ENTRADAS];
        for (i, salida) in gamma.iter_mut().enumerate() {
            // la tabla se indexa con la raíz del valor para tener más detalle en los oscuros
            let lineal = (i as f32 / (ENTRADAS - 1) as f32).powi(2);
            *salida = (lineal.powf(1.0 / 2.2) * 255.0 + 0.5) as u8;
        }
        Revelador { gamma }
    }

    #[inline]
    pub fn revelar(&self, color: Tinte) -> [u8; 4] {
        let canal = |v: f32| {
            let v = curva_de_pelicula(v.max(0.0));
            self.gamma[(v.sqrt() * (ENTRADAS - 1) as f32) as usize]
        };
        [canal(color.x), canal(color.y), canal(color.z), 255]
    }
}

/// Tone mapping (ajuste ACES de Narkowicz): comprime los brillos fuertes en vez de
/// cortarlos de golpe, para que la lava y el sol no queden como manchas planas.
#[inline]
fn curva_de_pelicula(x: f32) -> f32 {
    let x = x * 0.85;
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).min(1.0)
}
