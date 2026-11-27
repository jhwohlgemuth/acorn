"""Original meter-scale campus vegetation, parked cars and lighting. MIT."""
import math
import random


def _block(api, center, size, material):
    mesh = api.G.pyramid(*size, top=1)
    return api.part(mesh, material, (center[0], center[1] - size[1] / 2, center[2]))


def _loft(api, rings, material):
    vertices, faces = api.G.ring_mesh(rings)
    return api.part((vertices, [(c, b, a) for a, b, c in faces]), material)


def _place(api, parts, x, z, angle):
    return [dict(part, vertices=api.G.transform((part["vertices"], part["triangles"]),
        center=(x, 0, z), rotation=(0, angle, 0))[0]) for part in parts]


def car(api, x, z, angle=0, color="vehicle-silver"):
    """Return a 4.6 m sedan facing +Z; angle rotates around the ground pivot."""
    sections = [(-2.30, 0.70, 0.41, 0.70), (-2.12, 0.86, 0.30, 0.84),
        (-1.30, 0.90, 0.29, 0.98), (1.28, 0.90, 0.29, 0.93),
        (2.12, 0.84, 0.33, 0.78), (2.30, 0.69, 0.41, 0.69)]
    rings = [[(px, py, depth) for px, py in [(-width, low + 0.07), (-width + 0.07, low),
        (width - 0.07, low), (width, low + 0.07), (width, high - 0.07),
        (width - 0.07, high), (-width + 0.07, high), (-width, high - 0.07)]]
        for depth, width, low, high in sections]
    cabin = [[(-lower, 0.91, depth), (lower, 0.91, depth), (upper, top, depth), (-upper, top, depth)]
        for depth, lower, upper, top in [(-1.36, 0.77, 0.72, 1.02), (-0.83, 0.77, 0.64, 1.46),
            (0.60, 0.77, 0.64, 1.46), (1.17, 0.77, 0.72, 1.02)]]
    parts = [_loft(api, rings, color), _loft(api, cabin, "glass-dark"),
        _block(api, (0, 1.475, -0.115), (1.30, 0.05, 1.45), color),
        _block(api, (0, 0.53, 2.307), (0.68, 0.15, 0.014), "glass-dark")]
    for side in [-1, 1]:
        parts.append(_block(api, (side * 0.733, 1.16, -0.12), (0.07, 0.50, 0.09), color))
        parts.append(_block(api, (side * 0.49, 0.67, 2.27), (0.34, 0.12, 0.06), "lamp-white"))
        for depth in [-1.43, 1.40]:
            parts.append(api.part(api.G.lathe([(-0.11, 0.30), (0.11, 0.30)], 12),
                "tire", (side * 0.79, 0.30, depth), rotation=(0, 0, math.pi / 2)))
            parts.append(api.part(api.G.lathe([(-0.012, 0.17), (0.012, 0.17)], 8),
                "metal", (side * 0.905, 0.30, depth), rotation=(0, 0, math.pi / 2)))
    return _place(api, parts, x, z, angle)


def lamp(api, x, z, angle=0):
    """Return a slim 5 m pole with a forward-cantilevered LED luminaire."""
    parts = [api.part(api.G.lathe([(0, 0.13), (0.12, 0.13), (0.19, 0.065), (4.86, 0.045)], 8), "metal"),
        api.part(api.G.beam((0, 4.82, 0), (0, 4.94, 0.64), 0.043, sides=8), "metal"),
        _block(api, (0, 4.965, 0.82), (0.28, 0.07, 0.63), "metal"),
        _block(api, (0, 4.925, 0.86), (0.20, 0.012, 0.48), "lamp-white")]
    return _place(api, parts, x, z, angle)


def tree(api, x, z, height, seed=0):
    """Return a seeded broadleaf tree; bark and two leaf shades total three materials."""
    rng = random.Random(seed)
    phase = rng.uniform(0, 2 * math.pi)
    lean = (rng.uniform(-0.035, 0.035), rng.uniform(-0.035, 0.035))
    levels = [(0, 0.025), (0.025, 0.020), (0.18, 0.016), (0.34, 0.012), (0.48, 0.008), (0.63, 0.003)]
    rings = [[((lean[0] * y + radius * math.cos(a) * (1 + 0.08 * math.sin(i * 3 + row))) * height,
        y * height, (lean[1] * y + radius * math.sin(a) * (1 + 0.08 * math.sin(i * 3 + row))) * height)
        for i, a in enumerate([phase + j * math.pi / 4 for j in range(8)])] for row, (y, radius) in enumerate(levels)]
    parts = [api.part(api.G.ring_mesh(rings), "bark", smooth=True)]
    for index in range(8):
        angle = phase + index * math.pi / 4 + rng.uniform(-0.18, 0.18)
        start_height = 0.29 + index % 3 * 0.045
        fork = (math.cos(angle) * 0.11 * height, (0.47 + rng.uniform(-0.025, 0.025)) * height,
            math.sin(angle) * 0.11 * height)
        tip = (math.cos(angle + 0.15) * 0.21 * height, (0.62 + rng.uniform(-0.04, 0.06)) * height,
            math.sin(angle + 0.15) * 0.21 * height)
        start = (lean[0] * start_height * height, start_height * height, lean[1] * start_height * height)
        parts += [api.part(api.G.beam(start, fork, 0.010 * height, sides=6, end_radius=0.006 * height), "bark", smooth=True),
            api.part(api.G.beam(fork, tip, 0.006 * height, sides=6, end_radius=0.0015 * height), "bark", smooth=True)]
    leaf_shades = ("foliage-dark", "foliage-mid") if seed % 2 == 0 else ("foliage-mid", "foliage-light")
    for row, count, radius, level, size in [(0, 8, 0.19, 0.58, 0.16), (1, 8, 0.16, 0.74, 0.17), (2, 4, 0.09, 0.87, 0.13), (3, 1, 0, 0.89, 0.11)]:
        for index in range(count):
            angle = phase + index * 2 * math.pi / count + row * 0.42 + rng.uniform(-0.16, 0.16)
            center = ((radius * math.cos(angle) + rng.uniform(-0.025, 0.025)) * height,
                (level + rng.uniform(-0.025, 0.025)) * height,
                (radius * math.sin(angle) + rng.uniform(-0.025, 0.025)) * height)
            base, faces = api.G.sphere(sides=8, rings=6)
            distortions = [1 if abs(point[1]) > 0.999 else rng.uniform(0.84, 1.15) for point in base]
            vertices = [tuple(v * distortion for v in point) for point, distortion in zip(base, distortions)]
            scale = (size * height * rng.uniform(0.90, 1.17), size * height * rng.uniform(0.86, 1.03), size * height * rng.uniform(0.87, 1.10))
            parts.append(api.part((vertices, faces), leaf_shades[(index + row) % 2], center, scale,
                rotation=(rng.uniform(-0.3, 0.3), angle, rng.uniform(-0.3, 0.3)), smooth=True))
    top = max(vertex[1] for part in parts for vertex in part["vertices"])
    fitted = [dict(part, vertices=[(vertex[0] + x, vertex[1] * height / top, vertex[2] + z) for vertex in part["vertices"]]) for part in parts]
    return fitted
