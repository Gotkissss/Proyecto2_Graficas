use crate::algebra::Tinte;
use crate::texturas::Textura;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bloque {
    Aire = 0,
    Pasto,
    Tierra,
    Piedra,
    Adoquin,
    Arena,
    Agua,
    Vidrio,
    Tronco,
    Hojas,
    Tablones,
    Ladrillo,
    PiedraLuz,
    Lava,
    Obsidiana,
    Oro,
}

impl Bloque {
    pub const fn id(self) -> u8 {
        self as u8
    }
}

/// Cómo responde un tipo de bloque a la luz.
pub struct Material {
    // índices dentro de Bodega::texturas
    pub arriba: usize,
    pub lado: usize,
    pub abajo: usize,
    pub relieve: Option<usize>,

    /// qué tanto de la luz difusa devuelve
    pub albedo: f32,
    /// intensidad del brillo especular
    pub especular: f32,
    /// exponente del brillo: más alto = punto de luz más chico
    pub pulido: f32,
    pub reflejo: f32,
    /// los metales tiñen lo que reflejan con su propio color
    pub metalico: bool,
    pub transparencia: f32,
    pub indice_refraccion: f32,
    /// cuánto se come de cada canal por unidad de distancia dentro del bloque
    pub absorcion: Tinte,
    /// > 0 si el bloque brilla solo
    pub emision: f32,
    /// color de la luz que tira (solo importa si emite)
    pub resplandor: Tinte,
    /// la textura tiene huecos por donde pasa el rayo (hojas)
    pub calado: bool,
}

impl Material {
    pub fn deja_pasar_luz(&self) -> bool {
        self.transparencia > 0.0
    }

    /// Solo los bloques macizos oscurecen las esquinas de sus vecinos.
    pub fn hace_sombra_suave(&self) -> bool {
        !self.deja_pasar_luz() && !self.calado && self.emision == 0.0
    }
}

/// Todas las texturas y los materiales, indexados por id de bloque.
pub struct Bodega {
    pub texturas: Vec<Textura>,
    materiales: Vec<Material>,
}

/// Va cargando texturas sin repetir archivos.
struct Estante {
    nombres: Vec<String>,
    texturas: Vec<Textura>,
}

impl Estante {
    fn traer(&mut self, nombre: &str, es_color: bool) -> usize {
        if let Some(i) = self.nombres.iter().position(|n| n == nombre) {
            return i;
        }
        let ruta = format!("assets/texturas/{nombre}.png");
        self.texturas.push(Textura::abrir(&ruta, es_color));
        self.nombres.push(nombre.to_string());
        self.texturas.len() - 1
    }

    /// Material opaco y mate con la misma textura en todas las caras.
    /// De aquí se parte y cada bloque cambia lo que necesita.
    fn basico(&mut self, nombre: &str) -> Material {
        let t = self.traer(nombre, true);
        Material {
            arriba: t,
            lado: t,
            abajo: t,
            relieve: None,
            albedo: 0.9,
            especular: 0.05,
            pulido: 10.0,
            reflejo: 0.0,
            metalico: false,
            transparencia: 0.0,
            indice_refraccion: 1.0,
            absorcion: Tinte::CERO,
            emision: 0.0,
            resplandor: Tinte::CERO,
            calado: false,
        }
    }
}

impl Bodega {
    pub fn surtir() -> Bodega {
        let mut e = Estante { nombres: Vec::new(), texturas: Vec::new() };

        // el orden tiene que ser el mismo del enum Bloque
        let mut materiales = Vec::new();

        // Aire: nunca se dibuja, pero así el índice coincide con el id
        materiales.push(e.basico("tierra"));

        let mut pasto = e.basico("pasto_lado");
        pasto.arriba = e.traer("pasto_arriba", true);
        pasto.abajo = e.traer("tierra", true);
        pasto.albedo = 0.95;
        pasto.especular = 0.03;
        materiales.push(pasto);

        let mut tierra = e.basico("tierra");
        tierra.especular = 0.02;
        tierra.pulido = 6.0;
        materiales.push(tierra);

        let mut piedra = e.basico("piedra");
        piedra.relieve = Some(e.traer("piedra_n", false));
        piedra.albedo = 0.85;
        piedra.especular = 0.14;
        piedra.pulido = 22.0;
        materiales.push(piedra);

        let mut adoquin = e.basico("adoquin");
        adoquin.relieve = Some(e.traer("adoquin_n", false));
        adoquin.albedo = 0.85;
        adoquin.especular = 0.2;
        adoquin.pulido = 28.0;
        materiales.push(adoquin);

        let mut arena = e.basico("arena");
        arena.albedo = 0.95;
        arena.especular = 0.07;
        materiales.push(arena);

        let mut agua = e.basico("agua");
        agua.albedo = 0.5;
        agua.especular = 0.9;
        agua.pulido = 160.0;
        agua.reflejo = 0.1;
        agua.transparencia = 0.9;
        agua.indice_refraccion = 1.33;
        agua.absorcion = Tinte::new(0.34, 0.11, 0.05);
        materiales.push(agua);

        let mut vidrio = e.basico("vidrio");
        vidrio.albedo = 0.9;
        vidrio.especular = 1.0;
        vidrio.pulido = 220.0;
        vidrio.reflejo = 0.08;
        vidrio.transparencia = 0.95;
        vidrio.indice_refraccion = 1.5;
        vidrio.absorcion = Tinte::new(0.06, 0.02, 0.03);
        materiales.push(vidrio);

        let mut tronco = e.basico("tronco_lado");
        tronco.arriba = e.traer("tronco_arriba", true);
        tronco.abajo = tronco.arriba;
        tronco.albedo = 0.85;
        tronco.especular = 0.04;
        materiales.push(tronco);

        let mut hojas = e.basico("hojas");
        hojas.especular = 0.08;
        hojas.pulido = 14.0;
        hojas.calado = true;
        materiales.push(hojas);

        let mut tablones = e.basico("tablones");
        tablones.relieve = Some(e.traer("tablones_n", false));
        tablones.especular = 0.1;
        tablones.pulido = 18.0;
        materiales.push(tablones);

        let mut ladrillo = e.basico("ladrillo");
        ladrillo.relieve = Some(e.traer("ladrillo_n", false));
        ladrillo.especular = 0.08;
        ladrillo.pulido = 14.0;
        materiales.push(ladrillo);

        let mut piedra_luz = e.basico("piedra_luz");
        piedra_luz.emision = 3.2;
        piedra_luz.resplandor = Tinte::new(1.0, 0.76, 0.42);
        materiales.push(piedra_luz);

        let mut lava = e.basico("lava");
        lava.emision = 2.6;
        lava.resplandor = Tinte::new(1.0, 0.42, 0.12);
        materiales.push(lava);

        let mut obsidiana = e.basico("obsidiana");
        obsidiana.albedo = 0.7;
        obsidiana.especular = 0.8;
        obsidiana.pulido = 90.0;
        obsidiana.reflejo = 0.3;
        materiales.push(obsidiana);

        let mut oro = e.basico("oro");
        oro.albedo = 0.55;
        oro.especular = 1.0;
        oro.pulido = 120.0;
        oro.reflejo = 0.55;
        oro.metalico = true;
        materiales.push(oro);

        Bodega { texturas: e.texturas, materiales }
    }

    #[inline]
    pub fn material(&self, id: u8) -> &Material {
        &self.materiales[id as usize]
    }
}
