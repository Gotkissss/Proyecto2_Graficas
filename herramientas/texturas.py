# Genera las texturas de 16x16 de los bloques y sus mapas normales.
# Uso: python herramientas/texturas.py assets/texturas
import zlib, struct, random, math, os, sys

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
N = 16


def save(name, px, w=N, h=N):
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w):
            p = px[y][x]
            if len(p) == 3:
                p = (*p, 1.0)
            raw.extend(max(0, min(255, int(round(c * 255)))) for c in p)

    def chunk(t, d):
        c = struct.pack(">I", len(d)) + t + d
        return c + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")
    with open(os.path.join(OUT, name + ".png"), "wb") as f:
        f.write(png)


def grid(fn):
    return [[fn(x, y) for x in range(N)] for y in range(N)]


def mul(c, k):
    return tuple(v * k for v in c)


def mix(a, b, t):
    return tuple(a[i] + (b[i] - a[i]) * t for i in range(3))


def hexc(s):
    return tuple(int(s[i:i + 2], 16) / 255 for i in (0, 2, 4))


def blotch(rng, levels=4, scale=4):
    """ruido de baja frecuencia tileable, cuantizado"""
    g = [[rng.random() for _ in range(scale)] for _ in range(scale)]

    def f(x, y):
        fx, fy = x * scale / N, y * scale / N
        x0, y0 = int(fx) % scale, int(fy) % scale
        x1, y1 = (x0 + 1) % scale, (y0 + 1) % scale
        tx, ty = fx - int(fx), fy - int(fy)
        a = g[y0][x0] + (g[y0][x1] - g[y0][x0]) * tx
        b = g[y1][x0] + (g[y1][x1] - g[y1][x0]) * tx
        return a + (b - a) * ty

    return f


def normal_from_height(h, strength=2.0):
    def fn(x, y):
        l = h[y][(x - 1) % N]; r = h[y][(x + 1) % N]
        u = h[(y - 1) % N][x]; d = h[(y + 1) % N][x]
        nx, ny, nz = (l - r) * strength, (u - d) * strength, 1.0
        m = math.sqrt(nx * nx + ny * ny + nz * nz)
        return (nx / m * .5 + .5, ny / m * .5 + .5, nz / m * .5 + .5)
    return grid(fn)


def pick(rng, pal, weights=None):
    return rng.choices(pal, weights)[0]


# ---------- tierra / pasto
rng = random.Random(11)
dirt_pal = [hexc("866043"), hexc("79553a"), hexc("966c4a"), hexc("6b4a31"), hexc("a07a58")]
dirt = grid(lambda x, y: pick(rng, dirt_pal, [5, 4, 3, 2, 1]))
for _ in range(5):
    dirt[rng.randrange(N)][rng.randrange(N)] = hexc("8d8d8d")
save("tierra", dirt)

rng = random.Random(12)
g_pal = [hexc("5d9b3a"), hexc("528c33"), hexc("6aac43"), hexc("477a2c"), hexc("78ba4f")]
grass = grid(lambda x, y: pick(rng, g_pal, [5, 4, 3, 2, 1]))
save("pasto_arriba", grass)

rng = random.Random(13)
edge = [rng.choice([2, 3, 3, 4, 4, 5]) for _ in range(N)]
side = [[(grass[y][x] if y < edge[x] else (mul(dirt[y][x], 0.8) if y == edge[x] else dirt[y][x])) for x in range(N)] for y in range(N)]
save("pasto_lado", side)

# ---------- piedra
rng = random.Random(21)
b1 = blotch(rng, scale=5); b2 = blotch(rng, scale=8)
st_pal = [hexc("6f6f6f"), hexc("7d7d7d"), hexc("888888"), hexc("949494")]
sh = grid(lambda x, y: b1(x, y) * .65 + b2(x, y) * .35)
stone = grid(lambda x, y: st_pal[min(3, int(sh[y][x] * 4.6 - .3) if sh[y][x] > .07 else 0)])
save("piedra", stone)
save("piedra_n", normal_from_height(sh, 2.2))

# ---------- adoquin (voronoi tileable)
rng = random.Random(31)
pts = [(rng.uniform(0, N), rng.uniform(0, N), rng.uniform(.75, 1.05)) for _ in range(11)]


def voro(x, y):
    ds = []
    for px_, py_, tone in pts:
        dx = abs(x + .5 - px_); dx = min(dx, N - dx)
        dy = abs(y + .5 - py_); dy = min(dy, N - dy)
        ds.append((math.hypot(dx, dy), tone))
    ds.sort()
    return ds[0][0], ds[1][0] - ds[0][0], ds[0][1]


ch = [[0.0] * N for _ in range(N)]
cob = [[None] * N for _ in range(N)]
for y in range(N):
    for x in range(N):
        d, gap, tone = voro(x, y)
        if gap < 0.9:
            ch[y][x] = 0.0
            cob[y][x] = mul(hexc("4a4a4a"), rng.uniform(.9, 1.05))
        else:
            ch[y][x] = min(1.0, .55 + gap * .2)
            cob[y][x] = mul(hexc("8a8a8a"), tone * rng.uniform(.93, 1.05) * (1.08 if d < 1.2 else 1))
save("adoquin", cob)
save("adoquin_n", normal_from_height(ch, 3.0))

# ---------- arena
rng = random.Random(41)
s_pal = [hexc("dccf9c"), hexc("d3c590"), hexc("e6dbac"), hexc("c9ba84")]
save("arena", grid(lambda x, y: pick(rng, s_pal, [6, 4, 3, 1])))

# ---------- agua
rng = random.Random(51)
wb = blotch(rng, scale=4)
w_pal = [hexc("2a5fc4"), hexc("3068d0"), hexc("3a75dc"), hexc("4c88e8")]
save("agua", grid(lambda x, y: (*w_pal[min(3, int(((wb(x, y) + .25 * math.sin((x + y * 2) * math.pi / 4)) * .9 + .1) * 4) % 4)], .25)))

# ---------- vidrio (alpha)
rng = random.Random(61)


def glass(x, y):
    if x in (0, N - 1) or y in (0, N - 1):
        return (*mul(hexc("d8eef2"), rng.uniform(.9, 1.0)), 1.0)
    if (x + y) in (7, 8) and 3 <= x <= 6:
        return (.95, 1, 1, .75)
    if (x + y) in (21, 22) and 9 <= x <= 12:
        return (.95, 1, 1, .6)
    return (.78, .93, .96, 0.0)


save("vidrio", grid(glass))

# ---------- tronco
rng = random.Random(71)
cols = [rng.choice([.78, .88, .95, 1.0, 1.08]) for _ in range(N)]
bark = grid(lambda x, y: mul(hexc("6b5232"), cols[x] * rng.uniform(.92, 1.06) * (.8 if rng.random() < .08 else 1)))
save("tronco_lado", bark)


def ring(x, y):
    d = max(abs(x - 7.5), abs(y - 7.5))
    if d > 7:
        return mul(hexc("6b5232"), rng.uniform(.85, 1.05))
    return mul(hexc("b8945f") if int(d) % 2 == 0 else hexc("9a7a4a"), rng.uniform(.95, 1.04))


save("tronco_arriba", grid(ring))

# ---------- hojas (alpha calado)
rng = random.Random(81)
l_pal = [hexc("3f7d2a"), hexc("37702a"), hexc("4a8f33"), hexc("2d5d20"), hexc("58a03e")]


def leaf(x, y):
    if rng.random() < .17:
        return (0.1, 0.25, 0.08, 0.0)
    return (*pick(rng, l_pal, [5, 4, 3, 2, 1]), 1.0)


save("hojas", grid(leaf))

# ---------- tablones
rng = random.Random(91)
seams = {0: 5, 1: 12, 2: 2, 3: 9}
ph = [[0.0] * N for _ in range(N)]


def plank(x, y):
    row = y // 4
    base = hexc("b08a55")
    tone = [1.0, .93, 1.05, .97][row]
    if y % 4 == 3 or x == seams[row]:
        ph[y][x] = 0.0
        return mul(hexc("6e5430"), rng.uniform(.95, 1.05))
    ph[y][x] = 1.0
    grain = .94 if (x * 3 + row * 5 + (y % 4)) % 7 == 0 else 1.0
    return mul(base, tone * grain * rng.uniform(.96, 1.04))


save("tablones", grid(plank))
save("tablones_n", normal_from_height(ph, 1.6))

# ---------- ladrillo
rng = random.Random(101)
bh = [[0.0] * N for _ in range(N)]
tones = {}


def brick(x, y):
    row = y // 4
    off = 0 if row % 2 == 0 else 4
    if y % 4 == 3 or (x + off) % 8 == 7:
        bh[y][x] = 0.0
        return mul(hexc("b9b2a3"), rng.uniform(.9, 1.03))
    key = (row, ((x + off) % N) // 8)
    tones.setdefault(key, rng.uniform(.85, 1.1))
    bh[y][x] = 1.0
    return mul(hexc("a1503c"), tones[key] * rng.uniform(.94, 1.05))


save("ladrillo", grid(brick))
save("ladrillo_n", normal_from_height(bh, 3.0))

# ---------- piedra luminosa
rng = random.Random(111)
gb = blotch(rng, scale=6)
gl_pal = [hexc("8a5a1c"), hexc("c98f2e"), hexc("f0c14a"), hexc("ffe48a"), hexc("fff6c8")]
save("piedra_luz", grid(lambda x, y: gl_pal[min(4, int((gb(x, y) * .8 + rng.random() * .3) * 5))]))

# ---------- lava
rng = random.Random(121)
lb = blotch(rng, scale=4)
lv_pal = [hexc("b3260a"), hexc("d8430c"), hexc("f0700f"), hexc("fba21c"), hexc("ffd84a")]
save("lava", grid(lambda x, y: lv_pal[min(4, int((lb(x, y) * .85 + rng.random() * .2) * 5))]))

# ---------- obsidiana
rng = random.Random(131)
ob = blotch(rng, scale=5)
ob_pal = [hexc("120d1c"), hexc("1a1428"), hexc("251b38"), hexc("3a2b55")]
save("obsidiana", grid(lambda x, y: ob_pal[3] if rng.random() < .05 else ob_pal[min(2, int(ob(x, y) * 3))]))

# ---------- oro
rng = random.Random(141)


def gold(x, y):
    if x in (0, N - 1) or y in (0, N - 1):
        return mul(hexc("c99a1e"), rng.uniform(.92, 1.0))
    if (x - y) in (-1, 0, 5):
        return hexc("fff3a0")
    return mul(hexc("f2cc3a"), rng.uniform(.95, 1.04))


save("oro", grid(gold))

print("texturas ok")
