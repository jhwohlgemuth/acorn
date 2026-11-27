"""Original procedural mesh helpers for the ACORN miniature collection. MIT."""
import math


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def arch(radius, thickness, depth, start=0, end=math.pi, steps=20):
    outer = [(radius * math.cos(start + (end - start) * i / steps), radius * math.sin(start + (end - start) * i / steps)) for i in range(steps + 1)]
    inner = [((radius - thickness) * math.cos(start + (end - start) * i / steps), (radius - thickness) * math.sin(start + (end - start) * i / steps)) for i in reversed(range(steps + 1))]
    return extrude(outer + inner, depth)


def beam(a, b, radius, sides=12, end_radius=None):
    delta = tuple(y - x for x, y in zip(a, b))
    length = math.sqrt(sum(v * v for v in delta))
    up = unit(delta)
    reference = (0, 0, 1) if abs(up[2]) < 0.9 else (1, 0, 0)
    right = unit(cross(up, reference))
    front = cross(right, up)
    vertices, triangles = lathe([(0, radius), (length, radius if end_radius is None else end_radius)], sides)
    return ([tuple(a[j] + v[0] * right[j] + v[1] * up[j] + v[2] * front[j] for j in range(3)) for v in vertices], triangles)


def box(width, height, depth, bevel=0.035):
    b = min(bevel, min(width, height, depth) * 0.24)
    def ring(y, inset):
        return [(x, y, z) for x, z in [(-width / 2 + b, -depth / 2 + inset), (width / 2 - b, -depth / 2 + inset),
                (width / 2 - inset, -depth / 2 + b), (width / 2 - inset, depth / 2 - b),
                (width / 2 - b, depth / 2 - inset), (-width / 2 + b, depth / 2 - inset),
                (-width / 2 + inset, depth / 2 - b), (-width / 2 + inset, -depth / 2 + b)]]
    rings = [ring(-height / 2, b / 2), ring(-height / 2 + b, 0), ring(height / 2 - b, 0), ring(height / 2, b / 2)]
    return ring_mesh(rings)


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def dodecahedron(radius=1):
    """Triangulate twelve flat pentagonal faces with outward winding."""
    phi = (1 + math.sqrt(5)) / 2
    vertices = [(x, y, z) for x in [-1, 1] for y in [-1, 1] for z in [-1, 1]] + [
        point for a in [-1, 1] for b in [-1, 1] for point in
        [(0, a / phi, b * phi), (a / phi, b * phi, 0), (a * phi, 0, b / phi)]]
    normals = [point for a in [-1, 1] for b in [-1, 1] for point in
               [(0, a * phi, b), (a, 0, b * phi), (a * phi, b, 0)]]
    triangles = []
    for normal in normals:
        dots = [sum(a * b for a, b in zip(vertex, normal)) for vertex in vertices]
        face = [i for i, value in enumerate(dots) if abs(value - max(dots)) < 1e-9]
        center = tuple(sum(vertices[i][axis] for i in face) / len(face) for axis in range(3))
        right = unit(tuple(a - b for a, b in zip(vertices[face[0]], center)))
        up = cross(unit(normal), right)
        ordered = sorted(face, key=lambda i: math.atan2(
            sum((a - b) * c for a, b, c in zip(vertices[i], center, up)),
            sum((a - b) * c for a, b, c in zip(vertices[i], center, right))))
        triangles.extend((ordered[0], ordered[i], ordered[i + 1]) for i in range(1, len(ordered) - 1))
    return ([tuple(value * radius / math.sqrt(3) for value in vertex) for vertex in vertices], triangles)


def extrude(polygon, depth):
    """Extrude a simple XY polygon, using ear clipping for concave glyphs."""
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(polygon, polygon[1:] + polygon[:1]))
    points = polygon if area > 0 else list(reversed(polygon))
    def turn(a, b, c):
        return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    remaining = list(range(len(points)))
    caps = []
    while len(remaining) > 3:
        ears = [i for i in range(len(remaining)) if turn(points[remaining[i - 1]], points[remaining[i]], points[remaining[(i + 1) % len(remaining)]]) > 1e-10
                and not any(all(turn(points[a], points[b], points[j]) >= -1e-10 for a, b in zip(
                    [remaining[i - 1], remaining[i], remaining[(i + 1) % len(remaining)]],
                    [remaining[i], remaining[(i + 1) % len(remaining)], remaining[i - 1]]))
                    for j in remaining if j not in [remaining[i - 1], remaining[i], remaining[(i + 1) % len(remaining)]])]
        assert ears, "Polygon is not simple"
        i = ears[0]
        caps.append((remaining[i - 1], remaining[i], remaining[(i + 1) % len(remaining)]))
        remaining.pop(i)
    caps.append(tuple(remaining))
    n = len(points)
    vertices = [(x, y, depth / 2) for x, y in points] + [(x, y, -depth / 2) for x, y in points]
    faces = caps + [(c + n, b + n, a + n) for a, b, c in caps]
    faces += [face for i in range(n) for face in [(i, i + n, (i + 1) % n + n), (i, (i + 1) % n + n, (i + 1) % n)]]
    return vertices, faces


def lathe(profile, sides=32):
    rings = [[(radius * math.cos(2 * math.pi * i / sides), y, radius * math.sin(2 * math.pi * i / sides)) for i in range(sides)] for y, radius in profile]
    return ring_mesh(rings)


def pyramid(width, height, depth, top=0):
    points = [(-width / 2, 0, -depth / 2), (width / 2, 0, -depth / 2), (width / 2, 0, depth / 2), (-width / 2, 0, depth / 2)]
    upper = [(x * top, height, z * top) for x, _, z in points]
    return ring_mesh([points, upper])


def ring_mesh(rings):
    n = len(rings[0])
    vertices = [v for ring in rings for v in ring]
    faces = [face for row in range(len(rings) - 1) for i in range(n) for face in
             [(row * n + i, (row + 1) * n + i, (row + 1) * n + (i + 1) % n),
              (row * n + i, (row + 1) * n + (i + 1) % n, row * n + (i + 1) % n)]]
    faces += [(0, i, i + 1) for i in range(1, n - 1)]
    last = (len(rings) - 1) * n
    faces += [(last, last + i + 1, last + i) for i in range(1, n - 1)]
    return vertices, faces


def sphere(radius=1, sides=32, rings=20):
    return lathe([(-radius * math.cos(math.pi * i / rings), radius * math.sin(math.pi * i / rings)) for i in range(rings + 1)], sides)


def terrain(heights, width, depth, bottom=0):
    rows, columns = len(heights), len(heights[0])
    vertices = [((x / (columns - 1) - 0.5) * width, heights[z][x], (z / (rows - 1) - 0.5) * depth) for z in range(rows) for x in range(columns)]
    triangles = [face for z in range(rows - 1) for x in range(columns - 1) for a in [z * columns + x]
                 for face in [(a, a + columns, a + 1), (a + 1, a + columns, a + columns + 1)]]
    edge = list(range(columns)) + [z * columns + columns - 1 for z in range(1, rows)] + list(reversed(range((rows - 1) * columns, rows * columns - 1))) + [z * columns for z in reversed(range(1, rows - 1))]
    base = len(vertices)
    vertices += [(vertices[i][0], bottom, vertices[i][2]) for i in edge]
    triangles += [face for i, a in enumerate(edge) for b in [edge[(i + 1) % len(edge)]] for face in
                  [(a, base + (i + 1) % len(edge), base + i), (a, b, base + (i + 1) % len(edge))]]
    triangles += [(base, base + i, base + i + 1) for i in range(1, len(edge) - 1)]
    return vertices, triangles


def torus(radius, tube, sides=48, segments=10):
    vertices = [((radius + tube * math.cos(2 * math.pi * j / segments)) * math.cos(2 * math.pi * i / sides),
                 tube * math.sin(2 * math.pi * j / segments),
                 (radius + tube * math.cos(2 * math.pi * j / segments)) * math.sin(2 * math.pi * i / sides)) for i in range(sides) for j in range(segments)]
    faces = [face for i in range(sides) for j in range(segments) for a, b, c, d in
             [(i * segments + j, i * segments + (j + 1) % segments, ((i + 1) % sides) * segments + (j + 1) % segments, ((i + 1) % sides) * segments + j)]
             for face in [(a, b, c), (a, c, d)]]
    return vertices, faces


def transform(mesh, center=(0, 0, 0), scale=(1, 1, 1), rotation=(0, 0, 0)):
    rx, ry, rz = rotation
    def vertex(value):
        x, y, z = (a * b for a, b in zip(value, scale))
        y, z = y * math.cos(rx) - z * math.sin(rx), y * math.sin(rx) + z * math.cos(rx)
        x, z = x * math.cos(ry) + z * math.sin(ry), -x * math.sin(ry) + z * math.cos(ry)
        x, y = x * math.cos(rz) - y * math.sin(rz), x * math.sin(rz) + y * math.cos(rz)
        return tuple(round(a + b, 7) for a, b in zip((x, y, z), center))
    vertices = [vertex(v) for v in mesh[0]]
    faces = [face for face in mesh[1] if sum(v * v for v in cross(
        tuple(vertices[face[1]][a] - vertices[face[0]][a] for a in range(3)),
        tuple(vertices[face[2]][a] - vertices[face[0]][a] for a in range(3)))) > 1e-18]
    return vertices, faces


def unit(vector):
    length = math.sqrt(sum(v * v for v in vector))
    return tuple(v / length for v in vector)
