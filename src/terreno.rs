use crate::materiales::Bloque;
use crate::mundo::Mundo;
use crate::ruido::{azar, fractal};

pub const LADO: i32 = 40;
pub const ALTURA: i32 = 50;
pub const NIVEL_AGUA: i32 = 20;

const RADIO_ISLA: f32 = 17.5;

/// Lo que se sabe de cada columna de la isla después de esculpirla.
/// Las obras (árboles, cabaña...) lo usan para saber dónde pueden ir.
pub struct Relieve {
    /// y del bloque más alto de la columna, o -1 si ahí no hay isla
    alturas: Vec<i32>,
    /// columnas que ya tienen algo construido encima
    ocupado: Vec<bool>,
    /// ángulo (desde el centro) hacia donde quedó el volcán
    pub rumbo: f32,
}

impl Relieve {
    pub fn altura(&self, x: i32, z: i32) -> i32 {
        if x < 0 || z < 0 || x >= LADO || z >= LADO { -1 } else { self.alturas[(x + z * LADO) as usize] }
    }

    pub fn fijar(&mut self, x: i32, z: i32, y: i32) {
        self.alturas[(x + z * LADO) as usize] = y;
    }

    pub fn hay_isla(&self, x: i32, z: i32) -> bool {
        self.altura(x, z) >= 0
    }

    pub fn libre(&self, x: i32, z: i32) -> bool {
        self.hay_isla(x, z) && !self.ocupado[(x + z * LADO) as usize]
    }

    pub fn ocupar(&mut self, x: i32, z: i32) {
        if x >= 0 && z >= 0 && x < LADO && z < LADO {
            self.ocupado[(x + z * LADO) as usize] = true;
        }
    }
}

fn distancia2(x: f32, z: f32, punto: (f32, f32)) -> f32 {
    (x - punto.0).powi(2) + (z - punto.1).powi(2)
}

/// Genera la isla flotante: un mapa de alturas hecho con ruido fractal, más un volcán
/// y la hondonada del lago, y por debajo una panza de piedra que se va afinando.
/// Con la misma semilla siempre sale la misma isla.
pub fn esculpir_isla(semilla: u32) -> (Mundo, Relieve) {
    let mut mundo = Mundo::vacio(LADO, ALTURA, LADO);
    let centro = LADO as f32 / 2.0;

    let rumbo = azar(3, 7, semilla) * std::f32::consts::TAU;
    let volcan = (centro + rumbo.cos() * 8.5, centro + rumbo.sin() * 8.5);
    let lago = (centro - (rumbo + 0.5).cos() * 7.0, centro - (rumbo + 0.5).sin() * 7.0);

    let celdas = (LADO * LADO) as usize;
    let mut relieve = Relieve { alturas: vec![-1; celdas], ocupado: vec![false; celdas], rumbo };

    // la lava queda un poco más abajo que el borde del cráter
    let cima = 21.0 + 11.0 + fractal(volcan.0 * 0.08, volcan.1 * 0.08, 4, semilla) * 5.0;
    let nivel_lava = cima as i32 - 3;

    for z in 0..LADO {
        for x in 0..LADO {
            let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);

            // contorno irregular: la distancia al centro se deforma con ruido
            let contorno = (fractal(fx * 0.11, fz * 0.11, 3, semilla + 7) - 0.5) * 0.4;
            let lejos = distancia2(fx, fz, (centro, centro)).sqrt() / RADIO_ISLA + contorno;
            if lejos >= 1.0 {
                continue;
            }
            let borde = 1.0 - lejos;

            let al_volcan = distancia2(fx, fz, volcan);
            let al_lago = distancia2(fx, fz, lago);
            let cono = 11.0 * (-al_volcan / 32.0).exp();
            let hondonada = 7.0 * (-al_lago / 32.0).exp();

            let mut altura = 21.0 + fractal(fx * 0.08, fz * 0.08, 4, semilla) * 5.0 + cono - hondonada;
            // la orilla de la isla nunca queda bajo el agua, así el lago no se derrama
            if borde < 0.2 {
                altura = altura.max(NIVEL_AGUA as f32 + 0.5);
            }
            let mut tope = altura as i32;

            let en_crater = al_volcan < 1.7 * 1.7;
            if en_crater {
                tope = nivel_lava;
            }

            // la panza: gruesa en el centro, delgada en las orillas, con picos colgando
            let picos = (fractal(fx * 0.3, fz * 0.3, 2, semilla + 3) - 0.45).max(0.0) * 14.0;
            let grosor = 3.0 + borde.powf(0.8) * 16.0 + picos;
            let piso = (tope - grosor as i32).max(1);

            let arenoso = tope <= NIVEL_AGUA && al_lago < 64.0;
            let rocoso = cono > 2.6;
            for y in piso..=tope {
                let hondo = tope - y;
                let bloque = if en_crater && hondo < 2 {
                    Bloque::Lava
                } else if cono > 9.0 {
                    Bloque::Obsidiana
                } else if rocoso {
                    if azar(x + y * 31, z - y * 17, semilla + 5) < 0.22 { Bloque::Adoquin } else { Bloque::Piedra }
                } else if arenoso && hondo < 3 {
                    Bloque::Arena
                } else if hondo == 0 {
                    Bloque::Pasto
                } else if hondo < 4 {
                    Bloque::Tierra
                } else {
                    Bloque::Piedra
                };
                mundo.poner(x, y, z, bloque.id());
            }

            for y in (tope + 1)..=NIVEL_AGUA {
                mundo.poner(x, y, z, Bloque::Agua.id());
            }

            // de vez en cuando un cristal luminoso colgando de la panza
            if borde > 0.25 && azar(x, z, semilla + 11) < 0.014 {
                mundo.poner(x, piso - 1, z, Bloque::PiedraLuz.id());
                mundo.poner(x, piso - 2, z, Bloque::PiedraLuz.id());
            }

            relieve.fijar(x, z, tope);
        }
    }

    (mundo, relieve)
}
