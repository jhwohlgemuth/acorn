#!/usr/bin/env python3
"""Extrude ACORN's tracked SVG logo, retaining its open cutouts. MIT."""
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import subprocess
from xml.etree import ElementTree

COLOR = "#80350E"


def area(points):
    return sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(points, points[1:] + points[:1])) / 2


def cubic_points(a, b, c, d, depth=0):
    excess = math.dist(a, b) + math.dist(b, c) + math.dist(c, d) - math.dist(a, d)
    if excess < .5 or depth == 10:
        result = [d]
    else:
        ab, bc, cd = [tuple((x + y) / 2 for x, y in zip(left, right)) for left, right in [(a, b), (b, c), (c, d)]]
        abc, bcd = [tuple((x + y) / 2 for x, y in zip(left, right)) for left, right in [(ab, bc), (bc, cd)]]
        middle = tuple((x + y) / 2 for x, y in zip(abc, bcd))
        result = cubic_points(a, ab, abc, middle, depth + 1) + cubic_points(middle, bcd, cd, d, depth + 1)
    return result


def extrusion(m, contours, triangles, group, depth=.44, bevel=.014):
    """Join exact silhouette walls to inset faces, leaving every cutout open."""
    front_contours = [inset(contour, bevel) for contour in contours]
    flat = [point for contour in contours for point in contour]
    front = [point for contour in front_contours for point in contour]
    front_faces = [face if area([front[i] for i in face]) > 0 else tuple(reversed(face)) for face in triangles]
    parts = [m.part(([(x, y, depth / 2) for x, y in front], front_faces), "acorn-brown", group=group),
             m.part(([(x, y, -depth / 2) for x, y in front], [tuple(reversed(face)) for face in front_faces]), "acorn-brown", group=group)]
    rings = [[(x, y, z) for x, y in points] for z, points in
             [(-depth / 2, front), (-depth / 2 + bevel, flat), (depth / 2 - bevel, flat), (depth / 2, front)]]
    count, offset = len(flat), 0
    faces = []
    for contour in contours:
        for i in range(len(contour)):
            a, b = offset + i, offset + (i + 1) % len(contour)
            faces += [face for row in range(3) for face in
                      [(row * count + a, row * count + b, (row + 1) * count + b),
                       (row * count + a, (row + 1) * count + b, (row + 1) * count + a)]]
        offset += len(contour)
    parts.append(m.part(([point for ring in rings for point in ring], faces), "acorn-brown", smooth=True, group=group))
    return parts


def inset(points, distance):
    result = []
    for i, point in enumerate(points):
        left, right = points[i - 1], points[(i + 1) % len(points)]
        a = tuple((point[axis] - left[axis]) / math.dist(point, left) for axis in range(2))
        b = tuple((right[axis] - point[axis]) / math.dist(point, right) for axis in range(2))
        normal = (-a[1] - b[1], a[0] + b[0])
        amount = distance / max(.05, normal[0] * -a[1] + normal[1] * a[0])
        amount = min(amount, distance * 3 / max(math.hypot(*normal), .001))
        result.append((point[0] + normal[0] * amount, point[1] + normal[1] * amount))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/models")
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[3] / "docs/src/_assets/acorn.svg"
    paths = list(ElementTree.parse(source).getroot().iter("{http://www.w3.org/2000/svg}path"))
    shapes = [outline(path.attrib["d"]) for path in paths]
    assert [len(shape) for shape in shapes] == [5, 1], "ACORN logo must contain a four-cutout cap and a separate nut"
    points = [point for shape in shapes for contour in shape for point in contour]
    center_x = (min(p[0] for p in points) + max(p[0] for p in points)) / 2
    bottom = min(p[1] for p in points)
    scale = 3.4 / (max(p[1] for p in points) - bottom)
    shapes = [[[(round((x - center_x) * scale, 8), round((y - bottom) * scale, 8)) for x, y in contour] for contour in shape] for shape in shapes]
    shapes = [[contour if (area(contour) > 0) == (i == 0) else list(reversed(contour)) for i, contour in enumerate(shape)] for shape in shapes]
    bevels = [.014, .004]  # The nut's narrow opening needs a smaller edge radius.
    triangles = triangulate([[inset(contour, bevel) for contour in shape] for shape, bevel in zip(shapes, bevels)])
    spec = importlib.util.spec_from_file_location("collection", Path(__file__).with_name("model-collection.py"))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    parts = [part for shape, faces, name, bevel in zip(shapes, triangles, ["cap-and-stem", "nut-body"], bevels) for part in extrusion(m, shape, faces, name, bevel=bevel)]
    material = {"extras": {"brand": "custom accent", "colorName": "ACORN Brown", "srgbHex": COLOR},
                "pbrMetallicRoughness": {"baseColorFactor": [*m.BRAND.linear(COLOR), 1], "metallicFactor": .06, "roughnessFactor": .4}}
    value = {"id": "research/acorn-logo", "category": "research", "label": "ACORN Logo", "revision": 1,
             "materials": {"acorn-brown": material}, "parts": [dict(part, name=f"{part['group']}-{i}") for i, part in enumerate(parts)],
             "provenance": "Three-dimensional extrusion of docs/src/_assets/acorn.svg in its ACORN Brown (#80350E). Source silhouette, curved stem, four cap openings, cap/body gap and curved nut cutout retained as geometry, with shallow beveled edges. Original repository artwork and derived geometry under the repository MIT license. No background tile or pedestal. Illustrative 3.4-meter authoring height; +Y up, +Z front and bottom-center origin."}
    target = args.root / "research/acorn-logo.model.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")
    palette_path = args.root / "palette.json"
    palette = json.loads(palette_path.read_text())
    palette["customAccents"] = palette.get("customAccents", {}) | {"ACORN Brown": COLOR}
    palette_path.write_text(json.dumps(palette, sort_keys=True, indent=2) + "\n")
    print(f"Authored {value['id']}: {sum(len(part['triangles']) for part in parts)} triangles")


def outline(data):
    """Flatten this logo's move, line and cubic commands, including closed holes."""
    tokens = re.findall(r"[A-Za-z]|[-+]?(?:\d*\.?\d+)(?:[eE][-+]?\d+)?", data)
    command, index, current, start = "", 0, (0, 0), (0, 0)
    contours, points = [], []
    while index < len(tokens):
        if tokens[index].isalpha():
            command = tokens[index]
            index += 1
        assert command in {"M", "m", "c", "l", "z"}, f"Unexpected ACORN SVG command: {command}"
        count = {"M": 2, "m": 2, "c": 6, "l": 2, "z": 0}[command]
        values = [float(value) for value in tokens[index:index + count]]
        index += count
        pairs = [tuple(values[i:i + 2]) for i in range(0, count, 2)]
        absolute = [tuple(current[axis] + point[axis] for axis in range(2)) for point in pairs] if command.islower() else pairs
        if command in {"M", "m"}:
            start = current = absolute[0]
            points = [current]
        elif command == "l":
            current = absolute[0]
            points.append(current)
        elif command == "c":
            points += cubic_points(current, *absolute)
            current = absolute[-1]
        else:
            contours.append(points[:-1] if math.dist(points[0], points[-1]) < 1e-6 else points)
            current = start
    return contours


def triangulate(shapes):
    """Use Cesium's installed Earcut dependency to preserve the logo's holes."""
    module = Path(__file__).resolve().parents[1] / "geospatial/node_modules/earcut/src/earcut.js"
    script = f"import earcut, {{deviation}} from {json.dumps(module.as_uri())};" + """
let input = '';
for await (const chunk of process.stdin) input += chunk;
const result = JSON.parse(input).map(contours => {
    const flat = contours.flat();
    const holes = contours.slice(0, -1).map((_, i) => contours.slice(0, i + 1).reduce((n, ring) => n + ring.length, 0));
    const coordinates = flat.flat();
    const indices = earcut(coordinates, holes, 2);
    if (deviation(coordinates, holes, 2, indices) > 1e-7) throw new Error('ACORN logo triangulation lost filled area');
    return Array.from({length: indices.length / 3}, (_, i) => indices.slice(i * 3, i * 3 + 3));
});
process.stdout.write(JSON.stringify(result));
"""
    result = subprocess.run([os.environ.get("NODE_BINARY", "node"), "--input-type=module", "-e", script],
                            input=json.dumps(shapes), text=True, capture_output=True, check=True)
    return json.loads(result.stdout)


if __name__ == "__main__":
    main()
