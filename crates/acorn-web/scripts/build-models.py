#!/usr/bin/env python3
"""Build checked-in, self-contained GLBs from crate-owned model recipes (MIT)."""

import argparse
import hashlib
import importlib.util
import itertools
import json
import math
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1] / "assets" / "models"


def cross(left, right):
    return (left[1] * right[2] - left[2] * right[1],
            left[2] * right[0] - left[0] * right[2],
            left[0] * right[1] - left[1] * right[0])


def primitive_geometry(part):
    """Triangulate convex horizontal rings with outward, flat face normals."""
    width, height, depth = part["size"]
    bevel = part["bevel"]
    assert 0 < bevel < min(width, height, depth) / 2
    if part["shape"] == "cylinder":
        angles = tuple(2 * math.pi * index / 48 for index in range(48))
        ring = lambda inset: tuple(((width / 2 - inset) * math.cos(a),
                                    (depth / 2 - inset) * math.sin(a)) for a in angles)
    else:
        assert part["shape"] == "box", "Unsupported recipe shape"
        ring = lambda inset: tuple((x, z) for x, z in (
            (-width / 2 + bevel, -depth / 2 + inset),
            (width / 2 - bevel, -depth / 2 + inset),
            (width / 2 - inset, -depth / 2 + bevel),
            (width / 2 - inset, depth / 2 - bevel),
            (width / 2 - bevel, depth / 2 - inset),
            (-width / 2 + bevel, depth / 2 - inset),
            (-width / 2 + inset, depth / 2 - bevel),
            (-width / 2 + inset, -depth / 2 + bevel)))
    # Half-bevel end rings keep all eight corner edges nondegenerate.
    levels = ((-height / 2, bevel / 2), (-height / 2 + bevel, 0),
              (height / 2 - bevel, 0), (height / 2, bevel / 2))
    rings = tuple(tuple((x, y, z) for x, z in ring(inset)) for y, inset in levels)
    count = len(rings[0])
    sides = tuple((rings[level][index], rings[level][(index + 1) % count],
                   rings[level + 1][(index + 1) % count], rings[level + 1][index])
                  for level in range(3) for index in range(count))
    faces = sides + (rings[0], rings[-1])
    vertices = []
    normals = []
    for face in faces:
        normal = cross(tuple(b - a for a, b in zip(face[0], face[1])),
                       tuple(b - a for a, b in zip(face[0], face[2])))
        center = tuple(sum(vertex[axis] for vertex in face) / len(face) for axis in range(3))
        outward = face if sum(a * b for a, b in zip(center, normal)) > 0 else tuple(reversed(face))
        normal = cross(tuple(b - a for a, b in zip(outward[0], outward[1])),
                       tuple(b - a for a, b in zip(outward[0], outward[2])))
        length = math.sqrt(sum(value * value for value in normal))
        assert length > 0
        unit = tuple(value / length for value in normal)
        triangles = tuple((outward[0], outward[index], outward[index + 1])
                          for index in range(1, len(outward) - 1))
        vertices.extend(tuple(a + b for a, b in zip(vertex, part["center"]))
                        for triangle in triangles for vertex in triangle)
        normals.extend(unit for triangle in triangles for _ in triangle)
    return vertices, normals


def geometry(part):
    """Expand authored indexed meshes or the original beveled primitives."""
    if part["shape"] == "mesh":
        positions = part["vertices"]
        faces = part["triangles"]
        vectors = [cross(tuple(positions[b][axis] - positions[a][axis] for axis in range(3)),
                         tuple(positions[c][axis] - positions[a][axis] for axis in range(3))) for a, b, c in faces]
        assert all(sum(v * v for v in normal) > 1e-20 for normal in vectors), part["name"]
        sums = [[0.0, 0.0, 0.0] for _ in positions]
        if part.get("smooth"):
            for face, normal in zip(faces, vectors):
                for index in face:
                    sums[index] = [a + b for a, b in zip(sums[index], normal)]
        normal_values = [sums[index] if part.get("smooth") else normal
                         for face, normal in zip(faces, vectors) for index in face]
        normals = [tuple(value / math.sqrt(sum(v * v for v in normal)) for value in normal)
                   for normal in normal_values]
        vertices = [positions[index] for face in faces for index in face]
        result = vertices, normals
    else:
        result = primitive_geometry(part)
    return result


def roughness_png():
    """Deterministic packed PBR surface grain; green roughness, blue metal factor."""
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    pixels = b"".join(b"\0" + bytes(channel for x in range(64) for channel in
                    [255, 214 + ((x * 374761393 + y * 668265263) ^ ((x + y) * 1274126177)) % 42, 255]) for y in range(64))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 64, 64, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(pixels, 9)) + chunk(b"IEND", b"")


def texture_coordinates(positions, normals, scale=3, upright=False):
    """Stable planar projection per triangle, in meters, with repeatable grain."""
    coordinates = []
    for offset in range(0, len(positions), 3):
        normal = tuple(sum(n[axis] for n in normals[offset:offset + 3]) for axis in range(3))
        dominant = max(range(3), key=lambda axis: abs(normal[axis]))
        axes = [axis for axis in range(3) if axis != dominant]
        if upright and dominant == 0:
            axes = [2, 1]
        coordinates.extend(tuple(p[axis] * scale for axis in axes) for p in positions[offset:offset + 3])
    return coordinates


def weld(positions, normals, coordinates):
    """Share bit-identical float32 corners; return unique attributes and triangle indices."""
    keys = [struct.pack("<8f", *p, *n, *c) for p, n, c in zip(positions, normals, coordinates)]
    first = {}
    for offset, key in enumerate(keys):
        first.setdefault(key, (len(first), offset))
    order = [offset for _, offset in first.values()]
    return ([positions[i] for i in order], [normals[i] for i in order], [coordinates[i] for i in order],
            [first[key][0] for key in keys])


def build(recipe):
    """Return deterministic GLB bytes and catalog metadata, without file I/O."""
    materials = sorted(recipe["materials"])
    groups = sorted({part["group"] for part in recipe["parts"]})
    document = {
        "asset": {"version": "2.0", "generator": "ACORN build-models.py",
                  "copyright": recipe.get("copyright", "ACORN contributors; MIT"),
                  "extras": {"id": recipe["id"], "revision": recipe["revision"]}},
        "scene": 0, "scenes": [{"nodes": list(range(len(groups)))}],
        "nodes": [], "meshes": [], "materials": [dict(name=name, **recipe["materials"][name]) for name in materials],
        "accessors": [], "bufferViews": [], "buffers": [],
    }
    binary = bytearray()
    all_vertices = []

    def accessor(values, bounds=False):
        index = len(document["accessors"])
        dimensions = len(values[0])
        packed = struct.pack("<" + "f" * len(values) * dimensions, *itertools.chain.from_iterable(values))
        document["bufferViews"].append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(packed), "target": 34962})
        binary.extend(packed)
        value = {"bufferView": index, "componentType": 5126, "count": len(values), "type": f"VEC{dimensions}"}
        if bounds:
            value.update(min=[min(v[axis] for v in values) for axis in range(3)],
                         max=[max(v[axis] for v in values) for axis in range(3)])
        document["accessors"].append(value)
        return index

    def index_accessor(indices):
        index = len(document["accessors"])
        wide = max(indices) > 65535
        packed = struct.pack("<" + ("I" if wide else "H") * len(indices), *indices)
        document["bufferViews"].append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(packed), "target": 34963})
        binary.extend(packed + b"\0" * (-len(packed) % 4))
        document["accessors"].append({"bufferView": index, "componentType": 5125 if wide else 5123, "count": len(indices), "type": "SCALAR"})
        return index

    for group in groups:
        primitives = []
        for material in materials:
            selected = tuple(part for part in recipe["parts"] if part["group"] == group and part["material"] == material)
            parts = tuple(geometry(part) for part in selected)
            if parts:
                positions = tuple(itertools.chain.from_iterable(part[0] for part in parts))
                normals = tuple(itertools.chain.from_iterable(part[1] for part in parts))
                coordinates = texture_coordinates(positions, normals, recipe.get("surfaces", {}).get(material, {}).get("uvScale", 3), material in recipe.get("surfaces", {}))
                shared_positions, shared_normals, shared_coordinates, indices = weld(positions, normals, coordinates)
                primitives.append({"attributes": {"POSITION": accessor(shared_positions, True), "NORMAL": accessor(shared_normals),
                                                  "TEXCOORD_0": accessor(shared_coordinates)},
                                   "indices": index_accessor(indices), "material": materials.index(material), "mode": 4})
                all_vertices.extend(positions)
        document["nodes"].append({"name": group, "mesh": len(document["meshes"])})
        document["meshes"].append({"name": group, "primitives": primitives})
    grain = roughness_png()
    image_view = len(document["bufferViews"])
    document["bufferViews"].append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(grain)})
    binary.extend(grain)
    binary.extend(b"\0" * (-len(binary) % 4))
    document["images"] = [{"bufferView": image_view, "mimeType": "image/png", "name": "surface-roughness"}]
    document["samplers"] = [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}]
    document["textures"] = [{"source": 0, "sampler": 0}]
    for material in document["materials"]:
        material["pbrMetallicRoughness"] = dict(material["pbrMetallicRoughness"], metallicRoughnessTexture={"index": 0})
    if recipe.get("surfaces"):
        spec = importlib.util.spec_from_file_location("model_surface", Path(__file__).with_name("model-surface.py"))
        surface = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(surface)
        for kind in sorted({value["kind"] for value in recipe["surfaces"].values()}):
            pixels = surface.surface_png(kind)
            view = len(document["bufferViews"])
            document["bufferViews"].append({"buffer": 0, "byteOffset": len(binary), "byteLength": len(pixels)})
            binary.extend(pixels)
            binary.extend(b"\0" * (-len(binary) % 4))
            index = len(document["textures"])
            document["images"].append({"bufferView": view, "mimeType": "image/png", "name": f"surface-{kind}"})
            document["textures"].append({"source": index, "sampler": 0})
            for material in document["materials"]:
                if recipe["surfaces"].get(material["name"], {}).get("kind") == kind:
                    material["pbrMetallicRoughness"]["baseColorTexture"] = {"index": index}
    document["buffers"] = [{"byteLength": len(binary)}]
    encoded = json.dumps(document, separators=(",", ":"), sort_keys=True).encode()
    encoded += b" " * (-len(encoded) % 4)
    payload = (struct.pack("<I4s", len(encoded), b"JSON") + encoded
               + struct.pack("<I4s", len(binary), b"BIN\x00") + binary)
    glb = struct.pack("<4sII", b"glTF", 2, 12 + len(payload)) + payload
    metadata = {
        "bounds": {"min": [min(v[axis] for v in all_vertices) for axis in range(3)],
                   "max": [max(v[axis] for v in all_vertices) for axis in range(3)]},
        "bytes": len(glb), "category": recipe["category"], "front_axis": "+Z",
        "id": recipe["id"], "label": recipe["label"], "license": recipe.get("license", "MIT"),
        "materials": len(materials), "nodes": groups, "pivot": "bottom-center",
        "primitives": sum(len(mesh["primitives"]) for mesh in document["meshes"]),
        "provenance": recipe["provenance"], "revision": recipe["revision"],
        "sha256": hashlib.sha256(glb).hexdigest(), "triangles": len(all_vertices) // 3,
        "units": "meters", "up_axis": "+Y", "textures": len(document["textures"]),
    }
    return glb, metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Fail on stale GLB or catalog; write nothing")
    parser.add_argument("--root", type=Path, default=ROOT, help="Model library directory")
    args = parser.parse_args()
    entries = []
    for source in sorted(args.root.glob("*/*.model.json")):
        recipe = json.loads(source.read_text())
        glb, metadata = build(recipe)
        target = source.with_name(source.name.replace(".model.json", ".glb"))
        if args.check:
            assert target.read_bytes() == glb, f"Stale model: {target}"
        else:
            target.write_bytes(glb)
        entries.append(dict(metadata, file=target.relative_to(args.root).as_posix(), source=source.relative_to(args.root).as_posix()))
    catalog = json.dumps({"version": 1, "assets": sorted(entries, key=lambda entry: entry["id"])}, indent=2, sort_keys=True) + "\n"
    target = args.root / "catalog.json"
    if args.check:
        assert target.read_text() == catalog, "Stale model catalog"
    else:
        target.write_text(catalog)
    print(f"{'Checked' if args.check else 'Built'} {len(entries)} model(s): "
          + ", ".join(f"{entry['id']} ({entry['triangles']} triangles, {entry['bytes']} bytes)" for entry in entries))


if __name__ == "__main__":
    main()
