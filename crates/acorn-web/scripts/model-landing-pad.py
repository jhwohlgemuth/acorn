"""Original generic flight-operations landing mat with modeled markings. MIT."""
import math


def create(m):
    """Return a reinforced landing mat, with no display pedestal or aircraft mark."""
    parts = [
        m.L((0, 0, 0), [(0, 1.43), (0.008, 1.49), (0.023, 1.50), (0.032, 1.49), (0.036, 1.43)], "dark", 128),
        m.C((0, 0.038, 0), 1.37, 0.020, "teal", 128),
        paint_ring(m, 1.095, 1.145, 0.0495, "white"),
        paint_ring(m, 0.200, 0.222, 0.0495, "white"),
        m.C((0, 0.049, 0), 0.042, 0.003, "ivory", 32),
    ]
    for index in range(16):
        angle = index * math.pi / 8
        parts.append(m.B((1.283 * math.cos(angle), 0.049, 1.283 * math.sin(angle)),
            (0.116 if index % 4 == 0 else 0.072, 0.002, 0.040 if index % 4 == 0 else 0.023), "ivory", (0, -angle, 0)))
    for index in range(4):
        angle = index * math.pi / 2
        chevron = [(0.64, 0), (0.83, -0.14), (0.86, -0.10), (0.72, 0), (0.86, 0.10), (0.83, 0.14)]
        points = [(x * math.cos(angle) - z * math.sin(angle), x * math.sin(angle) + z * math.cos(angle)) for x, z in chevron]
        parts.append(m.E(points, 0.002, "white", (0, 0.049, 0), (math.pi / 2, 0, 0)))
        parts.append(m.B((0.36 * math.cos(angle), 0.049, 0.36 * math.sin(angle)), (0.12, 0.002, 0.022), "white", (0, -angle, 0)))
    for index in range(8):
        angle = (index + 0.5) * math.pi / 4
        center = (1.425 * math.cos(angle), 0.048, 1.425 * math.sin(angle))
        parts.append(m.part(m.G.torus(0.030, 0.006, sides=24, segments=8), "steel", center, (1, 0.62, 1), smooth=True))
        parts.append(m.C((center[0], 0.047, center[2]), 0.023, 0.002, "dark", 24))
    parts.append(paint_ring(m, 1.465, 1.468, 0.044, "ivory"))
    return parts


def paint_ring(m, inner, outer, height, material):
    """Return a flat annular marking with upward-facing triangles."""
    sides = 128
    vertices = [(radius * math.cos(index * 2 * math.pi / sides), height, radius * math.sin(index * 2 * math.pi / sides))
        for index in range(sides) for radius in [inner, outer]]
    faces = [face for index in range(sides) for following in [(index + 1) % sides]
        for face in [(2 * index, 2 * following, 2 * following + 1), (2 * index, 2 * following + 1, 2 * index + 1)]]
    return m.part((vertices, faces), material)
