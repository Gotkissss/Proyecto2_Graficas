use crate::algebra::Vec3;

const CABECEO_MAX: f32 = 1.45;
const ZOOM_MIN: f32 = 6.0;
const ZOOM_MAX: f32 = 110.0;

/// Cámara que orbita alrededor de un punto fijo (el centro del diorama).
#[derive(Clone, Copy, PartialEq)]
pub struct CamaraOrbital {
    pub centro: Vec3,
    pub giro: f32,
    pub cabeceo: f32,
    pub distancia: f32,
    pub apertura: f32,
}

/// Todo lo que hace falta para sacar el rayo de un pixel sin recalcular la base cada vez.
pub struct Visor {
    pub ojo: Vec3,
    esquina: Vec3,
    paso_x: Vec3,
    paso_y: Vec3,
}

impl CamaraOrbital {
    pub fn nueva(centro: Vec3, distancia: f32) -> Self {
        CamaraOrbital {
            centro,
            giro: 0.7,
            cabeceo: 0.45,
            distancia,
            apertura: 55.0_f32.to_radians(),
        }
    }

    pub fn girar(&mut self, d_giro: f32, d_cabeceo: f32) {
        self.giro = (self.giro + d_giro).rem_euclid(std::f32::consts::TAU);
        self.cabeceo = (self.cabeceo + d_cabeceo).clamp(-CABECEO_MAX, CABECEO_MAX);
    }

    /// factor < 1 acerca, factor > 1 aleja
    pub fn acercar(&mut self, factor: f32) {
        self.distancia = (self.distancia * factor).clamp(ZOOM_MIN, ZOOM_MAX);
    }

    pub fn ojo(&self) -> Vec3 {
        let (sg, cg) = self.giro.sin_cos();
        let (sc, cc) = self.cabeceo.sin_cos();
        self.centro + Vec3::new(cc * sg, sc, cc * cg) * self.distancia
    }

    pub fn visor(&self, ancho: usize, alto: usize) -> Visor {
        let ojo = self.ojo();
        let frente = (self.centro - ojo).unitario();
        let derecha = frente.cruz(Vec3::ARRIBA).unitario();
        let arriba = derecha.cruz(frente);

        let medio_alto = (self.apertura * 0.5).tan();
        let medio_ancho = medio_alto * ancho as f32 / alto as f32;

        let paso_x = derecha * (2.0 * medio_ancho / ancho as f32);
        let paso_y = arriba * (-2.0 * medio_alto / alto as f32);
        let esquina = frente - derecha * medio_ancho + arriba * medio_alto;

        Visor { ojo, esquina, paso_x, paso_y }
    }
}

impl Visor {
    /// (px, py) en coordenadas de pixel, pueden llevar fracción para el antialiasing.
    #[inline]
    pub fn direccion(&self, px: f32, py: f32) -> Vec3 {
        (self.esquina + self.paso_x * px + self.paso_y * py).unitario()
    }
}
