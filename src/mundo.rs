use crate::algebra::Vec3;
use crate::materiales::Bloque;

pub const AIRE: u8 = Bloque::Aire.id();

/// hasta cuántas celdas de aire libre se miden alrededor de cada celda
const HOLGURA_MAX: u8 = 15;
/// con menos holgura que esto no compensa saltar, sale igual que avanzar normal
const SALTO_MINIMO: i32 = 3;

/// Grilla de cubos de lado 1. La celda (x, y, z) ocupa de (x, y, z) a (x+1, y+1, z+1).
pub struct Mundo {
    pub ancho: i32,
    pub alto: i32,
    pub fondo: i32,
    celdas: Vec<u8>,
    /// Para cada celda de aire, a cuántas celdas está el bloque más cercano (contando
    /// diagonales). Con holgura h, el cubo de radio h-1 alrededor está vacío seguro.
    holgura: Vec<u8>,
    /// caja más chica que encierra todos los bloques: [mínimo, máximo) por eje
    caja_min: [i32; 3],
    caja_max: [i32; 3],
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
        let total = (ancho * alto * fondo) as usize;
        Mundo {
            ancho,
            alto,
            fondo,
            celdas: vec![AIRE; total],
            holgura: vec![0; total],
            caja_min: [0; 3],
            caja_max: [ancho, alto, fondo],
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

    /// Ojo: después de cambiar bloques hay que volver a llamar `medir_vacios`.
    pub fn poner(&mut self, x: i32, y: i32, z: i32, bloque: u8) {
        if self.adentro(x, y, z) {
            let i = self.indice(x, y, z);
            self.celdas[i] = bloque;
        }
    }

    pub fn centro(&self) -> Vec3 {
        Vec3::new(self.ancho as f32, self.alto as f32, self.fondo as f32) * 0.5
    }

    /// Prepara lo que necesita `caminar` para no perder tiempo en el aire: la caja que
    /// encierra a la isla y la holgura de cada celda. La holgura sale de una búsqueda a
    /// lo ancho que arranca desde todos los bloques a la vez y se expande a los 26 vecinos.
    /// Se hace una sola vez por isla, después de construir todo.
    pub fn medir_vacios(&mut self) {
        const SIN_MEDIR: u8 = u8::MAX;
        let mut cola = Vec::new();
        self.caja_min = [self.ancho, self.alto, self.fondo];
        self.caja_max = [0; 3];

        for y in 0..self.alto {
            for z in 0..self.fondo {
                for x in 0..self.ancho {
                    let i = self.indice(x, y, z);
                    if self.celdas[i] == AIRE {
                        self.holgura[i] = SIN_MEDIR;
                        continue;
                    }
                    self.holgura[i] = 0;
                    cola.push([x, y, z]);
                    for (e, v) in [x, y, z].into_iter().enumerate() {
                        self.caja_min[e] = self.caja_min[e].min(v);
                        self.caja_max[e] = self.caja_max[e].max(v + 1);
                    }
                }
            }
        }

        let mut siguiente = 0;
        while siguiente < cola.len() {
            let [x, y, z] = cola[siguiente];
            siguiente += 1;
            let distancia = self.holgura[self.indice(x, y, z)];
            if distancia >= HOLGURA_MAX {
                continue;
            }
            for dy in -1..=1 {
                for dz in -1..=1 {
                    for dx in -1..=1 {
                        let (vx, vy, vz) = (x + dx, y + dy, z + dz);
                        if !self.adentro(vx, vy, vz) {
                            continue;
                        }
                        let v = self.indice(vx, vy, vz);
                        if self.holgura[v] == SIN_MEDIR {
                            self.holgura[v] = distancia + 1;
                            cola.push([vx, vy, vz]);
                        }
                    }
                }
            }
        }
        for h in &mut self.holgura {
            *h = (*h).min(HOLGURA_MAX);
        }
    }

    /// Avanza el rayo celda por celda (DDA de Amanatides y Woo) y le va avisando a
    /// `visita` cada vez que entra a una celda nueva. La visita devuelve `true` cuando
    /// ya encontró lo que buscaba y no hace falta seguir.
    ///
    /// Así no hay que probar el rayo contra cada cubo: solo se tocan las celdas que el
    /// rayo realmente atraviesa. Y donde hay mucho aire alrededor ni siquiera eso: se
    /// salta de una vez todo el cubo vacío que indica la holgura.
    ///
    /// La celda donde nace el rayo no se visita.
    #[inline]
    pub fn caminar(&self, origen: Vec3, dir: Vec3, alcance: f32, mut visita: impl FnMut(&Golpe) -> bool) {
        let (desde, hasta) = (self.caja_min, self.caja_max);
        let o = [origen.x, origen.y, origen.z];
        let d = [dir.x, dir.y, dir.z];
        let mut inv = [0.0_f32; 3];

        // primero recortar el rayo contra la caja de la isla: fuera de ahí solo hay aire
        let mut t_entra = 0.0_f32;
        let mut t_sale = f32::INFINITY;
        let mut eje_entrada = usize::MAX;
        for e in 0..3 {
            let (piso, techo) = (desde[e] as f32, hasta[e] as f32);
            if d[e].abs() < 1e-9 {
                if o[e] < piso || o[e] >= techo {
                    return;
                }
                continue;
            }
            inv[e] = 1.0 / d[e];
            let mut cerca = (piso - o[e]) * inv[e];
            let mut lejos = (techo - o[e]) * inv[e];
            if cerca > lejos {
                std::mem::swap(&mut cerca, &mut lejos);
            }
            if cerca > t_entra {
                t_entra = cerca;
                eje_entrada = e;
            }
            t_sale = t_sale.min(lejos);
        }
        if t_entra >= t_sale || t_entra > alcance {
            return;
        }

        let arranque = t_entra + 1e-4;
        let mut celda = [0_i32; 3];
        let mut paso = [0_i32; 3];
        let mut t_cruce = [f32::INFINITY; 3];
        let mut t_delta = [f32::INFINITY; 3];
        for e in 0..3 {
            let p = o[e] + d[e] * arranque;
            celda[e] = (p.floor() as i32).clamp(desde[e], hasta[e] - 1);
            if inv[e] > 0.0 {
                paso[e] = 1;
                t_delta[e] = inv[e];
                t_cruce[e] = ((celda[e] + 1) as f32 - o[e]) * inv[e];
            } else if inv[e] < 0.0 {
                paso[e] = -1;
                t_delta[e] = -inv[e];
                t_cruce[e] = (celda[e] as f32 - o[e]) * inv[e];
            }
        }

        let mut t = t_entra;
        let mut eje = eje_entrada;
        // si el rayo nace dentro de la caja, la celda donde nace no cuenta
        let mut en_casa = eje_entrada == usize::MAX;

        loop {
            let i = self.indice(celda[0], celda[1], celda[2]);
            if !en_casa {
                let golpe = Golpe { distancia: t, bloque: self.celdas[i], celda, eje, signo: -paso[eje] as f32 };
                if visita(&golpe) {
                    return;
                }
            }
            en_casa = false;

            let libre = self.holgura[i] as i32;
            if libre >= SALTO_MINIMO {
                // por dónde y cuándo sale el rayo del cubo vacío
                let mut t_salida = f32::INFINITY;
                for e in 0..3 {
                    if paso[e] == 0 {
                        continue;
                    }
                    let cara = if paso[e] > 0 { celda[e] + libre } else { celda[e] - libre + 1 };
                    let te = (cara as f32 - o[e]) * inv[e];
                    if te < t_salida {
                        t_salida = te;
                        eje = e;
                    }
                }
                // y rearmar el DDA en la celda donde cae
                for e in 0..3 {
                    if e == eje {
                        celda[e] += paso[e] * libre;
                        t_cruce[e] = t_salida + t_delta[e];
                    } else if paso[e] != 0 {
                        let p = o[e] + d[e] * t_salida;
                        celda[e] = (p.floor() as i32).clamp(celda[e] - libre + 1, celda[e] + libre - 1);
                        let cara = if paso[e] > 0 { celda[e] + 1 } else { celda[e] };
                        t_cruce[e] = (cara as f32 - o[e]) * inv[e];
                    }
                }
                t = t_salida;
                // el cubo vacío puede salirse de la caja, así que aquí se revisan los tres ejes
                if (0..3).any(|e| celda[e] < desde[e] || celda[e] >= hasta[e]) {
                    return;
                }
            } else {
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
            }

            if t > alcance || celda[eje] < desde[eje] || celda[eje] >= hasta[eje] {
                return;
            }
        }
    }
}
