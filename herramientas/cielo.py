# Genera las 6 caras de los cubemaps de dia y de noche (tarda un par de minutos).
# Uso: python herramientas/cielo.py assets/cielo 512
import zlib, struct, math, os, sys

OUT = sys.argv[1]
S = int(sys.argv[2]) if len(sys.argv) > 2 else 512
os.makedirs(OUT, exist_ok=True)


def save(name, rows):
    raw = bytearray()
    for r in rows:
        raw.append(0)
        raw.extend(r)

    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    with open(os.path.join(OUT, name + ".png"), "wb") as f:
        f.write(png)


def h2(ix, iy, seed=0):
    n = (ix * 374761393 + iy * 668265263 + seed * 2147483647) & 0xFFFFFFFF
    n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((n ^ (n >> 16)) & 0xFFFFFF) / 0xFFFFFF


def vnoise(x, y, seed):
    ix, iy = math.floor(x), math.floor(y)
    fx, fy = x - ix, y - iy
    fx = fx * fx * (3 - 2 * fx); fy = fy * fy * (3 - 2 * fy)
    a = h2(ix, iy, seed); b = h2(ix + 1, iy, seed)
    c = h2(ix, iy + 1, seed); d = h2(ix + 1, iy + 1, seed)
    return a + (b - a) * fx + (c + (d - c) * fx - a - (b - a) * fx) * fy


def fbm(x, y, seed, octs=5):
    t, amp, tot = 0.0, 0.5, 0.0
    for i in range(octs):
        t += vnoise(x, y, seed + i) * amp
        tot += amp
        x, y, amp = x * 2.03 + 11.7, y * 2.03 - 5.3, amp * 0.5
    return t / tot


def lerp3(a, b, t):
    return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t)


def smooth(a, b, x):
    t = max(0.0, min(1.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def day(dx, dy, dz):
    zen, hor, low = (0.16, 0.40, 0.84), (0.70, 0.85, 0.98), (0.55, 0.70, 0.88)
    if dy >= 0:
        col = lerp3(hor, zen, dy ** 0.55)
        k = 1.0 / (dy + 0.12)
        n = fbm(dx * k * 1.3 + 3.1, dz * k * 1.3 - 1.7, 7)
        cover = smooth(0.50, 0.72, n) * smooth(0.02, 0.22, dy)
        shade = 0.82 + 0.18 * smooth(0.55, 0.8, fbm(dx * k * 2.6, dz * k * 2.6, 19, 3))
        col = lerp3(col, (shade, shade, min(1.0, shade + 0.03)), cover * 0.93)
    else:
        # abajo: mar de nubes difuso
        k = 1.0 / (-dy + 0.10)
        n = fbm(dx * k * 0.9 - 8.0, dz * k * 0.9 + 4.0, 31, 4)
        sea = lerp3((0.62, 0.74, 0.90), (0.93, 0.96, 1.0), smooth(0.35, 0.7, n))
        col = lerp3(hor, lerp3(sea, low, smooth(0.5, 1.0, -dy) * 0.4), smooth(0.0, 0.18, -dy))
    return col


def night(dx, dy, dz):
    zen, hor = (0.010, 0.014, 0.045), (0.05, 0.07, 0.15)
    col = lerp3(hor, zen, abs(dy) ** 0.5)
    # nebulosa tenue
    ax, ay, az = abs(dx), abs(dy), abs(dz)
    neb = fbm(dx * 2.2 + dy * 1.1 + 5, dz * 2.2 - dy * 0.7, 53, 4)
    band = math.exp(-((dx * 0.6 + dy * 0.7 - dz * 0.35) ** 2) * 9)
    g = smooth(0.45, 0.8, neb) * band
    col = (col[0] + 0.10 * g, col[1] + 0.07 * g, col[2] + 0.16 * g)
    # estrellas: celdas sobre la cara dominante del cubo
    m = max(ax, ay, az)
    if m == ax:
        u, v, f = dy / m, dz / m, 1 if dx > 0 else 2
    elif m == ay:
        u, v, f = dx / m, dz / m, 3 if dy > 0 else 4
    else:
        u, v, f = dx / m, dy / m, 5 if dz > 0 else 6
    G = 70
    cu, cv = math.floor(u * G), math.floor(v * G)
    r = h2(cu, cv, 100 + f)
    if r > 0.93:
        sx = cu + 0.2 + 0.6 * h2(cu, cv, 200 + f)
        sy = cv + 0.2 + 0.6 * h2(cu, cv, 300 + f)
        d = math.hypot(u * G - sx, v * G - sy)
        br = (r - 0.93) / 0.07
        glow = max(0.0, 1 - d / (0.10 + 0.16 * br)) ** 2 * (0.5 + 0.7 * br)
        tint = h2(cu, cv, 400 + f)
        col = (col[0] + glow * (0.9 + 0.1 * tint), col[1] + glow * 0.9, col[2] + glow * (1.0 - 0.15 * tint))
    return col


FACES = {
    "px": lambda a, b: (1, -b, -a),
    "nx": lambda a, b: (-1, -b, a),
    "py": lambda a, b: (a, 1, b),
    "ny": lambda a, b: (a, -1, -b),
    "pz": lambda a, b: (a, -b, 1),
    "nz": lambda a, b: (-a, -b, -1),
}

for pref, fn in (("dia", day), ("noche", night)):
    for name, mk in FACES.items():
        rows = []
        for j in range(S):
            b = (j + 0.5) / S * 2 - 1
            row = bytearray()
            for i in range(S):
                a = (i + 0.5) / S * 2 - 1
                x, y, z = mk(a, b)
                l = math.sqrt(x * x + y * y + z * z)
                c = fn(x / l, y / l, z / l)
                row.extend(max(0, min(255, int(v * 255 + 0.5))) for v in c)
            rows.append(row)
        save(f"{pref}_{name}", rows)
        print(pref, name, flush=True)
