//! Todo lo que va encima del terreno: la cabaña, el altar y los árboles.
//! Los lugares salen de la semilla, así que cambian junto con la isla.

use crate::materiales::Bloque;
use crate::mundo::{AIRE, Mundo};
use crate::ruido::azar;
use crate::terreno::{LADO, NIVEL_AGUA, Relieve};

pub fn poblar(mundo: &mut Mundo, relieve: &mut Relieve, semilla: u32) {
    levantar_cabana(mundo, relieve);
    alzar_altar(mundo, relieve);
    sembrar_arboles(mundo, relieve, semilla);
}

/// Punto a cierta distancia del centro, en un ángulo relativo al del volcán.
fn sitio(relieve: &Relieve, desvio: f32, radio: f32) -> (i32, i32) {
    let centro = LADO as f32 / 2.0;
    let angulo = relieve.rumbo + desvio;
    ((centro + angulo.cos() * radio) as i32, (centro + angulo.sin() * radio) as i32)
}

/// Rellena de adoquín desde el suelo hasta `nivel` para que nada quede flotando.
fn cimentar(mundo: &mut Mundo, relieve: &mut Relieve, x: i32, z: i32, nivel: i32) {
    let suelo = relieve.altura(x, z).max(nivel - 5);
    for y in (suelo + 1)..=nivel {
        mundo.poner(x, y, z, Bloque::Adoquin.id());
    }
    relieve.fijar(x, z, nivel);
    relieve.ocupar(x, z);
}

fn levantar_cabana(mundo: &mut Mundo, relieve: &mut Relieve) {
    const ANCHO: i32 = 7;
    const FONDO: i32 = 6;

    let (cx, cz) = sitio(relieve, 2.1, 8.0);
    let (x0, z0) = (cx - ANCHO / 2, cz - FONDO / 2);

    // se construye al nivel del punto más alto del terreno que pisa
    let mut piso = 0;
    for z in z0..z0 + FONDO {
        for x in x0..x0 + ANCHO {
            piso = piso.max(relieve.altura(x, z));
        }
    }

    for z in (z0 - 1)..=(z0 + FONDO) {
        for x in (x0 - 1)..=(x0 + ANCHO) {
            relieve.ocupar(x, z);
        }
    }

    // la puerta mira hacia el centro de la isla
    let puerta_al_frente = cz < LADO / 2;
    let z_puerta = if puerta_al_frente { z0 + FONDO - 1 } else { z0 };
    let x_puerta = x0 + ANCHO / 2;

    for z in z0..z0 + FONDO {
        for x in x0..x0 + ANCHO {
            cimentar(mundo, relieve, x, z, piso);
            mundo.poner(x, piso, z, Bloque::Tablones.id());

            let pared_x = x == x0 || x == x0 + ANCHO - 1;
            let pared_z = z == z0 || z == z0 + FONDO - 1;
            if !pared_x && !pared_z {
                continue;
            }
            for y in (piso + 1)..=(piso + 3) {
                let ventana = y == piso + 2
                    && ((pared_z && !pared_x && (x - x0) % 2 == 1 && x != x_puerta)
                        || (pared_x && !pared_z && (z == z0 + 2 || z == z0 + 3)));
                let bloque = if pared_x && pared_z {
                    Bloque::Tronco
                } else if ventana {
                    Bloque::Vidrio
                } else {
                    Bloque::Tablones
                };
                mundo.poner(x, y, z, bloque.id());
            }
        }
    }
    mundo.poner(x_puerta, piso + 1, z_puerta, AIRE);
    mundo.poner(x_puerta, piso + 2, z_puerta, AIRE);

    // techo a dos aguas: cada capa se mete un bloque
    for capa in 0..4 {
        for z in (z0 - 1 + capa)..=(z0 + FONDO - capa) {
            for x in (x0 - 1)..=(x0 + ANCHO) {
                mundo.poner(x, piso + 4 + capa, z, Bloque::Ladrillo.id());
            }
        }
    }
    for y in (piso + 4)..=(piso + 8) {
        mundo.poner(x0 + ANCHO - 2, y, z0 + 1, Bloque::Adoquin.id());
    }

    // lámpara colgada del techo, de noche la luz sale por las ventanas
    mundo.poner(x_puerta, piso + 3, z0 + FONDO / 2, Bloque::PiedraLuz.id());

    // caminito de adoquín saliendo de la puerta y un farol a la par
    let avance = if puerta_al_frente { 1 } else { -1 };
    for i in 1..=6 {
        let z = z_puerta + avance * i;
        let y = relieve.altura(x_puerta, z);
        if y >= NIVEL_AGUA && mundo.bloque(x_puerta, y, z) == Bloque::Pasto.id() {
            mundo.poner(x_puerta, y, z, Bloque::Adoquin.id());
            relieve.ocupar(x_puerta, z);
        }
    }
    let (fx, fz) = (x_puerta + 2, z_puerta + avance * 3);
    let base = relieve.altura(fx, fz);
    if base >= NIVEL_AGUA {
        mundo.poner(fx, base + 1, fz, Bloque::Tronco.id());
        mundo.poner(fx, base + 2, fz, Bloque::Tronco.id());
        mundo.poner(fx, base + 3, fz, Bloque::PiedraLuz.id());
        relieve.ocupar(fx, fz);
    }
}

/// Plataforma de adoquín con pilares de obsidiana rematados en oro, cerca del lago.
fn alzar_altar(mundo: &mut Mundo, relieve: &mut Relieve) {
    let (cx, cz) = sitio(relieve, -1.6, 9.5);
    let nivel = relieve.altura(cx, cz).max(NIVEL_AGUA);

    for dz in -2..=2_i32 {
        for dx in -2..=2_i32 {
            let (x, z) = (cx + dx, cz + dz);
            // quitar lo que sobresalga (tierra o agua) antes de poner la plataforma
            for y in (nivel + 1)..(nivel + 6) {
                mundo.poner(x, y, z, AIRE);
            }
            cimentar(mundo, relieve, x, z, nivel);

            if dx.abs() == 2 && dz.abs() == 2 {
                for y in 1..=3 {
                    mundo.poner(x, nivel + y, z, Bloque::Obsidiana.id());
                }
                mundo.poner(x, nivel + 4, z, Bloque::Oro.id());
            }
        }
    }
    // en el centro una piedra luminosa, para que de noche se refleje en el oro y en el lago
    mundo.poner(cx, nivel + 1, cz, Bloque::Obsidiana.id());
    mundo.poner(cx, nivel + 2, cz, Bloque::PiedraLuz.id());
}

fn sembrar_arboles(mundo: &mut Mundo, relieve: &mut Relieve, semilla: u32) {
    for z in 2..LADO - 2 {
        for x in 2..LADO - 2 {
            if azar(x, z, semilla + 21) > 0.045 {
                continue;
            }
            let base = relieve.altura(x, z);
            if base < 0 || mundo.bloque(x, base, z) != Bloque::Pasto.id() {
                continue;
            }
            // que no queden pegados entre sí ni a las construcciones
            let despejado = (-2..=2).all(|dz| (-2..=2).all(|dx| relieve.libre(x + dx, z + dz)));
            if !despejado {
                continue;
            }

            let alto = 4 + (azar(z, x, semilla + 22) * 3.0) as i32;
            plantar_arbol(mundo, x, base, z, alto, semilla);
            for dz in -1..=1 {
                for dx in -1..=1 {
                    relieve.ocupar(x + dx, z + dz);
                }
            }
        }
    }
}

fn plantar_arbol(mundo: &mut Mundo, x: i32, base: i32, z: i32, alto: i32, semilla: u32) {
    let copa = base + alto;

    // dos capas anchas abajo y dos angostas arriba, como los robles de minecraft
    for dy in -2..=1_i32 {
        let radio: i32 = if dy < 0 { 2 } else { 1 };
        for dz in -radio..=radio {
            for dx in -radio..=radio {
                let esquina = dx.abs() == radio && dz.abs() == radio;
                if esquina && (dy == 1 || azar(x + dx + dy * 7, z + dz, semilla + 23) < 0.55) {
                    continue;
                }
                if mundo.bloque(x + dx, copa + dy, z + dz) == AIRE {
                    mundo.poner(x + dx, copa + dy, z + dz, Bloque::Hojas.id());
                }
            }
        }
    }
    for y in (base + 1)..copa {
        mundo.poner(x, y, z, Bloque::Tronco.id());
    }
}
