use crate::algebra::{Tinte, Vec3, escalon_suave};

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
        let claridad = escalon_suave(-0.12, 0.2, sol.y);

        let de_dia = sol.y > 0.0;
        let hacia = if de_dia { sol } else { -sol };

        // cerca del horizonte la luz se apaga para que el cambio sol/luna no pegue un salto
        let fuerza = escalon_suave(0.0, 0.16, sol.y.abs());
        let color = if de_dia {
            let atardecer = Tinte::new(1.0, 0.5, 0.22);
            let mediodia = Tinte::new(1.0, 0.96, 0.88);
            atardecer.mezclar(mediodia, escalon_suave(0.0, 0.45, sol.y)) * (1.25 * fuerza)
        } else {
            Tinte::new(0.16, 0.22, 0.4) * (0.55 * fuerza)
        };

        let ambiente = Tinte::new(0.018, 0.024, 0.05).mezclar(Tinte::new(0.30, 0.38, 0.52), claridad);

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
