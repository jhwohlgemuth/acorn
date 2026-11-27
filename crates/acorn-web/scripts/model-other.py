"""Original clean tabletop components with modeled dice markings. MIT."""
import itertools
import math

DIGITS = {
    "0": "abcedf", "1": "bc", "2": "abged", "3": "abgcd", "4": "fgbc",
    "5": "afgcd", "6": "afgecd", "7": "abc", "8": "abcdefg", "9": "abfgcd",
}
FINISHES = {}
SEGMENTS = {
    "a": ((-0.34, 0.5), (0.34, 0.5)), "b": ((0.4, 0.43), (0.4, 0.07)),
    "c": ((0.4, -0.07), (0.4, -0.43)), "d": ((-0.34, -0.5), (0.34, -0.5)),
    "e": ((-0.4, -0.43), (-0.4, -0.07)), "f": ((-0.4, 0.07), (-0.4, 0.43)),
    "g": ((-0.34, 0), (0.34, 0)),
}


def beveled_outline(m, outline, depth, material="teal", bevel=0.035):
    """Extrude a simple outline with a narrow, continuous edge bevel."""
    outline = list(outline)
    if sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(outline, outline[1:] + outline[:1])) < 0:
        outline.reverse()
    inset = []
    for i, point in enumerate(outline):
        a = m.G.unit(sub(point, outline[i - 1]))
        b = m.G.unit(sub(outline[(i + 1) % len(outline)], point))
        normal = (-a[1] - b[1], a[0] + b[0])
        amount = bevel / max(0.15, dot(normal, (-a[1], a[0])))
        inset.append((point[0] + normal[0] * amount, point[1] + normal[1] * amount))
    count = len(outline)
    rings = [[(x, y, z) for x, y in points] for z, points in
        [(-depth / 2, inset), (-depth / 2 + bevel, outline), (depth / 2 - bevel, outline), (depth / 2, inset)]]
    caps = m.G.extrude(inset, depth)[1][:count - 2]
    faces = [(c, b, a) for a, b, c in caps] + [(a + 3 * count, b + 3 * count, c + 3 * count) for a, b, c in caps]
    faces += [face for row in range(3) for i in range(count) for j in [(i + 1) % count] for face in
        [(row * count + i, row * count + j, (row + 1) * count + j), (row * count + i, (row + 1) * count + j, (row + 1) * count + i)]]
    return m.part(([point for ring in rings for point in ring], faces), material)


def coin_token(m):
    return [m.C((0, 0.045, 0), 0.76, 0.09, "gold", 80)]


def collection(m):
    return [(name, label, lambda create=create: create(m)) for name, label, create in [
        ("coin-token", "Coin Token", coin_token), ("die-d20", "Twenty-Sided Die", die_d20),
        ("die-d6", "Six-Sided Die", die_d6), ("flag-marker", "Flag Marker", flag_marker),
        ("house", "House", house), ("outpost-tower", "Outpost Tower", outpost_tower),
        ("player-pawn", "Player Pawn", player_pawn), ("resource-barrel", "Resource Barrel", resource_barrel),
        ("resource-cube", "Resource Cube", resource_cube), ("score-disc", "Score Disc", score_disc),
        ("terrain-hex", "Terrain Hex", terrain_hex), ("worker-meeple", "Worker Meeple", worker_meeple)]]


def die_d20(m):
    phi = (1 + math.sqrt(5)) / 2
    vertices = [m.G.unit(v) for a in [-1, 1] for b in [-1, 1] for v in [(0, a, b * phi), (a, b * phi, 0), (a * phi, 0, b)]]
    distances = [sum((a - b) ** 2 for a, b in zip(vertices[i], vertices[j])) for i, j in itertools.combinations(range(12), 2)]
    edge = min(distances)
    raw = [face for face in itertools.combinations(range(12), 3) if all(abs(sum((a - b) ** 2 for a, b in zip(vertices[i], vertices[j])) - edge) < 1e-8 for i, j in itertools.combinations(face, 2))]
    faces = [face if dot(m.G.cross(sub(vertices[face[1]], vertices[face[0]]), sub(vertices[face[2]], vertices[face[0]])), vertices[face[0]]) > 0 else (face[0], face[2], face[1]) for face in raw]
    normals = [m.G.unit(tuple(sum(vertices[i][a] for i in face) for a in range(3))) for face in faces]
    # Rotating a face normal to down gives the die a stable, flat resting face.
    down = tuple(-v for v in normals[0])
    right = m.G.unit(m.G.cross((0, 0, 1), down))
    forward = m.G.cross(right, down)
    orient = lambda v: (dot(v, right) * 1.15, dot(v, down) * 1.15, dot(v, forward) * 1.15)
    vertices = [orient(v) for v in vertices]
    centers = [tuple(sum(vertices[i][a] for i in face) / 3 for a in range(3)) for face in faces]
    normals = [m.G.unit(center) for center in centers]
    numbers = {}
    for i in range(20):
        if i not in numbers:
            opposite = min(range(20), key=lambda j: dot(normals[i], normals[j]))
            value = len(numbers) // 2 + 1
            numbers[i], numbers[opposite] = value, 21 - value
    inset = {(i, vertex): tuple(centers[i][a] + (vertices[vertex][a] - centers[i][a]) * 0.94 for a in range(3)) for i, face in enumerate(faces) for vertex in face}
    parts = []
    for i, face in enumerate(faces):
        parts.append(m.part(([inset[i, vertex] for vertex in face], [(0, 1, 2)]), "water", group=f"face-{numbers[i]}"))
        up = m.G.unit(sub((0, 1, 0), tuple(normals[i][a] * normals[i][1] for a in range(3)))) if abs(normals[i][1]) < 0.98 else (0, 0, -1)
        right = m.G.unit(m.G.cross(up, normals[i]))
        parts += number(m, numbers[i], centers[i], normals[i], right, up, 0.34, "ivory", f"face-{numbers[i]}")
    for edge in itertools.combinations(range(12), 2):
        adjacent = [i for i, face in enumerate(faces) if all(v in face for v in edge)]
        if len(adjacent) == 2:
            i, j = adjacent
            parts.append(surface(m, [inset[i, edge[0]], inset[i, edge[1]], inset[j, edge[1]], inset[j, edge[0]]], "water"))
    for vertex in range(12):
        points = [inset[i, vertex] for i, face in enumerate(faces) if vertex in face]
        normal = m.G.unit(vertices[vertex])
        center = tuple(sum(p[a] for p in points) / len(points) for a in range(3))
        right = m.G.unit(sub(points[0], center))
        up = m.G.cross(normal, right)
        points.sort(key=lambda p: math.atan2(dot(sub(p, center), up), dot(sub(p, center), right)))
        parts.append(surface(m, points, "water"))
    return parts


def die_d6(m):
    values = [((0, 1, 0), (1, 0, 0), (0, 0, -1), 1), ((0, -1, 0), (1, 0, 0), (0, 0, 1), 6),
        ((0, 0, 1), (1, 0, 0), (0, 1, 0), 2), ((0, 0, -1), (-1, 0, 0), (0, 1, 0), 5),
        ((1, 0, 0), (0, 0, -1), (0, 1, 0), 3), ((-1, 0, 0), (0, 0, 1), (0, 1, 0), 4)]
    patterns = {1: [(0, 0)], 2: [(-1, 1), (1, -1)], 3: [(-1, 1), (0, 0), (1, -1)],
        4: [(-1, -1), (-1, 1), (1, -1), (1, 1)],
        5: [(-1, -1), (-1, 1), (0, 0), (1, -1), (1, 1)],
        6: [(x, y) for x in [-1, 1] for y in [-1, 0, 1]]}
    parts = [m.part(m.G.box(1.6, 1.6, 1.6, bevel=0.13), "ivory", (0, 0.8, 0))]
    for normal, right, up, value in values:
        for x, y in patterns[value]:
            center = tuple((0, 0.8, 0)[a] + normal[a] * 0.798 + right[a] * x * 0.38 + up[a] * y * 0.38 for a in range(3))
            disk = m.G.sphere(sides=24, rings=12)
            vertices = [tuple(center[a] + right[a] * v[0] * 0.125 + up[a] * v[1] * 0.125 + normal[a] * v[2] * 0.008 for a in range(3)) for v in disk[0]]
            parts.append(m.part((vertices, disk[1]), "teal", smooth=True, group=f"face-{value}"))
    return parts


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def finish(value, brand):
    materials = {key: brand.material(key, metallic=0.35 if key == "gold" else 0, roughness=0.48) for key in value["materials"]}
    return dict(value, materials=materials, revision=2, provenance="Original simple tabletop component with clean solid-color geometry and soft edges. Dice carry complete modeled markings. No imported meshes or licensed game designs.")


def flag_marker(m):
    return [m.C((0, 0.06, 0), 0.43, 0.12, "teal", 64),
        m.R((0, 0.10, 0), (0, 1.65, 0), 0.042, "teal"),
        beveled_outline(m, [(0, 1.04), (0.87, 1.34), (0, 1.65)], 0.07, bevel=0.015)]


def house(m):
    return [beveled_outline(m, [(-0.66, 0), (0.66, 0), (0.66, 0.75), (0, 1.25), (-0.66, 0.75)], 1.0)]


def number(m, value, center, normal, right, up, height, material, group="markings"):
    text = str(value)
    strokes = [(tuple((a[0] * 0.58 + (i - (len(text) - 1) / 2) * 0.64, a[1])), tuple((b[0] * 0.58 + (i - (len(text) - 1) / 2) * 0.64, b[1])))
        for i, digit in enumerate(text) for segment in DIGITS[digit] for a, b in [SEGMENTS[segment]]]
    strokes += [((-0.24, -0.69), (0.24, -0.69))] if value in {6, 9} else []
    parts = []
    for a, b in strokes:
        direction = m.G.unit((b[0] - a[0], b[1] - a[1]))
        width = 0.045
        polygon = [(a[0] - direction[1] * width, a[1] + direction[0] * width), (b[0] - direction[1] * width, b[1] + direction[0] * width),
            (b[0] + direction[1] * width, b[1] - direction[0] * width), (a[0] + direction[1] * width, a[1] - direction[0] * width)]
        mesh = m.G.extrude(polygon, 0.006)
        vertices = [tuple(center[i] + right[i] * p[0] * height + up[i] * p[1] * height + normal[i] * (p[2] + 0.006) for i in range(3)) for p in mesh[0]]
        parts.append(m.part((vertices, mesh[1]), material, group=group))
    return parts


def outpost_tower(m):
    return [m.L((0, 0, 0), [(0, 0.57), (0.05, 0.62), (0.14, 0.62), (0.23, 0.48),
        (1.16, 0.37), (1.24, 0.52), (1.39, 0.52)], "teal", 64),
        *[m.B((0.42 * math.cos(a), 1.50, 0.42 * math.sin(a)), (0.24, 0.28, 0.24), "teal", (0, -a, 0))
            for a in [i * math.pi / 3 for i in range(6)]]]


def player_pawn(m):
    profile = [(0, 0.44), (0.04, 0.49), (0.13, 0.49), (0.21, 0.42), (0.52, 0.29), (0.86, 0.18), (1.06, 0.16)]
    profile += [(1.34 + 0.32 * math.sin(a), 0.32 * math.cos(a)) for a in
        [-math.pi / 3 + i * 5 * math.pi / 6 / 20 for i in range(21)]]
    return [m.L((0, 0, 0), profile, "teal", 64)]


def resource_barrel(m):
    return [m.L((0, 0, 0), [(0, 0.44), (0.04, 0.48), (0.27, 0.56), (0.60, 0.60),
        (0.93, 0.56), (1.16, 0.48), (1.20, 0.44)], "amber", 64)]


def resource_cube(m):
    return [m.part(m.G.box(1.12, 1.12, 1.12, bevel=0.05), "teal", (0, 0.56, 0))]


def score_disc(m):
    return [m.C((0, 0.075, 0), 0.76, 0.15, "teal", 80)]


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def surface(m, points, material):
    normal = m.G.cross(sub(points[1], points[0]), sub(points[2], points[0]))
    ordered = points if dot(normal, points[0]) > 0 else list(reversed(points))
    return m.part((ordered, [(0, i, i + 1) for i in range(1, len(points) - 1)]), material)


def terrain_hex(m):
    return [m.C((0, 0.10, 0), 1.23, 0.20, "teal", 6)]


def worker_meeple(m):
    outline = [(-0.21, 0), (-0.55, 0), (-0.57, 0.10), (-0.35, 0.71), (-0.69, 0.55), (-0.84, 0.65), (-0.89, 0.84),
        (-0.64, 1.09), (-0.28, 1.24), (-0.32, 1.45), (-0.25, 1.65), (-0.10, 1.76), (0.10, 1.76), (0.25, 1.65),
        (0.32, 1.45), (0.28, 1.24), (0.64, 1.09), (0.89, 0.84), (0.84, 0.65), (0.69, 0.55), (0.35, 0.71),
        (0.57, 0.10), (0.55, 0), (0.21, 0), (0, 0.47)]
    return [beveled_outline(m, outline, 0.44)]
