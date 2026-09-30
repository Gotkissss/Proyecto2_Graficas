use crate::algebra::{Tinte, Vec3};
use crate::luces::LuzDelCielo;
use crate::materiales::{Bodega, Material};
use crate::mundo::{AIRE, Golpe, Mundo};

/// separación para que los rayos secundarios no choquen con la cara de donde salen
const PELLIZCO: f32 = 1e-3;
const REBOTES_MAX: u32 = 5;
/// un rayo secundario que aporta menos que esto al pixel ya no vale la pena
const APORTE_MINIMO: f32 = 0.02;

pub struct Escena {
    pub mundo: Mundo,
    pub bodega: Bodega,
    pub cielo: LuzDelCielo,
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
        self.seguir(origen, dir, AIRE, 0, 1.0)
    }

    fn fondo(&self, _dir: Vec3) -> Tinte {
        Tinte::new(0.45, 0.65, 0.95) * self.cielo.claridad
    }

    /// Sigue un rayo por la escena. `medio` es el bloque dentro del cual viaja (aire,
    /// agua o vidrio) y `aporte` es cuánto pesa este rayo en el color final del pixel.
    fn seguir(&self, origen: Vec3, dir: Vec3, medio: u8, rebote: u32, aporte: f32) -> Tinte {
        let Some(golpe) = self.primer_golpe(origen, dir, medio) else {
            return self.fondo(dir);
        };
        if medio == AIRE {
            return self.sombrear(&golpe, origen, dir, medio, rebote, aporte);
        }

        // el rayo venía por dentro de algo transparente: ese tramo se tiñe (ley de Beer)
        let por_dentro = self.bodega.material(medio);
        let a = por_dentro.absorcion * (-golpe.distancia);
        let filtro = Tinte::new(a.x.exp(), a.y.exp(), a.z.exp());

        let color = if golpe.bloque == AIRE {
            self.salir_al_aire(&golpe, origen, dir, medio, rebote, aporte)
        } else {
            self.sombrear(&golpe, origen, dir, medio, rebote, aporte)
        };
        color * filtro
    }

    fn salir_al_aire(&self, golpe: &Golpe, origen: Vec3, dir: Vec3, medio: u8, rebote: u32, aporte: f32) -> Tinte {
        let material = self.bodega.material(medio);
        let punto = origen + dir * golpe.distancia;
        let normal = golpe.normal();

        // el marco del vidrio también se ve por la cara de atrás
        let mut cara = *golpe;
        cara.signo = -golpe.signo;
        let (u, v) = coordenadas_de_cara(punto, golpe.eje);
        let texel = self.bodega.texturas[self.textura_de_cara(material, &cara)].texel(u, v);
        if texel[3] > 0.9 {
            let luz = self.cielo.ambiente + self.cielo.color * 0.5;
            return Tinte::new(texel[0], texel[1], texel[2]) * luz * material.albedo;
        }

        if rebote >= REBOTES_MAX {
            return self.fondo(dir);
        }
        match dir.doblar(normal, material.indice_refraccion) {
            Some(doblado) => self.seguir(punto - normal * PELLIZCO, doblado, AIRE, rebote + 1, aporte),
            // reflexión interna total: el rayo se queda adentro
            None => self.seguir(punto + normal * PELLIZCO, dir.rebotar(normal), medio, rebote + 1, aporte),
        }
    }

    fn textura_de_cara(&self, material: &Material, golpe: &Golpe) -> usize {
        if golpe.eje != 1 {
            material.lado
        } else if golpe.signo > 0.0 {
            material.arriba
        } else {
            material.abajo
        }
    }

    /// ¿El rayo pasó justo por un hueco de la textura (hojas)?
    fn cae_en_hueco(&self, material: &Material, golpe: &Golpe, origen: Vec3, dir: Vec3) -> bool {
        let (u, v) = coordenadas_de_cara(origen + dir * golpe.distancia, golpe.eje);
        let cual = self.textura_de_cara(material, golpe);
        self.bodega.texturas[cual].texel(u, v)[3] < 0.5
    }

    /// Lo primero que encuentra el rayo que no sea el medio por donde va.
    /// Si el rayo viaja dentro del agua, el golpe puede ser la salida al aire.
    fn primer_golpe(&self, origen: Vec3, dir: Vec3, medio: u8) -> Option<Golpe> {
        let mut hallado = None;
        self.mundo.caminar(origen, dir, f32::INFINITY, |paso| {
            if paso.bloque == medio {
                return false;
            }
            if paso.bloque != AIRE {
                let material = self.bodega.material(paso.bloque);
                if material.calado && self.cae_en_hueco(material, paso, origen, dir) {
                    return false;
                }
            }
            hallado = Some(*paso);
            true
        });
        hallado
    }

    /// Rayo de sombra: devuelve cuánta luz (y de qué color) logra llegar.
    /// El agua y el vidrio no tapan del todo, solo tiñen.
    fn luz_que_llega(&self, origen: Vec3, dir: Vec3, alcance: f32) -> Tinte {
        let mut filtro = Tinte::UNO;
        let mut anterior = AIRE;
        self.mundo.caminar(origen, dir, alcance, |paso| {
            let id = paso.bloque;
            if id == AIRE {
                anterior = AIRE;
                return false;
            }
            let material = self.bodega.material(id);
            if material.emision > 0.0 {
                return false;
            }
            if material.calado {
                if self.cae_en_hueco(material, paso, origen, dir) {
                    return false;
                }
            } else if material.deja_pasar_luz() {
                let (u, v) = coordenadas_de_cara(origen + dir * paso.distancia, paso.eje);
                let texel = self.bodega.texturas[self.textura_de_cara(material, paso)].texel(u, v);
                // el marco del vidrio sí es opaco
                if texel[3] < 0.9 {
                    let tinte = Tinte::new(texel[0], texel[1], texel[2]);
                    let peso = if id == anterior { 0.12 } else { 0.3 };
                    filtro = filtro * Tinte::UNO.mezclar(tinte, peso) * material.transparencia;
                    anterior = id;
                    return false;
                }
            }
            filtro = Tinte::CERO;
            true
        });
        filtro
    }

    /// Oclusión ambiental al estilo minecraft: se mira qué vecinos tapan cada esquina
    /// de la cara y se interpola según dónde cayó el rayo. Son solo 8 lecturas de la grilla.
    fn oclusion(&self, golpe: &Golpe, punto: Vec3) -> f32 {
        let (eje_a, eje_b) = match golpe.eje {
            0 => (2, 1),
            1 => (0, 2),
            _ => (0, 1),
        };
        let mut frente = golpe.celda;
        frente[golpe.eje] += golpe.signo as i32;

        let tapa = |da: i32, db: i32| -> f32 {
            let mut c = frente;
            c[eje_a] += da;
            c[eje_b] += db;
            let id = self.mundo.bloque(c[0], c[1], c[2]);
            if id != AIRE && self.bodega.material(id).hace_sombra_suave() { 1.0 } else { 0.0 }
        };
        let esquina = |da: i32, db: i32| -> f32 {
            let (lado_a, lado_b) = (tapa(da, 0), tapa(0, db));
            if lado_a + lado_b > 1.5 {
                return 0.0;
            }
            (3.0 - lado_a - lado_b - tapa(da, db)) / 3.0
        };

        let fa = punto.componente(eje_a) - punto.componente(eje_a).floor();
        let fb = punto.componente(eje_b) - punto.componente(eje_b).floor();
        let abajo = esquina(-1, -1) * (1.0 - fa) + esquina(1, -1) * fa;
        let arriba = esquina(-1, 1) * (1.0 - fa) + esquina(1, 1) * fa;
        abajo * (1.0 - fb) + arriba * fb
    }

    fn sombrear(&self, golpe: &Golpe, origen: Vec3, dir: Vec3, medio: u8, rebote: u32, aporte: f32) -> Tinte {
        let material = self.bodega.material(golpe.bloque);
        let punto = origen + dir * golpe.distancia;
        let normal = golpe.normal();
        let afuera = punto + normal * PELLIZCO;

        let (u, v) = coordenadas_de_cara(punto, golpe.eje);
        let texel = self.bodega.texturas[self.textura_de_cara(material, golpe)].texel(u, v);
        let base = Tinte::new(texel[0], texel[1], texel[2]);

        let oclusion = 0.35 + 0.65 * self.oclusion(golpe, punto);
        let mut difusa = self.cielo.ambiente * oclusion;
        let mut brillo = Tinte::CERO;

        let hacia_luz = self.cielo.hacia;
        let de_frente = normal.punto(hacia_luz);
        if de_frente > 0.0 {
            let llega = self.luz_que_llega(afuera, hacia_luz, f32::INFINITY);
            if llega.mayor() > 0.0 {
                let luz = self.cielo.color * llega;
                difusa += luz * (de_frente * (0.6 + 0.4 * oclusion));

                // blinn-phong
                let medio_camino = (hacia_luz - dir).unitario();
                let reflejo = normal.punto(medio_camino).max(0.0).powf(material.pulido);
                brillo += luz * (reflejo * material.especular);
            }
        }

        let superficie = base * difusa * material.albedo;
        if rebote >= REBOTES_MAX {
            return superficie + brillo;
        }

        // la textura manda qué partes del bloque son transparentes (el marco del vidrio no)
        let transparencia = material.transparencia * (1.0 - texel[3]);
        if transparencia > 0.01 {
            let n1 = self.bodega.material(medio).indice_refraccion;
            let n2 = material.indice_refraccion;

            // fresnel (aproximación de Schlick): de lado refleja más que de frente
            let r0 = ((n1 - n2) / (n1 + n2)).powi(2);
            let coseno = (-dir.punto(normal)).clamp(0.0, 1.0);
            let mut espejo = (r0 + (1.0 - r0) * (1.0 - coseno).powi(5)).max(material.reflejo);

            let doblado = dir.doblar(normal, n1 / n2);
            if doblado.is_none() {
                espejo = 1.0;
            }

            let rebotado = dir.rebotar(normal);
            let reflejado = if aporte * transparencia * espejo > APORTE_MINIMO {
                self.seguir(afuera, rebotado, medio, rebote + 1, aporte * transparencia * espejo)
            } else {
                self.fondo(rebotado)
            };
            let refractado = match doblado {
                Some(d) => {
                    let peso = aporte * transparencia * (1.0 - espejo);
                    self.seguir(punto - normal * PELLIZCO, d, golpe.bloque, rebote + 1, peso)
                }
                None => Tinte::CERO,
            };

            let a_traves = reflejado * espejo + refractado * (1.0 - espejo);
            return superficie * (1.0 - transparencia) + a_traves * transparencia + brillo;
        }

        if material.reflejo > 0.0 && aporte * material.reflejo > APORTE_MINIMO {
            let reflejado = self.seguir(afuera, dir.rebotar(normal), medio, rebote + 1, aporte * material.reflejo);
            // los metales tiñen lo que reflejan con su propio color
            let tono = Tinte::UNO.mezclar(base / base.mayor().max(1e-3), 0.5);
            return superficie * (1.0 - material.reflejo) + reflejado * tono * material.reflejo + brillo;
        }

        superficie + brillo
    }
}
