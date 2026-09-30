use crate::algebra::{Tinte, Vec3};
use crate::boveda::Boveda;
use crate::luces::{Farol, LuzDelCielo};
use crate::materiales::{Bloque, Bodega, Material};
use crate::mundo::{AIRE, Golpe, Mundo};

/// separación para que los rayos secundarios no choquen con la cara de donde salen
const PELLIZCO: f32 = 1e-3;
const REBOTES_MAX: u32 = 5;
/// un rayo secundario que aporta menos que esto al pixel ya no vale la pena
const APORTE_MINIMO: f32 = 0.02;

pub struct Escena {
    pub mundo: Mundo,
    pub bodega: Bodega,
    pub boveda: Boveda,
    pub cielo: LuzDelCielo,
    pub faroles: Vec<Farol>,
    /// segundos desde que arrancó, para lo que se anima
    pub reloj: f32,
}

const AGUA: u8 = Bloque::Agua.id();

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

    fn fondo(&self, dir: Vec3) -> Tinte {
        self.boveda.mirar(dir, &self.cielo)
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

    /// Saca la normal del mapa normal (que viene en espacio tangente) y la pasa al mundo.
    /// Como todas las caras están alineadas a los ejes, la tangente y la bitangente
    /// salen directo del eje de la cara.
    fn normal_con_relieve(&self, mapa: usize, golpe: &Golpe, u: f32, v: f32) -> Vec3 {
        let t = self.bodega.texturas[mapa].texel(u, v);
        let (nx, ny, nz) = (t[0] * 2.0 - 1.0, t[1] * 2.0 - 1.0, t[2] * 2.0 - 1.0);

        let mundo = match golpe.eje {
            // en las caras laterales la v de la textura va al revés que el eje y
            0 => Vec3::new(nz * golpe.signo, -ny, nx),
            1 => Vec3::new(nx, nz * golpe.signo, ny),
            _ => Vec3::new(nx, -ny, nz * golpe.signo),
        };
        mundo.unitario()
    }

    /// Ondas en la superficie del agua: un par de senos que se mueven con el reloj.
    fn oleaje(&self, punto: Vec3, signo: f32) -> Vec3 {
        let t = self.reloj;
        let dx = (punto.x * 2.3 + t * 1.4).sin() * 0.6 + (punto.x * 5.1 - punto.z * 3.3 + t * 2.2).sin() * 0.4;
        let dz = (punto.z * 2.7 - t * 1.1).sin() * 0.6 + (punto.z * 4.6 + punto.x * 3.9 + t * 1.9).sin() * 0.4;
        Vec3::new(dx * 0.035, signo, dz * 0.035).unitario()
    }

    fn sombrear(&self, golpe: &Golpe, origen: Vec3, dir: Vec3, medio: u8, rebote: u32, aporte: f32) -> Tinte {
        let material = self.bodega.material(golpe.bloque);
        let punto = origen + dir * golpe.distancia;
        let cara = golpe.normal();
        let afuera = punto + cara * PELLIZCO;

        let (u, v) = coordenadas_de_cara(punto, golpe.eje);
        let texel = self.bodega.texturas[self.textura_de_cara(material, golpe)].texel(u, v);
        let base = Tinte::new(texel[0], texel[1], texel[2]);

        // la normal de la cara sirve para despegar los rayos; para iluminar se usa la del relieve
        if material.emision > 0.0 {
            return base * material.emision;
        }

        let normal = if golpe.bloque == AGUA && golpe.eje == 1 {
            self.oleaje(punto, golpe.signo)
        } else if let Some(mapa) = material.relieve {
            self.normal_con_relieve(mapa, golpe, u, v)
        } else {
            cara
        };

        let oclusion = 0.35 + 0.65 * self.oclusion(golpe, punto);
        let mut difusa = self.cielo.ambiente * oclusion;
        let mut brillo = Tinte::CERO;

        // una luz cualquiera (sol, luna o farol) pegándole a este punto
        let mut alumbrar = |hacia_luz: Vec3, color: Tinte, alcance: f32| {
            let de_frente = normal.punto(hacia_luz);
            if de_frente <= 0.0 || cara.punto(hacia_luz) <= 0.0 {
                return;
            }
            let llega = self.luz_que_llega(afuera, hacia_luz, alcance);
            if llega.mayor() <= 0.0 {
                return;
            }
            let luz = color * llega;
            difusa += luz * (de_frente * (0.6 + 0.4 * oclusion));

            // blinn-phong
            let medio_camino = (hacia_luz - dir).unitario();
            let reflejo = normal.punto(medio_camino).max(0.0).powf(material.pulido);
            brillo += luz * (reflejo * material.especular);
        };

        alumbrar(self.cielo.hacia, self.cielo.color, f32::INFINITY);

        // a pleno sol un farol lejano no se nota, así que ni se le tira el rayo de sombra
        let minimo_visible = 0.012 + 0.09 * self.cielo.claridad;
        for farol in &self.faroles {
            let separacion = farol.posicion - afuera;
            let d2 = separacion.punto(separacion);
            if d2 >= farol.alcance * farol.alcance {
                continue;
            }
            let d = d2.sqrt();
            // cae con el cuadrado de la distancia y se corta suave al llegar al alcance
            let ventana = 1.0 - d / farol.alcance;
            let caida = ventana * ventana / (1.0 + 0.06 * d2);
            if caida * farol.color.mayor() < minimo_visible {
                continue;
            }
            alumbrar(separacion / d, farol.color * caida, d);
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
                    self.seguir(punto - cara * PELLIZCO, d, golpe.bloque, rebote + 1, peso)
                }
                None => Tinte::CERO,
            };

            let a_traves = reflejado * espejo + refractado * (1.0 - espejo);
            return superficie * (1.0 - transparencia) + a_traves * transparencia + brillo;
        }

        if material.reflejo > 0.0 {
            // el metal refleja parejo y tiñe el reflejo; lo demás (obsidiana pulida)
            // refleja poco de frente y mucho de lado
            let (espejo, tono) = if material.metalico {
                (material.reflejo, base / base.mayor().max(1e-3))
            } else {
                let coseno = (-dir.punto(normal)).clamp(0.0, 1.0);
                (material.reflejo + (0.75 - material.reflejo) * (1.0 - coseno).powi(5), Tinte::UNO)
            };
            if aporte * espejo > APORTE_MINIMO {
                let reflejado = self.seguir(afuera, dir.rebotar(normal), medio, rebote + 1, aporte * espejo);
                return superficie * (1.0 - espejo) + reflejado * tono * espejo + brillo;
            }
        }

        superficie + brillo
    }
}
