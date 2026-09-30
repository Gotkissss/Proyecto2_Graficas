use crate::algebra::{Tinte, Vec3, escalon_suave};
use crate::luces::LuzDelCielo;
use crate::texturas::Textura;

const CARAS: [&str; 6] = ["px", "nx", "py", "ny", "pz", "nz"];

/// Skybox: dos cubemaps (día y noche) que se mezclan según la hora,
/// con el sol y la luna dibujados encima en la dirección real de la luz.
pub struct Boveda {
    dia: Vec<Textura>,
    noche: Vec<Textura>,
}

/// A qué cara del cubo apunta la dirección y en qué parte de esa cara cae.
fn cara_y_uv(d: Vec3) -> (usize, f32, f32) {
    let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
    let (cara, a, b) = if ax >= ay && ax >= az {
        if d.x > 0.0 { (0, -d.z / ax, -d.y / ax) } else { (1, d.z / ax, -d.y / ax) }
    } else if ay >= az {
        if d.y > 0.0 { (2, d.x / ay, d.z / ay) } else { (3, d.x / ay, -d.z / ay) }
    } else if d.z > 0.0 {
        (4, d.x / az, -d.y / az)
    } else {
        (5, -d.x / az, -d.y / az)
    };
    (cara, a * 0.5 + 0.5, b * 0.5 + 0.5)
}

impl Boveda {
    pub fn abrir() -> Boveda {
        let juego = |prefijo: &str| -> Vec<Textura> {
            CARAS
                .iter()
                .map(|c| Textura::abrir(&format!("assets/cielo/{prefijo}_{c}.png"), true))
                .collect()
        };
        Boveda { dia: juego("dia"), noche: juego("noche") }
    }

    pub fn mirar(&self, dir: Vec3, luz: &LuzDelCielo) -> Tinte {
        let (cara, u, v) = cara_y_uv(dir);

        let mut color = Tinte::CERO;
        if luz.claridad > 0.01 {
            color += self.dia[cara].suave(u, v) * luz.claridad;
        }
        if luz.claridad < 0.99 {
            color += self.noche[cara].suave(u, v) * (1.0 - luz.claridad);
        }

        // franja naranja del amanecer/atardecer, más fuerte del lado donde está el sol
        let sol_bajo = 1.0 - escalon_suave(0.0, 0.4, luz.sol.y.abs());
        if sol_bajo > 0.0 {
            let cerca_del_horizonte = (-dir.y.abs() * 5.5).exp();
            let hacia_el_sol = 0.35 + 0.65 * dir.punto(luz.sol).max(0.0);
            let fuerza = sol_bajo * cerca_del_horizonte * hacia_el_sol;
            color = color.mezclar(Tinte::new(1.0, 0.42, 0.14), (fuerza * 0.7).min(1.0));
        }

        let alineado = dir.punto(luz.sol);
        if luz.sol.y > -0.12 && alineado > 0.0 {
            let disco = escalon_suave(0.9991, 0.9995, alineado);
            let halo = alineado.powf(350.0) * 0.5 + alineado.powf(24.0) * 0.12;
            color += Tinte::new(1.0, 0.85, 0.6) * (halo * luz.claridad.max(0.3)) + Tinte::new(3.0, 2.7, 2.1) * disco;
        } else if luz.sol.y < 0.12 && alineado < 0.0 {
            let disco = escalon_suave(0.9993, 0.9996, -alineado);
            let halo = (-alineado).powf(500.0) * 0.15;
            color += Tinte::new(0.85, 0.9, 1.0) * (disco + halo) * (1.0 - luz.claridad);
        }

        color
    }
}
