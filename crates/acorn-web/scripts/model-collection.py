#!/usr/bin/env python3
"""Author the original ACORN/Risk miniature recipes. Standard library only. MIT."""
import argparse
import importlib.util
import json
import math
import sys
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("geometry", Path(__file__).with_name("model-geometry.py"))
G = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(G)
PI = math.pi
PALETTE_SPEC = importlib.util.spec_from_file_location("palette", Path(__file__).with_name("model-palette.py"))
BRAND = importlib.util.module_from_spec(PALETTE_SPEC)
PALETTE_SPEC.loader.exec_module(BRAND)
DETAIL_SPEC = importlib.util.spec_from_file_location("detail", Path(__file__).with_name("model-detail.py"))
DETAIL = importlib.util.module_from_spec(DETAIL_SPEC)
DETAIL_SPEC.loader.exec_module(DETAIL)


def part(mesh, material="ivory", center=(0, 0, 0), scale=(1, 1, 1), rotation=(0, 0, 0), smooth=False, group="body"):
    vertices, faces = G.transform(mesh, center, scale, rotation)
    return dict(shape="mesh", vertices=vertices, triangles=faces, material=material, group=group, smooth=smooth)


def B(center, size, material="ivory", rotation=(0, 0, 0)):
    return part(G.box(*size), material, center, rotation=rotation)


def C(center, radius, height, material="ivory", sides=32):
    b = min(height * 0.2, radius * 0.05)
    return part(G.lathe([(-height / 2, radius - b), (-height / 2 + b, radius), (height / 2 - b, radius), (height / 2, radius - b)], sides), material, center)


def E(points, depth, material="ivory", center=(0, 0, 0), rotation=(0, 0, 0)):
    return part(G.extrude(points, depth), material, center, rotation=rotation)


def L(center, profile, material="ivory", sides=32, scale=(1, 1, 1)):
    return part(G.lathe(profile, sides), material, center, scale=scale, smooth=True)


def P(center, size, material="ivory", top=0):
    return part(G.pyramid(*size, top), material, center)


def R(a, b, radius, material="ivory", end_radius=None):
    return part(G.beam(a, b, radius, end_radius=end_radius), material, smooth=True)


def S(center, scale, material="ivory", smooth=True):
    return part(G.sphere(sides=12 if max(scale) < 0.12 else 32, rings=8 if max(scale) < 0.12 else 20), material, center, scale, smooth=smooth)


def T(center, radius, tube, material="ivory", rotation=(0, 0, 0)):
    return part(G.torus(radius, tube), material, center, rotation=rotation, smooth=True)


def windows(x, y, z, count, gap=0.38, size=(0.25, 0.32, 0.035)):
    return [B((x + (i - (count - 1) / 2) * gap, y, z), size, "teal") for i in range(count)]


def laboratory():
    return [B((0, 0.62, 0), (2.6, 1.24, 1.65)), B((0, 1.30, 0), (2.75, 0.16, 1.8)),
            B((0, 1.48, -0.15), (1.3, 0.22, 0.88), "teal", (0.12, 0, 0)),
            B((0, 1.52, -0.15), (1.45, 0.06, 0.10), "ivory"), C((0.9, 1.58, -0.45), 0.16, 0.48),
            B((0, 0.48, 0.89), (0.55, 0.88, 0.15), "teal"), B((0, 0.94, 1.00), (0.9, 0.12, 0.5)),
            B((0, 0.06, 1.02), (0.95, 0.12, 0.6))] + windows(-0.88, 0.7, 0.835, 2) + windows(0.88, 0.7, 0.835, 2)


def doe_national_laboratory():
    return [C((0, 0.62, -0.5), 1.12, 1.24, "steel"), C((0, 1.27, -0.5), 1.17, 0.12, "dark"),
            B((0, 0.85, 0.67), (0.95, 1.7, 0.95)), B((0, 0.8, 1.17), (0.55, 1.45, 0.08), "teal"),
            B((-1.1, 0.55, 0.6), (1.05, 1.1, 1.1)), B((1.1, 0.55, 0.6), (1.05, 1.1, 1.1)),
            B((0, 0.04, 1.45), (1.3, 0.08, 0.7)), B((0, 1.75, 0.67), (1.08, 0.10, 1.08))] + windows(-1.1, 0.72, 1.17, 2) + windows(1.1, 0.72, 1.17, 2)


def user_facility():
    return [T((0, 0.65, 0), 1.35, 0.22), T((0, 0.65, 0), 1.35, 0.08, "teal"),
            C((0, 0.42, 0), 0.42, 0.84, "steel"), B((0, 0.9, 0), (0.55, 0.34, 0.55))] + [
            item for a in [i * PI / 4 for i in range(8)] for item in
            [B((1.35 * math.cos(a), 0.3, 1.35 * math.sin(a)), (0.3, 0.6, 0.3)),
             R((0, 0.65, 0), (1.35 * math.cos(a), 0.65, 1.35 * math.sin(a)), 0.07, "steel")]]


def research_project():
    bench = [B((0, 0.86, 0.12), (2.75, 0.14, 1.45), "ivory"),
             B((0, 0.76, 0.12), (2.65, 0.10, 1.35), "teal"),
             *[B((x, 0.36, z), (0.10, 0.72, 0.10), "steel") for x in [-1.2, 1.2] for z in [-0.45, 0.67]],
             *[B((x, 0.08, z), (0.14, 0.06, 0.14), "dark") for x in [-1.2, 1.2] for z in [-0.45, 0.67]],
             B((0, 0.30, -0.45), (2.45, 0.08, 0.08), "steel")]
    monitor = [B((0, 1.70, -0.46), (2.24, 1.20, 0.12), "dark"),
               B((0, 1.72, -0.393), (2.09, 1.04, 0.018), "pages"),
               B((0, 1.09, -0.48), (0.13, 0.34, 0.13), "steel"),
               B((0, 0.96, -0.40), (0.66, 0.06, 0.40), "steel"),
               S((0, 2.265, -0.39), (0.025, 0.025, 0.012), "water"),
               S((0.96, 1.15, -0.39), (0.015, 0.015, 0.01), "teal"),
               *[B((x, 1.85, -0.526), (0.022, 0.30, 0.015), "steel") for x in [-0.35 + i * 0.07 for i in range(11)]],
               *[B((x, 1.23, -0.53), (0.10, 0.055, 0.02), "steel") for x in [-0.45, -0.22, 0.22, 0.45]],
               R((0, 1.15, -0.54), (0, 0.98, -0.63), 0.02, "dark")]
    dashboard = [B((0, 2.18, -0.375), (2.06, 0.10, 0.018), "teal"),
                 *[S((x, 2.18, -0.36), (0.018, 0.018, 0.008), "pages") for x in [-0.95, -0.88, -0.81]],
                 *[B((x, y, -0.376), (0.86, 0.38, 0.012), "ivory") for x in [-0.51, 0.51] for y in [1.87, 1.40]],
                 *[B((-0.51, y, -0.365), (0.77, 0.008, 0.006), "steel") for y in [1.74, 1.84, 1.94, 2.02]],
                 R((-0.90, 1.71, -0.355), (-0.90, 2.04, -0.355), 0.008, "dark"),
                 R((-0.90, 1.71, -0.355), (-0.13, 1.71, -0.355), 0.008, "dark")]
    points = [(-0.86, 1.77, -0.35), (-0.73, 1.84, -0.35), (-0.60, 1.81, -0.35), (-0.47, 1.94, -0.35), (-0.32, 1.89, -0.35), (-0.17, 2.01, -0.35)]
    dashboard += [R(a, b, 0.015, "teal") for a, b in zip(points, points[1:])] + [S(point, (0.025, 0.025, 0.009), "teal") for point in points]
    dashboard += [B((0.23 + i * 0.14, 1.25 + height / 2, -0.35), (0.085, height, 0.018), "water" if i % 2 else "teal") for i, height in enumerate([0.10, 0.18, 0.13, 0.25, 0.29])]
    dashboard += [R((0.16, 1.23, -0.35), (0.88, 1.23, -0.35), 0.008, "dark")]
    dashboard += [R((0.51 + 0.13 * math.cos(i * PI / 16), 1.87 + 0.13 * math.sin(i * PI / 16), -0.35),
                    (0.51 + 0.13 * math.cos((i + 1) * PI / 16), 1.87 + 0.13 * math.sin((i + 1) * PI / 16), -0.35), 0.035, "teal" if i < 19 else "water" if i < 27 else "amber") for i in range(32)]
    nodes = [(-0.81, 1.40, -0.35), (-0.60, 1.52, -0.35), (-0.51, 1.29, -0.35), (-0.29, 1.44, -0.35)]
    dashboard += [R(nodes[a], nodes[b], 0.008, "steel") for a, b in [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]] + [S(point, (0.043, 0.043, 0.012), "water" if i % 2 else "teal") for i, point in enumerate(nodes)]
    keyboard = [B((-0.59, 0.97, 0.38), (1.10, 0.075, 0.36), "dark"),
                *[B((-1.04 + col * 0.09, 1.018, 0.27 + row * 0.075), (0.065, 0.026, 0.052), "ivory") for row in range(3) for col in range(11)],
                B((-0.59, 1.018, 0.505), (0.40, 0.026, 0.035), "ivory"),
                B((0.13, 0.94, 0.37), (0.26, 0.015, 0.36), "water"),
                S((0.13, 0.98, 0.37), (0.09, 0.055, 0.13), "ivory"),
                R((0.13, 1.035, 0.31), (0.13, 1.035, 0.40), 0.006, "dark")]
    notebook = [B((0.72, 0.96, 0.40), (0.90, 0.04, 0.66), "teal"),
                *[B((x, 0.99, 0.40), (0.39, 0.035, 0.58), "pages") for x in [0.49, 0.95]],
                *[B((x, 1.012, z), (0.28, 0.008, 0.013), "steel") for x in [0.49, 0.95] for z in [0.20, 0.29, 0.38, 0.47, 0.56]],
                R((0.72, 1.012, 0.10), (0.72, 1.012, 0.69), 0.015, "dark"),
                R((1.25, 0.96, 0.15), (1.25, 0.96, 0.64), 0.018, "teal"),
                R((1.25, 0.96, 0.64), (1.25, 0.96, 0.69), 0.018, "steel", 0.002)]
    papers = [B((-0.91 + i * 0.017, 0.94 + i * 0.009, -0.12 + i * 0.013), (0.55, 0.008, 0.45), "pages", (0, i * 0.04, 0)) for i in range(4)] + [
              B((-0.86, 0.979, -0.20 + i * 0.07), (0.32 if i else 0.39, 0.006, 0.012), "teal" if i == 0 else "steel") for i in range(4)] + [
              B((-0.98, 0.99, -0.30), (0.10, 0.016, 0.025), "steel")]
    accessories = [L((1.07, 0.94, -0.15), [(0, 0.10), (0.22, 0.11)], "teal"),
                   T((1.07, 1.16, -0.15), 0.106, 0.012, "ivory"),
                   C((1.07, 1.15, -0.15), 0.095, 0.012, "dark"),
                   T((1.20, 1.05, -0.15), 0.065, 0.018, "teal", (PI / 2, 0, 0)),
                   B((0.91, 0.40, -0.04), (0.36, 0.65, 0.55), "dark"),
                   B((0.91, 0.40, 0.242), (0.31, 0.60, 0.025), "steel"),
                   *[B((0.91, 0.22 + i * 0.04, 0.259), (0.23, 0.015, 0.01), "dark") for i in range(7)],
                   B((0.91, 0.63, 0.261), (0.08, 0.025, 0.012), "dark"),
                   S((1.01, 0.64, 0.267), (0.015, 0.015, 0.008), "teal")]
    return bench + monitor + dashboard + keyboard + notebook + papers + accessories


def dataset():
    return [C((0, y, 0), 0.95, 0.44) for y in [0.25, 0.85, 1.45]] + [C((0, y, 0), 0.88, 0.18, "teal") for y in [0.55, 1.15]]


def software():
    chevron = [(-0.25, 0), (0.18, 0.48), (0.31, 0.34), (0, 0), (0.31, -0.34), (0.18, -0.48)]
    return [B((0, 1.05, 0), (2.6, 2.1, 0.32)), E(chevron, 0.10, "teal", (-0.8, 1.05, 0.22)),
            E(chevron, 0.10, "teal", (0.8, 1.05, 0.22), (0, 0, PI)),
            B((0, 1.05, 0.22), (0.16, 1.35, 0.10), "teal", (0, 0, -0.28))]


def publication():
    return [B((-0.62, 0.20, 0), (1.3, 0.16, 1.8), "dark", (0, 0, -0.16)),
            B((0.62, 0.20, 0), (1.3, 0.16, 1.8), "dark", (0, 0, 0.16)),
            B((-0.62, 0.34, 0), (1.18, 0.20, 1.65), "pages", (0, 0, -0.16)),
            B((0.62, 0.34, 0), (1.18, 0.20, 1.65), "pages", (0, 0, 0.16)),
            R((0, 0.30, -0.88), (0, 0.30, 0.88), 0.10, "dark")] + [
            B((x, 0.49, z), (0.72, 0.028, 0.07), "teal", (0, 0, -0.16 if x < 0 else 0.16)) for x in [-0.65, 0.65] for z in [-0.43, -0.13, 0.17, 0.47]]


def research_team():
    return [p for x, z, mat in [(-0.8, 0.1, "ivory"), (0, -0.28, "teal"), (0.8, 0.1, "amber")] for p in
            [L((x, 0, z), [(0, 0.48), (0.18, 0.49), (0.55, 0.4), (0.85, 0.22)], mat), S((x, 1.14, z), (0.30, 0.34, 0.30), mat)]]


def funding():
    return [B((-0.35, 1.0, -0.25), (1.4, 2, 0.18)), C((-0.35, 0.75, -0.09), 0.0 + 0.22, 0.12, "gold"),
            S((-0.35, 0.75, -0.09), (0.24, 0.24, 0.09), "gold"),
            B((-0.44, 0.45, -0.1), (0.14, 0.33, 0.08), "gold", (0, 0, -0.15)),
            B((-0.23, 0.45, -0.1), (0.14, 0.33, 0.08), "gold", (0, 0, 0.15))] + [
            B((-0.35, y, -0.13), (0.9, 0.07, 0.05), "teal") for y in [1.35, 1.6, 1.8]] + [C((0.75, 0.10 + i * 0.19, 0.25), 0.42, 0.16, "gold") for i in range(4)]


def collaboration():
    nodes = [(-1, 0.42, 0.35), (1, 0.42, 0.35), (0, 1.85, -0.15)]
    return [S(p, (0.42, 0.42, 0.42)) for p in nodes] + [R(a, b, 0.12, "teal") for a, b in zip(nodes, nodes[1:] + nodes[:1])]


def field_site():
    return [B((0.55, 0.52, 0), (1.3, 1.04, 1.1)),
            E([(-0.78, 0), (0, 0.45), (0.78, 0)], 1.3, "teal", (0.55, 1.04, 0)),
            B((0.5, 0.35, 0.57), (0.32, 0.70, 0.04), "teal"), B((0.94, 0.65, 0.57), (0.28, 0.3, 0.04), "teal"),
            R((-1, 0, 0), (-1, 2.0, 0), 0.055, "steel"), R((-1.4, 1.9, 0), (-0.6, 1.9, 0), 0.04),
            S((-1.4, 1.9, 0), (0.12, 0.12, 0.12)), S((-0.6, 1.9, 0), (0.12, 0.12, 0.12))] + [C((-1, y, 0), 0.18, 0.07) for y in [1.0, 1.12, 1.24, 1.36]]


RESEARCH = [("laboratory", "Laboratory", laboratory), ("doe-national-laboratory", "DOE National Laboratory", doe_national_laboratory),
            ("user-facility", "User Facility", user_facility), ("research-project", "Research Project", research_project),
            ("dataset", "Dataset", dataset), ("software", "Software", software), ("publication", "Publication", publication),
            ("research-team", "Research Team", research_team), ("funding", "Funding", funding),
            ("collaboration", "Collaboration", collaboration), ("field-site", "Field Site", field_site)]


def recipe(category, name, label, parts, radius=2.2, max_height=3.6, pedestal=True):
    vertices = [v for p in parts for v in p["vertices"]]
    lower = [min(v[a] for v in vertices) for a in range(3)]
    upper = [max(v[a] for v in vertices) for a in range(3)]
    factor = min((radius * 1.70) / max(upper[0] - lower[0], upper[2] - lower[2]), max_height / (upper[1] - lower[1]))
    factor = factor if pedestal else 1.0
    offset = (-(upper[0] + lower[0]) / 2, -lower[1], -(upper[2] + lower[2]) / 2)
    fitted = [dict(p, vertices=[[round((v[a] + offset[a]) * factor + (0.18 if pedestal and a == 1 else 0), 7) for a in range(3)] for v in p["vertices"]]) for p in parts]
    base = [part(G.lathe([(0, radius - 0.04), (0.04, radius), (0.14, radius), (0.18, radius - 0.04)], 64), "pedestal", group="pedestal")] if pedestal else []
    final = [dict(p, name=f"{p['group']}-{i:03d}") for i, p in enumerate(base + fitted)]
    used = sorted({p["material"] for p in final})
    materials = {key: BRAND.material(key) for key in used}
    if name == "statue-of-liberty":
        materials["copper"] = {"extras": {"brand": "natural finish", "colorName": "Oxidized copper patina", "srgbHex": "#78A99A"},
                               "pbrMetallicRoughness": {"baseColorFactor": [*BRAND.linear("#78A99A"), 1], "metallicFactor": 0.05, "roughnessFactor": 0.85}}
    return dict(id=f"{category}/{name}", category=category, label=label, revision=5 if name in {"research-project", "statue-of-liberty"} else 4, materials=materials, parts=final,
                provenance="Original ACORN procedural miniature with ORNL brand palette and requested natural earth finishes; simplified illustrative geometry, not a measured architectural or geographic replica. No imported meshes or vendor branding.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--category", choices=["drone", "research", "landmarks", "wonders", "risk"], required=True)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/models")
    args = parser.parse_args()
    collection = {"drone": DRONE, "research": RESEARCH, "landmarks": LANDMARKS, "wonders": WONDERS, "risk": RISK}[args.category]
    folder = args.root / ("drones" if args.category == "drone" else args.category)
    folder.mkdir(parents=True, exist_ok=True)
    (args.root / "palette.json").write_text(json.dumps({"brand": "ORNL", "source": "acorn-lib/src/util/constants/app.rs ORNL_COLOR_*", "srgb": BRAND.COLORS, "customAccents": BRAND.ACCENTS, "naturalFinishes": dict(BRAND.NATURAL.values()) | {"Oxidized copper patina": "#78A99A"}}, indent=2, sort_keys=True) + "\n")
    for name, label, create in collection:
        value = recipe(args.category, name, label, DETAIL.refine(name, create(), sys.modules[__name__], args.category), radius=1.2 if args.category == "risk" else 2.2,
                       max_height=2.8 if args.category == "risk" else 3.6, pedestal=not name.startswith("base-"))
        if args.category == "drone":
            value["provenance"] = f"Original simplified illustration inspired by {label}; ORNL palette; not manufacturer CAD or a flight/simulation model. See drones/README.md for source and selection context."
        (folder / f"{name}.model.json").write_text(json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n")
        print(f"Authored {value['id']}")


def statue_of_liberty():
    return [B((0, 0.18, 0), (1.0, 0.36, 0.95), "sand"), B((0, 0.55, 0), (0.70, 0.40, 0.65), "ivory"),
            L((0, 0.75, 0), [(0, 0.48), (0.15, 0.48), (1.15, 0.26), (1.45, 0.32), (1.6, 0.17)], "copper", scale=(1, 1, 0.7)),
            S((0, 2.62, 0), (0.22, 0.29, 0.19), "copper"),
            C((0, 2.83, 0), 0.27, 0.08, "copper"),
            R((-0.20, 2.12, 0), (-0.43, 2.62, 0), 0.14, "copper", 0.09),
            R((-0.43, 2.62, 0), (-0.55, 3.18, 0), 0.09, "copper", 0.075),
            C((-0.55, 3.22, 0), 0.14, 0.12, "copper"),
            L((-0.55, 3.25, 0), [(0, 0.13), (0.16, 0.12), (0.35, 0)], "gold"),
            R((0.23, 2.13, 0), (0.4, 1.75, 0.22), 0.12, "copper"),
            B((0.38, 1.75, 0.3), (0.3, 0.48, 0.12), "copper", (0.12, 0, -0.18))] + [
            R((0.21 * math.cos(a), 2.82 + 0.13 * math.sin(a), 0), (0.48 * math.cos(a), 2.85 + 0.4 * math.sin(a), 0), 0.04, "copper", 0.008)
            for a in [i * PI / 6 for i in range(7)]] + [R((x, 0.83, 0.29), (x * 0.3, 2.05, 0.18), 0.035, "copper") for x in [-0.33, -0.16, 0, 0.16, 0.33]]


def golden_gate_bridge():
    def cable_y(x):
        return 1.15 + 1.9 * (x / 1.7) ** 2 if abs(x) <= 1.7 else 3.05 - (abs(x) - 1.7) * 1.5
    xs = [-3 + i * 0.15 for i in range(41)]
    return [B((0, 0.05, 0), (6.3, 0.1, 1.6), "water"), B((0, 0.75, 0), (6.2, 0.16, 0.75), "red"),
            B((0, 0.85, 0), (6.1, 0.05, 0.58), "dark")] + [
            item for x in [-1.7, 1.7] for item in [B((x, 1.6, -0.32), (0.17, 3.2, 0.17), "red"),
            B((x, 1.6, 0.32), (0.17, 3.2, 0.17), "red"), B((x, 0.25, 0), (0.50, 0.50, 1.0), "ivory"),
            *[B((x, y, 0), (0.19, 0.12, 0.75), "red") for y in [1.15, 2.15, 3.1]]]] + [
            R((a, cable_y(a), z), (b, cable_y(b), z), 0.04, "red") for a, b in zip(xs, xs[1:]) for z in [-0.32, 0.32]] + [
            R((x, 0.85, z), (x, cable_y(x), z), 0.018, "red") for x in xs[::2] for z in [-0.32, 0.32]]


def us_capitol():
    return [B((0, 0.15, 0), (4.8, 0.3, 1.8)), B((0, 0.68, 0), (1.4, 1.0, 1.5)),
            *[B((x, 0.60, -0.05), (1.65, 0.90, 1.2)) for x in [-1.5, 1.5]],
            *[B((x, 1.1, -0.05), (1.78, 0.15, 1.35)) for x in [-1.5, 1.5]],
            L((0, 1.15, 0), [(0, 0.61), (0.22, 0.61), (0.26, 0.66), (0.33, 0.65), (0.48, 0.57), (0.72, 0.44), (0.95, 0.23), (1.02, 0.12)]),
            C((0, 2.25, 0), 0.13, 0.26), L((0, 2.38, 0), [(0, 0.18), (0.16, 0)]),
            E([(-0.78, 0), (0, 0.36), (0.78, 0)], 0.38, center=(0, 1.0, 0.84)),
            *[B((0, 0.04 + i * 0.07, 1.2 - i * 0.1), (1.7, 0.08, 0.35)) for i in range(5)]] + [
            C((x, 0.62, 0.97), 0.055, 0.75, "ivory", 12) for x in [-0.55, -0.33, -0.11, 0.11, 0.33, 0.55]] + [
            B((x, y, 0.568), (0.12, 0.22, 0.03), "dark") for x in [-2.1, -1.8, -1.5, -1.2, -0.9, 0.9, 1.2, 1.5, 1.8, 2.1] for y in [0.46, 0.82]]


def gateway_arch():
    outside = [(-1.4 + 2.8 * i / 40, 3.2 * (1 - ((-1.4 + 2.8 * i / 40) / 1.4) ** 2)) for i in range(41)]
    inside = [(1.15 - 2.3 * i / 40, 2.95 * (1 - ((1.15 - 2.3 * i / 40) / 1.15) ** 2)) for i in range(41)]
    return [E(outside + inside, 0.20, "steel")]


def grand_canyon():
    heights = [[0.10 if abs((x - 24) / 12 - 0.3 * math.sin(z * 0.244)) < 0.22 else
                0.22 + 0.19 * math.floor(min(5, abs((x - 24) / 12 - 0.3 * math.sin(z * 0.244)) * 3)) + 0.025 * math.sin(x * 0.7 + z * 0.6)
                for x in range(49)] for z in range(33)]
    mesh = G.terrain(heights, 4.0, 3.0, bottom=0)
    colors = ["terracotta", "rose", "sand"]
    groups = [part((mesh[0], [face for face in mesh[1] if min(2, int(sum(mesh[0][i][1] for i in face) / 3 * 2.4)) == index]), color) for index, color in enumerate(colors)]
    river = [(0.3 * math.sin(z * 0.244) - 0.18, (z - 16) * 3 / 32) for z in range(33)] + [(0.3 * math.sin(z * 0.244) + 0.18, (z - 16) * 3 / 32) for z in reversed(range(33))]
    return groups + [E(river, 0.035, "water", (0, 0.13, 0), (PI / 2, 0, 0))]


def yellowstone():
    return [L((0, 0, 0), [(0, 1.7), (0.13, 1.7), (0.2, 1.35), (0.25, 1.0), (0.28, 0.6)], "ivory", 40, (1, 1, 0.8)),
            part(G.torus(0.70, 0.20), "orange", (0, 0.23, 0), scale=(1, 0.24, 0.8)),
            part(G.lathe([(0, 0.55), (0.03, 0.55)]), "water", (0, 0.27, 0), scale=(1, 1, 0.8)),
            C((0.97, 0.24, -0.65), 0.23, 0.40, "sand"),
            R((0.97, 0.35, -0.65), (1.02, 1.27, -0.65), 0.10, "white", 0.17)] + [
            S((1.0 + 0.10 * math.sin(i * 2), 0.6 + i * 0.18, -0.65), (0.18 + i * 0.025, 0.19, 0.18 + i * 0.025), "white") for i in range(6)]


def kennedy_space_center():
    return [B((0, 0.08, 0), (2.1, 0.16, 1.7), "sand"),
            L((-0.35, 0.16, 0.15), [(0, 0.27), (0.2, 0.21), (2.25, 0.21), (2.65, 0)], "white"),
            C((-0.35, 1.55, 0.15), 0.216, 0.18, "dark"),
            L((-0.69, 0.18, 0.15), [(0, 0.11), (1.4, 0.11), (1.65, 0)], "white"),
            L((-0.01, 0.18, 0.15), [(0, 0.11), (1.4, 0.11), (1.65, 0)], "white"),
            *[R((x, 0.16, z), (x, 2.7, z), 0.045, "steel") for x in [0.5, 1.0] for z in [-0.3, 0.2]],
            *[B((0.75, y, -0.05), (0.62, 0.07, 0.62), "steel") for y in [0.3, 0.9, 1.5, 2.1, 2.7]],
            *[R((0.5, y, 0.2), (1.0, y + 0.6, 0.2), 0.025, "steel") for y in [0.3, 0.9, 1.5, 2.1]],
            B((0.15, 2.1, 0.1), (0.75, 0.07, 0.20), "steel")]


def hoover_dam():
    a0, a1 = 0.22 * PI, 0.78 * PI
    dam = part(G.arch(1.6, 0.28, 1.28, a0, a1, 24), "ivory", (0, 0.76, -0.7), rotation=(PI / 2, 0, 0))
    crest = part(G.arch(1.64, 0.33, 0.10, a0, a1, 24), "pages", (0, 1.44, -0.7), rotation=(PI / 2, 0, 0))
    return [dam, crest, B((0, 0.12, -0.25), (2.7, 0.24, 2.3), "water"),
            *[L((x, 0, -0.3), [(0, 0.7), (0.35, 0.75), (0.7, 0.62), (1.1, 0.67), (1.55, 0.44)], "terracotta", 7, (1, 1, 1.5)) for x in [-1.4, 1.4]],
            *[R((1.59 * math.cos(a), 0.2, 1.59 * math.sin(a) - 0.7), (1.59 * math.cos(a), 1.40, 1.59 * math.sin(a) - 0.7), 0.025) for a in [a0 + (a1 - a0) * i / 12 for i in range(13)]]]


LANDMARKS = [("statue-of-liberty", "Statue of Liberty", statue_of_liberty), ("golden-gate-bridge", "Golden Gate Bridge", golden_gate_bridge),
             ("us-capitol", "U.S. Capitol", us_capitol), ("gateway-arch", "Gateway Arch", gateway_arch),
             ("grand-canyon", "Grand Canyon", grand_canyon), ("yellowstone", "Yellowstone", yellowstone),
             ("kennedy-space-center", "Kennedy Space Center", kennedy_space_center), ("hoover-dam", "Hoover Dam", hoover_dam)]
def chichen_itza():
    levels = [B((0, 0.11 + i * 0.20, 0), (3.2 - i * 0.25, 0.22, 3.2 - i * 0.25), "sand") for i in range(9)]
    stairs = [B((0, 0.045 + i * 0.078, 1.65 - i * 0.049), (0.50, 0.09, 0.10), "ivory") for i in range(23)]
    return levels + stairs + [B((0, 2.05, 0), (0.85, 0.5, 0.82), "sand"), B((0, 2.34, 0), (1.02, 0.12, 0.95), "ivory"), B((0, 2.03, 0.42), (0.26, 0.4, 0.035), "dark")]


def christ_redeemer():
    return [L((0, 0, 0), [(0, 1.1), (0.20, 1), (0.45, 0.7), (0.65, 0.42)], "grass", 9),
            B((0, 0.72, 0), (0.65, 0.2, 0.62), "steel"),
            E([(-0.37, 0), (0.37, 0), (0.23, 1.3), (0.40, 1.5), (0.27, 1.7), (-0.27, 1.7), (-0.40, 1.5), (-0.23, 1.3)], 0.36, "ivory", (0, 0.82, 0)),
            E([(-1.45, 1.42), (-1.45, 1.62), (-0.25, 1.70), (-0.25, 1.29)], 0.25, "ivory", (0, 0.82, 0)),
            E([(0.25, 1.29), (0.25, 1.70), (1.45, 1.62), (1.45, 1.42)], 0.25, "ivory", (0, 0.82, 0)),
            S((-1.54, 2.35, 0), (0.15, 0.08, 0.12)), S((1.54, 2.35, 0), (0.15, 0.08, 0.12)),
            S((0, 2.77, 0), (0.20, 0.27, 0.19)),
            *[R((x * 1.4, 0.84, 0.20), (x, 2.0, 0.20), 0.018, "pages") for x in [-0.16, 0, 0.16]]]


def colosseum():
    # Separate open arches make the arcades readable from either side.
    arcades = []
    for level in range(3):
        for i in range(24):
            angle = 2 * PI * i / 24
            if level == 2 and 3 <= i <= 9:
                continue
            center = (1.7 * math.sin(angle), 0.34 + level * 0.48, 1.25 * math.cos(angle))
            arcades += [B((center[0], center[1] - 0.10, center[2]), (0.12, 0.40, 0.18), "sand", (0, angle, 0)),
                        part(G.arch(0.22, 0.07, 0.18, steps=8), "sand", (center[0], center[1] + 0.06, center[2]), rotation=(0, angle, 0))]
    rims = [part(G.arch(1.76, 0.20, 0.10, 0, 2 * PI - 0.001, 72), "ivory", (0, y, 0), scale=(1, 0.74, 1), rotation=(PI / 2, 0, 0)) for y in [0.10, 0.58, 1.06]]
    return [part(G.lathe([(0, 1.45), (0.10, 1.45)]), "terracotta", scale=(1, 1, 0.70))] + arcades + rims


def great_pyramid_giza():
    return [B((0, 0.08, 0), (3.5, 0.16, 2.8), "sand"), P((-0.55, 0.16, -0.1), (2.2, 1.7, 2.2), "sand"),
            P((1.05, 0.16, -0.35), (1.15, 0.9, 1.15), "ivory"), P((0.95, 0.16, 0.85), (0.63, 0.48, 0.63), "sand"),
            *[P((-1.25 + i * 0.40, 0.16, 1.15), (0.30, 0.24, 0.30), "ivory") for i in range(3)]]


def great_wall():
    heights = [[0.22 + 0.38 * (math.sin(x * 0.60 + z * 0.35) + 1) for x in range(13)] for z in range(9)]
    route = [(x, 0.9 + 0.18 * math.cos(x * 1.7), 0.48 * math.sin(x * 1.6)) for x in [-2 + i / 4 for i in range(17)]]
    parts = [part(G.terrain(heights, 4.8, 2.8), "grass")]
    for a, b in zip(route, route[1:]):
        dx, dz = b[0] - a[0], b[2] - a[2]
        angle = -math.atan2(dz, dx)
        x, y, z = [(a[k] + b[k]) / 2 for k in range(3)]
        parts += [B((x, y, z), (math.hypot(dx, dz) + 0.06, 0.65, 0.37), "sand", (0, angle, 0)),
                  *[B((x + side * 0.19 * math.sin(angle), y + 0.40, z + side * 0.19 * math.cos(angle)), (0.12, 0.18, 0.09), "ivory", (0, angle, 0)) for side in [-1, 1]]]
    for x, y, z in [route[1], route[8], route[15]]:
        parts += [B((x, y + 0.25, z), (0.58, 0.95, 0.58), "sand"), B((x, y + 0.77, z), (0.70, 0.12, 0.70), "ivory"),
                  *[B((x + dx, y + 0.90, z + dz), (0.15, 0.20, 0.15), "sand") for dx in [-0.25, 0.25] for dz in [-0.25, 0.25]]]
    return parts


def machu_picchu():
    return [L((0.65, 0, -0.60), [(0, 0.90), (0.6, 0.78), (1.5, 0.48), (2.2, 0.12), (2.35, 0)], "grass", 7, (0.85, 1, 0.7)),
            L((-0.80, 0, -0.8), [(0, 0.9), (0.55, 0.7), (1.1, 0.25), (1.2, 0)], "leaf", 6),
            *[B((0, 0.1 + i * 0.19, 0.5 - i * 0.10), (3.0 - i * 0.30, 0.20, 1.5 - i * 0.16), "grass" if i % 2 else "sand") for i in range(5)],
            *[p for x, z, y in [(-0.8, 0.5, 0.85), (-0.1, 0.5, 1.0), (0.65, 0.5, 0.85), (-0.4, -0.05, 1.0)] for p in
              [B((x, y + 0.16, z), (0.45, 0.32, 0.35), "sand"), E([(-0.27, 0), (0, 0.2), (0.27, 0)], 0.42, "ivory", (x, y + 0.32, z)), B((x, y + 0.12, z + 0.18), (0.12, 0.23, 0.025), "dark")]]]


def petra():
    cliffs = [L((x, 0, -0.35), [(0, 0.70), (0.7, 0.75), (1.5, 0.57), (2.3, 0.65), (2.9, 0.4)], "rose", 7, (0.8, 1, 0.8)) for x in [-1.22, 1.22]]
    return cliffs + [B((0, 1.32, -0.35), (2.25, 2.64, 0.55), "rose"), B((0, 0.08, 0.2), (2.5, 0.16, 1.1), "sand"),
            B((0, 0.62, -0.015), (0.48, 0.9, 0.06), "dark"),
            *[C((x, 0.68, 0.13), 0.10, 1.0, "sand", 12) for x in [-0.95, -0.60, 0.60, 0.95]],
            B((0, 1.23, 0.12), (2.25, 0.16, 0.42), "sand"), E([(-1.05, 0), (0, 0.40), (1.05, 0)], 0.36, "sand", (0, 1.31, 0.12)),
            *[C((x, 1.96, 0.13), 0.085, 0.76, "sand", 12) for x in [-0.93, -0.55, -0.18, 0.18, 0.55, 0.93]],
            *[B((x, 2.36, 0.13), (0.68, 0.12, 0.40), "sand") for x in [-0.65, 0.65]],
            L((0, 2.3, 0.12), [(0, 0.3), (0.1, 0.3), (0.38, 0.05), (0.49, 0.05)], "sand"),
            *[B((0, 0.02 + i * 0.05, 0.62 - i * 0.12), (2.15, 0.06, 0.18), "ivory") for i in range(3)]]


def taj_mahal():
    dome = [(0, 0.46), (0.15, 0.59), (0.35, 0.62), (0.53, 0.53), (0.71, 0.30), (0.91, 0.04), (1.05, 0)]
    return [B((0, 0.12, 0), (3.3, 0.24, 2.6), "ivory"), B((0, 0.74, 0), (1.75, 1.0, 1.35), "white"),
            L((0, 1.25, 0), dome, "white"),
            E([(-0.29, 0), (0.29, 0), (0.29, 0.50), (0, 0.78), (-0.29, 0.50)], 0.035, "dark", (0, 0.25, 0.69)),
            *[p for x in [-1.37, 1.37] for z in [-1.03, 1.03] for p in [L((x, 0.24, z), [(0, 0.15), (1.35, 0.10)], "white"), C((x, 1.10, z), 0.18, 0.08), C((x, 1.60, z), 0.17, 0.10), L((x, 1.65, z), [(0, 0.15), (0.1, 0.16), (0.28, 0)], "white")]],
            *[L((x, 1.25, z), [(y * 0.35, r * 0.35) for y, r in dome], "white", 20) for x in [-0.65, 0.65] for z in [-0.45, 0.45]],
            B((0, 0.25, 1.0), (0.5, 0.025, 0.55), "water")]


WONDERS = [("chichen-itza", "Chichen Itza", chichen_itza), ("christ-the-redeemer", "Christ the Redeemer", christ_redeemer),
           ("colosseum", "Colosseum", colosseum), ("great-pyramid-of-giza", "Great Pyramid of Giza", great_pyramid_giza),
           ("great-wall-of-china", "Great Wall of China", great_wall), ("machu-picchu", "Machu Picchu", machu_picchu),
           ("petra", "Petra", petra), ("taj-mahal", "Taj Mahal", taj_mahal)]

def infantry():
    return [B((-0.22, 0.12, 0.08), (0.30, 0.24, 0.55), "dark"), B((0.22, 0.12, 0.08), (0.30, 0.24, 0.55), "dark"),
            R((-0.22, 0.22, 0), (-0.18, 0.9, 0), 0.16, "teal"), R((0.22, 0.22, 0), (0.18, 0.9, 0), 0.16, "teal"),
            L((0, 0.75, 0), [(0, 0.36), (0.65, 0.42), (0.78, 0.20)], "teal", 12, (1, 1, 0.65)),
            S((0, 1.78, 0.02), (0.25, 0.28, 0.24), "ivory"), C((0, 1.97, 0), 0.32, 0.08, "teal"),
            L((0, 2.0, 0), [(0, 0.26), (0.12, 0.22), (0.18, 0)], "teal"),
            R((-0.36, 1.37, 0), (-0.44, 0.98, 0.24), 0.13, "teal"), R((-0.44, 0.98, 0.24), (0.25, 1.18, 0.34), 0.10, "ivory"),
            R((0.36, 1.37, 0), (0.40, 1.1, 0.31), 0.13, "teal"),
            R((-0.45, 0.94, 0.39), (0.65, 1.46, 0.39), 0.065, "dark"), B((-0.30, 1.0, 0.39), (0.36, 0.15, 0.16), "steel", (0, 0, 0.42))]


def cavalry():
    horse = [S((0, 0.95, 0), (0.72, 0.37, 0.31), "ivory"),
             R((0.48, 1.0, 0), (0.66, 1.65, 0), 0.25, "ivory", 0.17), S((0.78, 1.65, 0), (0.32, 0.18, 0.17), "ivory"),
             *[P((0.60, 1.77, z), (0.12, 0.23, 0.12), "ivory") for z in [-0.12, 0.12]],
             *[p for x in [-0.45, 0.45] for z in [-0.22, 0.22] for p in [R((x, 0.9, z), (x + (0.10 if x < 0 else -0.07), 0.12, z), 0.08, "ivory"), B((x + (0.10 if x < 0 else -0.07), 0.08, z), (0.22, 0.16, 0.17), "dark")]],
             R((-0.63, 1.08, 0), (-0.86, 0.45, 0), 0.09, "dark"), B((0, 1.26, 0), (0.65, 0.12, 0.70), "teal")]
    rider = [R((0, 1.47, -0.34), (0, 0.9, -0.38), 0.11, "teal"), R((0, 1.47, 0.34), (0, 0.9, 0.38), 0.11, "teal"),
             L((0, 1.36, 0), [(0, 0.22), (0.49, 0.26), (0.60, 0.15)], "teal", 12), S((0, 2.17, 0), (0.19, 0.23, 0.19), "ivory"),
             C((0, 2.35, 0), 0.25, 0.09, "teal"), R((0.15, 1.82, 0.17), (0.5, 1.58, 0.17), 0.08, "teal"),
             R((0.5, 1.58, 0.17), (0.82, 1.57, 0.14), 0.025, "dark")]
    return horse + rider


def artillery():
    wheels = [p for x in [-0.68, 0.68] for p in [T((x, 0.52, 0), 0.43, 0.075, "dark", (0, 0, PI / 2)),
              R((x - 0.08, 0.52, 0), (x + 0.08, 0.52, 0), 0.12, "steel"),
              *[R((x, 0.52, 0), (x, 0.52 + 0.42 * math.cos(a), 0.42 * math.sin(a)), 0.035, "steel") for a in [i * PI / 4 for i in range(8)]]]]
    barrel = part(G.lathe([(0, 0.23), (0.15, 0.25), (1.35, 0.16), (1.42, 0.20), (1.50, 0.20)]), "teal", (0, 0.85, -0.4), rotation=(PI / 2 - 0.12, 0, 0), smooth=True)
    muzzle = part(G.lathe([(0, 0.135), (0.012, 0.135)]), "dark", (0, 1.03, 1.09), rotation=(PI / 2 - 0.12, 0, 0))
    return wheels + [barrel, muzzle, B((0, 0.5, -0.15), (0.9, 0.22, 0.60), "steel"), R((-0.30, 0.5, -0.1), (-0.38, 0.1, -1.0), 0.10, "teal"), R((0.30, 0.5, -0.1), (0.38, 0.1, -1.0), 0.10, "teal")]


def scout():
    return [C((0, 0.16, 0), 0.55, 0.32, "steel", 6), P((0, 0.32, 0), (0.78, 1.8, 0.78), "teal", 0.32),
            P((0, 2.12, 0), (0.32, 0.45, 0.32), "ivory"), B((0, 1.10, 0.30), (0.30, 0.10, 0.05), "white")]


def squadron():
    return [C((0, 0.14, 0), 0.74, 0.28, "steel", 8), L((0, 0.28, 0), [(0, 0.60), (0.30, 0.45), (1.6, 0.45)], "teal", 8),
            C((0, 1.85, 0), 0.70, 0.22, "ivory", 8),
            *[B((x, 2.12, z), (0.32, 0.48, 0.32), "teal") for x in [-0.40, 0.40] for z in [-0.40, 0.40]],
            *[B((x, 1.15, 0.44), (0.11, 0.42, 0.045), "white") for x in [-0.15, 0.15]]]


def legion():
    return [C((0, 0.12, 0), 1.05, 0.24, "steel", 6), C((0, 0.70, 0), 0.88, 1.15, "teal", 6),
            C((0, 1.28, 0), 1.0, 0.18, "ivory", 6),
            *[C((0.78 * math.cos(a), 1.55, 0.78 * math.sin(a)), 0.25, 0.50, "teal", 6) for a in [i * PI / 3 for i in range(6)]],
            P((0, 1.38, 0), (0.66, 0.70, 0.66), "teal", 0.25),
            *[B((x, 0.78, 0.77), (0.10, 0.44, 0.035), "white") for x in [-0.25, 0, 0.25]]]


def faction_base(shape, color):
    polygons = {
        "diamond": [(0, -1), (0.75, 0), (0, 1), (-0.75, 0)],
        "plus": [(-0.3, -1), (0.3, -1), (0.3, -0.3), (1, -0.3), (1, 0.3), (0.3, 0.3), (0.3, 1), (-0.3, 1), (-0.3, 0.3), (-1, 0.3), (-1, -0.3), (-0.3, -0.3)],
        "square": [(-0.8, -0.8), (0.8, -0.8), (0.8, 0.8), (-0.8, 0.8)],
        "star": [((1 if i % 2 == 0 else 0.44) * math.cos(PI / 2 + i * PI / 5), (1 if i % 2 == 0 else 0.44) * math.sin(PI / 2 + i * PI / 5)) for i in range(10)],
        "triangle": [(-0.9, -0.7), (0.9, -0.7), (0, 0.9)],
    }
    symbols = [C((0, 0.22, z), 0.15, 0.06, "white") if shape == "circle" else
               E([(x * 0.16, y * 0.16) for x, y in polygons[shape]], 0.06, "white", (0, 0.22, z), (PI / 2, 0, 0)) for z in [-1.11, 1.11]]
    return [dict(C((0, 0.09, 0), 1.35, 0.18, color, 64), group="pedestal"),
            dict(C((0, 0.185, 0), 0.87, 0.01, color, 64), group="pedestal")] + [dict(p, group="symbol") for p in symbols]


RISK = [("artillery", "Artillery", artillery), ("cavalry", "Cavalry", cavalry), ("infantry", "Infantry", infantry),
        ("legion", "Legion", legion), ("scout", "Scout", scout), ("squadron", "Squadron", squadron)] + [
        (f"base-{shape}", f"{BRAND.NATURAL.get(color, (BRAND.ROLES[color],))[0]} / {shape.title()} Base", lambda s=shape, c=color: faction_base(s, c))
        for shape, color in [("circle", "teal"), ("diamond", "magenta"), ("plus", "orange"), ("square", "blue"), ("star", "purple"), ("triangle", "amber")]]



def propeller(center, radius=0.42, angle=0, rotation=(0, 0, 0)):
    outline = [(-radius, -0.025), (-0.82 * radius, -0.10), (-0.30 * radius, -0.12), (0.05, -0.04), (radius, 0.025), (0.82 * radius, 0.10), (0.30 * radius, 0.12), (-0.05, 0.04)]
    blade = G.transform(G.extrude(outline, 0.025), rotation=(PI / 2, angle, 0))
    hub = G.lathe([(-0.04, 0.06), (0.06, 0.06)], 12)
    return [part(blade, "dark", center, rotation=rotation), part(hub, "steel", center, rotation=rotation)]


def camera(center, size=0.22, lenses=1):
    x, y, z = center
    return [B(center, (size * 1.7, size, size), "dark"),
            *[T((x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size * 0.67), size * 0.22, size * 0.035, "steel", (PI / 2, 0, 0)) for i in range(lenses)],
            *[R((x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size / 2), (x + (i - (lenses - 1) / 2) * size * 0.48, y, z + size * 0.66), size * 0.20, "water") for i in range(lenses)]]


def quad_arms(span=1.1, length=0.8, y=0.75, rotor=0.45, material="ivory", rear_lift=0):
    return [p for x in [-span, span] for z in [-length, length] for p in
            [R((x * 0.18, y, z * 0.25), (x, y + (rear_lift if z < 0 else 0), z), 0.065, material),
             C((x, y + (rear_lift if z < 0 else 0), z), 0.105, 0.18, "dark", 16),
             *propeller((x, y + 0.14 + (rear_lift if z < 0 else 0), z), rotor, 0.45 if x * z > 0 else -0.45)]]


def skids(width=0.45, depth=0.70, height=0.65, material="dark"):
    return [p for x in [-width, width] for p in [R((x, 0.06, -depth), (x, 0.06, depth), 0.045, material),
            *[R((x * 0.65, height, z * 0.7), (x, 0.06, z), 0.035, material) for z in [-depth * 0.65, depth * 0.65]]]]


def dji_mavic_3_enterprise():
    return quad_arms(0.85, 0.65, 0.42, 0.36) + [B((0, 0.48, 0), (0.48, 0.30, 1.0)),
            B((0, 0.65, -0.12), (0.37, 0.10, 0.58), "teal"), C((0, 0.77, -0.15), 0.13, 0.14),
            *camera((0, 0.26, 0.49), 0.22, 2), *[R((x, 0.42, 0.65), (x, 0.06, 0.65), 0.04) for x in [-0.85, 0.85]]]


def dji_matrice_350_rtk():
    return quad_arms(1.1, 0.85, 1.05, 0.50, "dark") + skids(0.55, 0.75, 1.05) + [
            B((0, 1.05, 0), (0.64, 0.38, 0.88)), B((0, 1.28, -0.07), (0.54, 0.12, 0.64), "teal"),
            *[R((x, 1.2, -0.25), (x, 1.65, -0.25), 0.035, "dark") for x in [-0.29, 0.29]],
            *[C((x, 1.65, -0.25), 0.09, 0.06) for x in [-0.29, 0.29]],
            R((0, 0.91, 0.20), (0, 0.55, 0.20), 0.07, "dark"), S((0, 0.48, 0.20), (0.22, 0.21, 0.20), "ivory"), *camera((0, 0.48, 0.38), 0.22, 2)]


def dji_phantom_4_rtk():
    return quad_arms(0.83, 0.70, 0.85, 0.40, "white") + skids(0.34, 0.58, 0.85, "white") + [
            S((0, 0.86, 0), (0.43, 0.21, 0.48), "white"), C((0, 1.20, 0), 0.12, 0.30, "ivory"),
            C((0, 1.38, 0), 0.20, 0.10, "teal"), R((0, 0.73, 0.20), (0, 0.48, 0.20), 0.05), *camera((0, 0.41, 0.27), 0.25)]


def parrot_anafi_ai():
    return quad_arms(0.95, 0.68, 0.48, 0.36, "white") + [
            S((0, 0.48, -0.07), (0.22, 0.16, 0.64), "white"), B((0, 0.60, -0.16), (0.30, 0.09, 0.62), "teal"),
            *[S((x, 0.49, 0.56), (0.15, 0.16, 0.16), "white") for x in [-0.25, 0.25]],
            *[R((x, 0.49, 0.68), (x, 0.49, 0.72), 0.08, "dark") for x in [-0.25, 0.25]],
            *camera((0, 0.27, 0.61), 0.22), *[R((x, 0.44, z), (x, 0.06, z), 0.03) for x in [-0.95, 0.95] for z in [-0.68, 0.68]]]


def autel_evo_max_4t():
    return quad_arms(0.94, 0.72, 0.6, 0.40, "ivory", 0.06) + [
            B((0, 0.62, -0.02), (0.60, 0.34, 1.08)), B((0, 0.82, -0.1), (0.48, 0.10, 0.72), "teal"),
            S((0, 0.64, 0.50), (0.26, 0.18, 0.12), "dark"), *camera((0, 0.29, 0.52), 0.30, 3),
            *[R((x, 0.6, 0.72), (x, 0.06, 0.72), 0.04) for x in [-0.94, 0.94]]]


def skydio_x10():
    return quad_arms(1.02, 0.75, 0.58, 0.43, "dark", 0.20) + [
            B((0, 0.62, -0.02), (0.45, 0.25, 1.13), "teal"), B((0, 0.79, -0.18), (0.34, 0.10, 0.62), "dark"),
            *camera((0, 0.37, 0.60), 0.28, 3), *[S((x, 0.68, 0.33), (0.07, 0.07, 0.07), "white") for x in [-0.25, 0.25]],
            *[R((x, 0.55, z), (x, 0.06, z), 0.035, "dark") for x in [-0.40, 0.40] for z in [-0.43, 0.43]]]


def freefly_astro():
    return quad_arms(1.15, 0.83, 0.95, 0.52, "dark") + skids(0.6, 0.72, 0.95) + [
            B((0, 0.95, 0), (0.84, 0.12, 0.84), "dark"), *[B((x, 1.14, 0), (0.28, 0.28, 0.72), "teal") for x in [-0.22, 0.22]],
            C((0, 1.37, -0.18), 0.12, 0.10), R((0, 0.9, 0.12), (0, 0.52, 0.12), 0.05),
            *camera((0, 0.45, 0.22), 0.32), *[B((x, 0.45, 0.10), (0.06, 0.34, 0.32), "steel") for x in [-0.32, 0.32]]]


def holybro_x500_v2():
    return quad_arms(1.0, 0.83, 0.85, 0.44, "dark") + skids(0.50, 0.65, 0.85) + [
            B((0, 0.85, 0), (0.70, 0.06, 0.8), "dark"), B((0, 1.0, 0), (0.66, 0.05, 0.75), "dark"),
            B((0, 1.09, 0.05), (0.26, 0.12, 0.40), "teal"), B((0, 0.72, 0), (0.38, 0.20, 0.70), "ivory"),
            R((0, 1.03, -0.28), (0, 1.48, -0.28), 0.025, "dark"), C((0, 1.50, -0.28), 0.13, 0.06),
            *[C((x, 0.93, z), 0.025, 0.16, "steel", 8) for x in [-0.27, 0.27] for z in [-0.30, 0.30]],
            R((0.30, 1.02, 0.23), (0.35, 1.33, 0.23), 0.02, "dark")]


def crazyflie_2_1():
    return quad_arms(0.75, 0.75, 0.16, 0.34, "ivory") + [
            B((0, 0.19, 0), (0.55, 0.06, 0.65), "teal"), B((0, 0.30, 0.04), (0.32, 0.18, 0.43), "steel"),
            *[B((x, 0.24, z), (0.10, 0.05, 0.12), "dark") for x, z in [(-0.18, -0.20), (0.18, -0.20), (0.18, 0.20)]],
            *[C((x, 0.08, z), 0.065, 0.16, "steel", 12) for x in [-0.75, 0.75] for z in [-0.75, 0.75]],
            R((-0.24, 0.24, -0.27), (-0.30, 0.58, -0.35), 0.012, "dark")]


def ornl_strelka():
    return quad_arms(1.35, 1.05, 1.15, 0.60, "dark") + skids(0.70, 0.92, 1.15) + [
            B((0, 1.14, 0), (0.90, 0.14, 1.4), "dark"), B((0, 1.40, -0.1), (0.65, 0.38, 0.9), "teal"),
            *[B((x, 1.36, -0.2), (0.26, 0.30, 1.05), "ivory") for x in [-0.53, 0.53]],
            B((0, 0.65, 0), (0.65, 0.58, 0.92), "ivory"), B((0, 0.34, 0), (0.60, 0.08, 0.87), "teal"),
            R((0, 1.6, -0.4), (0, 2.0, -0.4), 0.025, "dark"), C((0, 2.01, -0.4), 0.11, 0.07)]


def ebee_x():
    wing = [(-2, -0.65), (-1.9, 0.10), (-0.40, 0.75), (0, 0.80), (0.4, 0.75), (1.9, 0.1), (2, -0.65), (0.45, -0.25), (-0.45, -0.25)]
    return [E(wing, 0.10, "teal", (0, 0.35, 0), (PI / 2, 0, 0)), S((0, 0.36, 0.30), (0.37, 0.20, 0.72), "ivory"),
            *[E([(-0.3, 0), (0.22, 0), (0.12, 0.45), (-0.12, 0.6)], 0.045, "teal", (x, 0.35, -0.35), (0, PI / 2, 0)) for x in [-1.91, 1.91]],
            R((0, 0.37, -0.4), (0, 0.37, -0.69), 0.10, "dark"), *propeller((0, 0.37, -0.73), 0.35, 0.3, (PI / 2, 0, 0)),
            *camera((0, 0.15, 0.40), 0.20)]


def wingtraone_gen_ii():
    wing = [(-1.9, -0.42), (-1.9, 0.12), (-0.45, 0.40), (0.45, 0.40), (1.9, 0.12), (1.9, -0.42)]
    return [E(wing, 0.12, "teal", (0, 0.75, 0), (PI / 2, 0, 0)), S((0, 0.75, 0.20), (0.28, 0.21, 0.90), "ivory"),
            *[p for x in [-0.85, 0.85] for p in [R((x, 0.76, 0), (x, 0.76, 0.63), 0.11, "ivory"), *propeller((x, 0.76, 0.70), 0.43, 0.4, (PI / 2, 0, 0))]],
            B((0, 0.75, -1.0), (1.1, 0.07, 0.32), "teal"),
            *[E([(-0.25, 0), (0.18, 0), (0.1, 0.48), (-0.1, 0.55)], 0.045, "teal", (x, 0.76, -0.99), (0, PI / 2, 0)) for x in [-0.48, 0.48]],
            *camera((0, 0.55, 0.36), 0.22)]


DRONE = [("autel-evo-max-4t", "Autel EVO Max 4T", autel_evo_max_4t), ("bitcraze-crazyflie-2-1", "Bitcraze Crazyflie 2.1", crazyflie_2_1),
         ("dji-matrice-350-rtk", "DJI Matrice 350 RTK", dji_matrice_350_rtk), ("dji-mavic-3-enterprise", "DJI Mavic 3 Enterprise", dji_mavic_3_enterprise),
         ("dji-phantom-4-rtk", "DJI Phantom 4 RTK", dji_phantom_4_rtk), ("ebee-x", "eBee X", ebee_x),
         ("freefly-astro", "Freefly Astro", freefly_astro), ("holybro-x500-v2", "Holybro X500 V2", holybro_x500_v2),
         ("ornl-strelka", "ORNL Strelka", ornl_strelka), ("parrot-anafi-ai", "Parrot ANAFI Ai", parrot_anafi_ai),
         ("skydio-x10", "Skydio X10", skydio_x10), ("wingtraone-gen-ii", "WingtraOne GEN II", wingtraone_gen_ii)]

if __name__ == "__main__":
    main()
