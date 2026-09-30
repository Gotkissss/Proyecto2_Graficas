use crate::algebra::{Tinte, Vec3, escalon_suave};
use crate::materiales::Bodega;
use crate::mundo::{AIRE, Mundo};

/// La luz direccional de la escena (sol de día, luna de noche) más la luz ambiente.
pub struct LuzDelCielo {
    /// dirección real del sol, aunque esté bajo el horizonte
    pub sol: Vec3,
    /// hacia dónde está el astro que alumbra ahora mismo
    pub hacia: Vec3,
    pub color: Tinte,
    pub ambiente: Tinte,
    /// 0 = noche cerrada, 1 = pleno día
    pub claridad: f32,
}

impl LuzDelCielo {
    /// `hora` va de 0 a 24. El sol sale a las 6 por +x y se pone a las 18 por -x.
    pub fn a_las(hora: f32) -> Self {
        let angulo = (hora - 6.0) / 24.0 * std::f32::consts::TAU;
        let sol = Vec3::new(angulo.cos(), angulo.sin(), 0.38).unitario();
        let claridad = escalon_suave(-0.22, 0.18, sol.y);

        let de_dia = sol.y > 0.0;
        let hacia = if de_dia { sol } else { -sol };

        // cerca del horizonte la luz se apaga para que el cambio sol/luna no pegue un salto
        let fuerza = escalon_suave(0.0, 0.16, sol.y.abs());
        let color = if de_dia {
            let atardecer = Tinte::new(1.0, 0.5, 0.22);
            let mediodia = Tinte::new(1.0, 0.96, 0.88);
            atardecer.mezclar(mediodia, escalon_suave(0.0, 0.45, sol.y)) * (1.25 * fuerza)
        } else {
            Tinte::new(0.2, 0.27, 0.46) * fuerza
        };

        let ambiente = Tinte::new(0.035, 0.045, 0.085).mezclar(Tinte::new(0.30, 0.38, 0.52), claridad);

        LuzDelCielo { sol, hacia, color, ambiente, claridad }
    }
}

/// Luz puntual que sale de un bloque emisivo.
#[derive(Clone, Copy)]
pub struct Farol {
    pub posicion: Vec3,
    pub color: Tinte,
    pub alcance: f32,
}

const VECINOS: [[i32; 3]; 6] = [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
/// bloques emisivos más cerca que esto se juntan en un solo farol
const RADIO_DE_GRUPO: f32 = 2.6;

/// Recorre el mundo y pone un farol por cada grupo de bloques que emiten luz.
/// Juntarlos en grupos evita tener una luz (y un rayo de sombra) por cada cubo de lava.
pub fn encender_faroles(mundo: &Mundo, bodega: &Bodega) -> Vec<Farol> {
    struct Grupo {
        suma: Vec3,
        color: Tinte,
        cuantos: f32,
    }
    let mut grupos: Vec<Grupo> = Vec::new();

    for y in 0..mundo.alto {
        for z in 0..mundo.fondo {
            for x in 0..mundo.ancho {
                let material = bodega.material(mundo.bloque(x, y, z));
                if material.emision <= 0.0 {
                    continue;
                }
                // solo cuentan los que tienen alguna cara al descubierto
                let mut salida = Vec3::CERO;
                let mut abiertas = 0;
                for v in VECINOS {
                    let vecino = mundo.bloque(x + v[0], y + v[1], z + v[2]);
                    if vecino == AIRE || bodega.material(vecino).deja_pasar_luz() {
                        salida += Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32);
                        abiertas += 1;
                    }
                }
                if abiertas == 0 {
                    continue;
                }
                let mut posicion = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                // si está medio enterrado (lava, lámpara de techo) la luz se asoma por el lado abierto
                if abiertas <= 2 {
                    posicion += salida.unitario() * 0.55;
                }

                let cercano = grupos
                    .iter_mut()
                    .find(|g| (g.suma / g.cuantos - posicion).largo() < RADIO_DE_GRUPO && g.color == material.resplandor);
                match cercano {
                    Some(g) => {
                        g.suma += posicion;
                        g.cuantos += 1.0;
                    }
                    None => grupos.push(Grupo { suma: posicion, color: material.resplandor, cuantos: 1.0 }),
                }
            }
        }
    }

    grupos
        .iter()
        .map(|g| {
            // un grupo grande alumbra más, pero no en proporción directa
            let fuerza = 3.0 * g.cuantos.sqrt().min(2.2);
            Farol { posicion: g.suma / g.cuantos, color: g.color * fuerza, alcance: 9.0 + g.cuantos.min(6.0) }
        })
        .collect()
}
