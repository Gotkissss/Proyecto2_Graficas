use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

/// Vector de 3 floats. Lo uso tanto para posiciones/direcciones como para colores (r, g, b).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub type Tinte = Vec3;

impl Vec3 {
    pub const CERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const UNO: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    pub const ARRIBA: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn parejo(v: f32) -> Self {
        Vec3::new(v, v, v)
    }

    pub fn punto(self, otro: Vec3) -> f32 {
        self.x * otro.x + self.y * otro.y + self.z * otro.z
    }

    pub fn cruz(self, otro: Vec3) -> Vec3 {
        Vec3::new(
            self.y * otro.z - self.z * otro.y,
            self.z * otro.x - self.x * otro.z,
            self.x * otro.y - self.y * otro.x,
        )
    }

    pub fn largo(self) -> f32 {
        self.punto(self).sqrt()
    }

    pub fn unitario(self) -> Vec3 {
        let l = self.largo();
        if l > 0.0 { self / l } else { self }
    }

    /// Interpolación lineal: t = 0 devuelve self, t = 1 devuelve `otro`.
    pub fn mezclar(self, otro: Vec3, t: f32) -> Vec3 {
        self + (otro - self) * t
    }

    pub fn componente(self, eje: usize) -> f32 {
        match eje {
            0 => self.x,
            1 => self.y,
            _ => self.z,
        }
    }

    pub fn mayor(self) -> f32 {
        self.x.max(self.y).max(self.z)
    }

    /// Refleja el vector (incidente) respecto a la normal.
    pub fn rebotar(self, normal: Vec3) -> Vec3 {
        self - normal * (2.0 * self.punto(normal))
    }

    /// Ley de Snell. `razon` = n1 / n2. Devuelve None si hay reflexión interna total.
    pub fn doblar(self, normal: Vec3, razon: f32) -> Option<Vec3> {
        let cos_i = -self.punto(normal);
        let sen2_t = razon * razon * (1.0 - cos_i * cos_i);
        if sen2_t > 1.0 {
            return None;
        }
        let cos_t = (1.0 - sen2_t).sqrt();
        Some(self * razon + normal * (razon * cos_i - cos_t))
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, o: Vec3) {
        *self = *self + o;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, k: f32) -> Vec3 {
        Vec3::new(self.x * k, self.y * k, self.z * k)
    }
}

// componente a componente, sirve para teñir colores
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, k: f32) -> Vec3 {
        self * (1.0 / k)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

pub fn escalon_suave(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
