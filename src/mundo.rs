use crate::algebra::Vec3;

pub const AIRE: u8 = 0;

/// Grilla de cubos de lado 1. La celda (x, y, z) ocupa de (x, y, z) a (x+1, y+1, z+1).
pub struct Mundo {
    pub ancho: i32,
    pub alto: i32,
    pub fondo: i32,
    celdas: Vec<u8>,
}

/// Resultado de chocar con una cara de algún cubo.
#[derive(Clone, Copy)]
pub struct Golpe {
    pub distancia: f32,
    pub bloque: u8,
    pub celda: [i32; 3],
    /// eje perpendicular a la cara (0 = x, 1 = y, 2 = z)
    pub eje: usize,
    /// +1 o -1, para qué lado del eje mira la cara
    pub signo: f32,
}

impl Golpe {
    pub fn normal(&self) -> Vec3 {
        match self.eje {
            0 => Vec3::new(self.signo, 0.0, 0.0),
            1 => Vec3::new(0.0, self.signo, 0.0),
            _ => Vec3::new(0.0, 0.0, self.signo),
        }
    }
}

impl Mundo {
    pub fn vacio(ancho: i32, alto: i32, fondo: i32) -> Self {
        Mundo {
            ancho,
            alto,
            fondo,
            celdas: vec![AIRE; (ancho * alto * fondo) as usize],
        }
    }

    #[inline]
    fn indice(&self, x: i32, y: i32, z: i32) -> usize {
        (x + z * self.ancho + y * self.ancho * self.fondo) as usize
    }

    #[inline]
    pub fn adentro(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && y >= 0 && z >= 0 && x < self.ancho && y < self.alto && z < self.fondo
    }

    /// Lo que hay en la celda; fuera de la grilla todo es aire.
    #[inline]
    pub fn bloque(&self, x: i32, y: i32, z: i32) -> u8 {
        if self.adentro(x, y, z) { self.celdas[self.indice(x, y, z)] } else { AIRE }
    }

    pub fn poner(&mut self, x: i32, y: i32, z: i32, bloque: u8) {
        if self.adentro(x, y, z) {
            let i = self.indice(x, y, z);
            self.celdas[i] = bloque;
        }
    }

    pub fn centro(&self) -> Vec3 {
        Vec3::new(self.ancho as f32, self.alto as f32, self.fondo as f32) * 0.5
    }

    /// Avanza el rayo celda por celda (DDA de Amanatides y Woo) hasta topar con algo
    /// distinto al `medio` por el que viaja. Normalmente el medio es aire, pero cuando
    /// el rayo va dentro del agua o del vidrio el "golpe" puede ser la salida al aire.
    ///
    /// Así no hay que probar el rayo contra cada cubo: solo se visitan las celdas que
    /// el rayo realmente atraviesa.
    pub fn recorrer(&self, origen: Vec3, dir: Vec3, medio: u8) -> Option<Golpe> {
        let tam = [self.ancho, self.alto, self.fondo];
        let o = [origen.x, origen.y, origen.z];
        let d = [dir.x, dir.y, dir.z];

        // primero recortar el rayo contra la caja de toda la grilla
        let mut t_entra = 0.0_f32;
        let mut t_sale = f32::INFINITY;
        let mut eje_entrada = usize::MAX;
        for e in 0..3 {
            let limite = tam[e] as f32;
            if d[e].abs() < 1e-9 {
                if o[e] < 0.0 || o[e] >= limite {
                    return None;
                }
                continue;
            }
            let inv = 1.0 / d[e];
            let mut cerca = -o[e] * inv;
            let mut lejos = (limite - o[e]) * inv;
            if cerca > lejos {
                std::mem::swap(&mut cerca, &mut lejos);
            }
            if cerca > t_entra {
                t_entra = cerca;
                eje_entrada = e;
            }
            t_sale = t_sale.min(lejos);
        }
        if t_entra >= t_sale {
            return None;
        }

        let arranque = t_entra + 1e-4;
        let mut celda = [0_i32; 3];
        let mut paso = [0_i32; 3];
        let mut t_cruce = [f32::INFINITY; 3];
        let mut t_delta = [f32::INFINITY; 3];
        for e in 0..3 {
            let p = o[e] + d[e] * arranque;
            celda[e] = (p.floor() as i32).clamp(0, tam[e] - 1);
            if d[e] > 0.0 {
                paso[e] = 1;
                t_delta[e] = 1.0 / d[e];
                t_cruce[e] = ((celda[e] + 1) as f32 - o[e]) / d[e];
            } else if d[e] < 0.0 {
                paso[e] = -1;
                t_delta[e] = -1.0 / d[e];
                t_cruce[e] = (celda[e] as f32 - o[e]) / d[e];
            }
        }

        let mut t = t_entra;
        let mut eje = eje_entrada;
        // si el rayo nace dentro de la grilla, la celda donde nace no cuenta
        let mut en_casa = eje_entrada == usize::MAX;

        loop {
            let id = self.celdas[self.indice(celda[0], celda[1], celda[2])];
            if id != medio && !en_casa {
                return Some(Golpe {
                    distancia: t,
                    bloque: id,
                    celda,
                    eje,
                    signo: -paso[eje] as f32,
                });
            }
            en_casa = false;

            eje = if t_cruce[0] < t_cruce[1] {
                if t_cruce[0] < t_cruce[2] { 0 } else { 2 }
            } else if t_cruce[1] < t_cruce[2] {
                1
            } else {
                2
            };
            t = t_cruce[eje];
            t_cruce[eje] += t_delta[eje];
            celda[eje] += paso[eje];
            if celda[eje] < 0 || celda[eje] >= tam[eje] {
                return None;
            }
        }
    }
}
